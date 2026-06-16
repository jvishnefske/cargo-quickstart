use anyhow::{bail, Context, Result};

use crate::cli::{Cli, Edition, License, Template};

/// Fully-resolved generation settings (after merging flags + prompts + defaults).
#[derive(Debug)]
pub struct ProjectConfig {
    pub project_name: String,
    pub crate_name: String,
    pub name_pascal: String,
    pub description: String,
    pub template: Template,
    pub license: License,
    pub edition: Edition,
    pub ci: bool,
    pub hooks: bool,
    pub git: bool,
    pub author_name: String,
    pub author_email: String,
    pub year: i64,
}

impl ProjectConfig {
    /// Build a config from CLI args, prompting interactively for anything the
    /// user did not supply (unless `--yes` was passed).
    pub fn resolve(cli: &Cli) -> Result<Self> {
        let interactive = !cli.yes && std::io::IsTerminal::is_terminal(&std::io::stdin());

        let project_name = match &cli.name {
            Some(n) => n.clone(),
            None if interactive => prompt_name()?,
            None => bail!(
                "a project name is required (pass it as the first argument, or run interactively)"
            ),
        };
        validate_name(&project_name)?;

        let template = pick(
            cli.template,
            interactive,
            Template::Cli,
            "What kind of project?",
            &Template::ALL,
            |t| format!("{:<9} — {}", t.slug(), t.blurb()),
        )?;

        let description = match &cli.description {
            Some(d) => d.clone(),
            None if interactive => inquire::Text::new("Description:")
                .with_default(&default_description(template))
                .prompt()?,
            None => default_description(template),
        };

        let license = pick(
            cli.license,
            interactive,
            License::Dual,
            "License?",
            &[License::Dual, License::Mit, License::Apache, License::None],
            |l| match l {
                License::Dual => "MIT OR Apache-2.0 (recommended)".to_string(),
                License::Mit => "MIT".to_string(),
                License::Apache => "Apache-2.0".to_string(),
                License::None => "none".to_string(),
            },
        )?;

        let edition = pick(
            cli.edition,
            interactive,
            Edition::E2024,
            "Rust edition?",
            &[Edition::E2024, Edition::E2021],
            |e| e.as_str().to_string(),
        )?;

        let ci = resolve_bool(
            cli.ci,
            cli.no_ci,
            interactive,
            true,
            "Add GitHub Actions CI?",
        )?;
        let hooks = resolve_bool(
            cli.hooks,
            cli.no_hooks,
            interactive,
            true,
            "Install git pre-commit hooks (fmt, clippy, check)?",
        )?;

        let git = !cli.no_git;
        let (author_name, author_email) = git_author();

        Ok(Self {
            crate_name: to_crate_name(&project_name),
            name_pascal: to_pascal(&project_name),
            project_name,
            description,
            template,
            license,
            edition,
            ci,
            hooks: hooks && git, // hooks require a git repo
            git,
            author_name,
            author_email,
            year: crate::names::current_year(),
        })
    }
}

fn prompt_name() -> Result<String> {
    Ok(inquire::Text::new("Project name:")
        .with_help_message("letters, digits, '-' and '_'")
        .prompt()?)
}

fn default_description(t: Template) -> String {
    format!("A Rust {} project.", t.slug())
}

/// Pick a value: use the CLI flag if present, prompt if interactive, else default.
fn pick<T: Copy>(
    flag: Option<T>,
    interactive: bool,
    default: T,
    prompt: &str,
    options: &[T],
    label: impl Fn(&T) -> String,
) -> Result<T> {
    if let Some(v) = flag {
        return Ok(v);
    }
    if !interactive {
        return Ok(default);
    }
    // Caller orders `options` so the recommended default is first.
    let _ = default;
    let labels: Vec<String> = options.iter().map(&label).collect();
    let choice = inquire::Select::new(prompt, labels).raw_prompt()?;
    Ok(options[choice.index])
}

fn resolve_bool(
    yes: bool,
    no: bool,
    interactive: bool,
    default: bool,
    prompt: &str,
) -> Result<bool> {
    if yes {
        return Ok(true);
    }
    if no {
        return Ok(false);
    }
    if !interactive {
        return Ok(default);
    }
    Ok(inquire::Confirm::new(prompt)
        .with_default(default)
        .prompt()?)
}

fn validate_name(name: &str) -> Result<()> {
    if name.is_empty() {
        bail!("project name must not be empty");
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        bail!("project name may only contain ASCII letters, digits, '-' and '_'");
    }
    if name.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        bail!("project name must not start with a digit");
    }
    Ok(())
}

fn to_crate_name(name: &str) -> String {
    name.replace('-', "_").to_ascii_lowercase()
}

fn to_pascal(name: &str) -> String {
    name.split(['-', '_'])
        .filter(|s| !s.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect()
}

/// Best-effort author identity from `git config`.
fn git_author() -> (String, String) {
    let read = |key: &str| -> Option<String> {
        let out = std::process::Command::new("git")
            .args(["config", key])
            .output()
            .ok()?;
        if !out.status.success() {
            return None;
        }
        let v = String::from_utf8(out.stdout).ok()?.trim().to_string();
        (!v.is_empty()).then_some(v)
    };
    (
        read("user.name").unwrap_or_else(|| "Your Name".to_string()),
        read("user.email").unwrap_or_else(|| "you@example.com".to_string()),
    )
}

pub fn ensure_target_dir(name: &str) -> Result<std::path::PathBuf> {
    let dir = std::path::PathBuf::from(name);
    if dir.exists() {
        bail!(
            "destination '{}' already exists — choose another name or remove it",
            dir.display()
        );
    }
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("creating project directory '{}'", dir.display()))?;
    Ok(dir)
}
