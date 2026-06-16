use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use include_dir::{include_dir, Dir, File};
use minijinja::{syntax::SyntaxConfig, Environment};
use serde::Serialize;

use crate::cli::License;
use crate::config::ProjectConfig;

static TEMPLATES: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates");

/// Template variables exposed to every `[[ ... ]]` placeholder.
#[derive(Serialize)]
struct Ctx {
    project_name: String,
    crate_name: String,
    name_pascal: String,
    description: String,
    kind: String,
    license: String,
    license_mit: bool,
    license_apache: bool,
    has_license: bool,
    edition: String,
    ci: bool,
    hooks: bool,
    author_name: String,
    author_email: String,
    year: i64,
}

impl Ctx {
    fn from(cfg: &ProjectConfig) -> Self {
        let (mit, apache) = match cfg.license {
            License::Dual => (true, true),
            License::Mit => (true, false),
            License::Apache => (false, true),
            License::None => (false, false),
        };
        Ctx {
            project_name: cfg.project_name.clone(),
            crate_name: cfg.crate_name.clone(),
            name_pascal: cfg.name_pascal.clone(),
            description: cfg.description.clone(),
            kind: cfg.template.slug().to_string(),
            license: cfg.license.spdx().to_string(),
            license_mit: mit,
            license_apache: apache,
            has_license: mit || apache,
            edition: cfg.edition.as_str().to_string(),
            ci: cfg.ci,
            hooks: cfg.hooks,
            author_name: cfg.author_name.clone(),
            author_email: cfg.author_email.clone(),
            year: cfg.year,
        }
    }
}

/// Render every template directory that applies to this config into `dest`.
pub fn render_project(cfg: &ProjectConfig, dest: &Path) -> Result<Vec<PathBuf>> {
    let env = build_env()?;
    let ctx = Ctx::from(cfg);

    let mut sources: Vec<&str> = vec!["shared", cfg.template.slug()];
    if cfg.ci {
        sources.push("ci");
    }
    if cfg.hooks {
        sources.push("hooks");
    }

    let mut written = Vec::new();
    for source in sources {
        let dir = TEMPLATES
            .get_dir(source)
            .with_context(|| format!("missing embedded template dir '{source}'"))?;
        render_dir(&env, &ctx, cfg, dir, source, dest, &mut written)?;
    }
    Ok(written)
}

fn build_env<'a>() -> Result<Environment<'a>> {
    // Use `[[ ]]`/`[% %]` delimiters so that GitHub Actions' own `${{ ... }}`
    // expressions pass through templating untouched.
    let syntax = SyntaxConfig::builder()
        .block_delimiters("[%", "%]")
        .variable_delimiters("[[", "]]")
        .comment_delimiters("[#", "#]")
        .build()
        .context("building minijinja syntax")?;
    let mut env = Environment::new();
    env.set_syntax(syntax);
    // Jinja strips a template's final newline by default; keep it so generated
    // files end in exactly one newline (rustfmt / POSIX-clean).
    env.set_keep_trailing_newline(true);
    Ok(env)
}

fn render_dir(
    env: &Environment<'_>,
    ctx: &Ctx,
    cfg: &ProjectConfig,
    dir: &Dir<'_>,
    prefix: &str,
    dest: &Path,
    written: &mut Vec<PathBuf>,
) -> Result<()> {
    let mut files = Vec::new();
    collect_files(dir, &mut files);

    for file in files {
        let rel = dest_relpath(file.path(), prefix);
        if should_skip(&rel, cfg) {
            continue;
        }

        let raw = file
            .contents_utf8()
            .with_context(|| format!("template '{}' is not UTF-8", file.path().display()))?;
        let rendered = env
            .render_str(raw, ctx)
            .with_context(|| format!("rendering template '{}'", file.path().display()))?;

        let out_path = dest.join(&rel);
        if let Some(parent) = out_path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("creating '{}'", parent.display()))?;
        }
        std::fs::write(&out_path, rendered)
            .with_context(|| format!("writing '{}'", out_path.display()))?;
        make_executable_if_hook(&rel, &out_path)?;
        written.push(rel);
    }
    Ok(())
}

fn collect_files<'a>(dir: &'a Dir<'a>, out: &mut Vec<&'a File<'a>>) {
    out.extend(dir.files());
    for sub in dir.dirs() {
        collect_files(sub, out);
    }
}

/// Convert an embedded path like `shared/dot__gitignore.jinja` into the
/// on-disk destination path `.gitignore`.
fn dest_relpath(full: &Path, prefix: &str) -> PathBuf {
    let s = full.to_string_lossy();
    let rel = s
        .strip_prefix(prefix)
        .and_then(|r| r.strip_prefix('/'))
        .unwrap_or(&s);
    let rel = rel.replace("dot__", ".");
    let rel = rel.strip_suffix(".jinja").unwrap_or(&rel);
    PathBuf::from(rel)
}

/// Drop license files that don't match the chosen license.
fn should_skip(rel: &Path, cfg: &ProjectConfig) -> bool {
    let name = rel.to_string_lossy();
    let (mit, apache) = match cfg.license {
        License::Dual => (true, true),
        License::Mit => (true, false),
        License::Apache => (false, true),
        License::None => (false, false),
    };
    if name == "LICENSE-MIT" && !mit {
        return true;
    }
    if name == "LICENSE-APACHE" && !apache {
        return true;
    }
    false
}

fn make_executable_if_hook(rel: &Path, path: &Path) -> Result<()> {
    let is_hook = rel.starts_with(".githooks");
    if !is_hook {
        return Ok(());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(path, perms)
            .with_context(|| format!("chmod +x '{}'", path.display()))?;
    }
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}
