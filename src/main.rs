mod cli;
mod config;
mod generate;
mod names;

use anyhow::{Context, Result};
use clap::Parser;

use cli::{Cli, Template};
use config::ProjectConfig;

fn main() {
    if let Err(err) = run() {
        eprintln!("\x1b[31merror:\x1b[0m {err:#}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    // When invoked as `cargo quickstart ...`, cargo passes "quickstart" as the
    // first argument. Drop it so clap sees a clean argv.
    let mut args: Vec<std::ffi::OsString> = std::env::args_os().collect();
    if args.get(1).is_some_and(|a| a == "quickstart") {
        args.remove(1);
    }
    let cli = Cli::parse_from(args);

    if cli.list {
        print_templates();
        return Ok(());
    }

    let cfg = ProjectConfig::resolve(&cli)?;
    let dest = config::ensure_target_dir(&cfg.project_name)?;

    let written = generate::render_project(&cfg, &dest).inspect_err(|_| {
        // Don't leave a half-written directory behind on failure.
        let _ = std::fs::remove_dir_all(&dest);
    })?;

    if cfg.git {
        init_git(&cfg, &dest)?;
    }

    print_summary(&cfg, &dest, written.len());
    Ok(())
}

fn init_git(cfg: &ProjectConfig, dest: &std::path::Path) -> Result<()> {
    let git = |args: &[&str]| -> Result<()> {
        let status = std::process::Command::new("git")
            .args(args)
            .current_dir(dest)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .context("running git (is it installed and on PATH?)")?;
        anyhow::ensure!(status.success(), "git {:?} failed", args);
        Ok(())
    };

    git(&["init", "-q"])?;
    if cfg.hooks {
        // Point git at the committed hooks dir so the pre-commit hook is active
        // for everyone who clones the repo after they run this once.
        git(&["config", "core.hooksPath", ".githooks"])?;
    }
    Ok(())
}

fn print_templates() {
    println!("Available templates:\n");
    for t in Template::ALL {
        println!("  {:<9}  {}", t.slug(), t.blurb());
    }
    println!("\nUsage: cargo quickstart <name> --template <slug>");
}

fn print_summary(cfg: &ProjectConfig, dest: &std::path::Path, files: usize) {
    let next = match cfg.template {
        Template::Wasm => "trunk serve",
        Template::Embedded => "cargo build --release",
        Template::Axum => "cargo run",
        Template::Microservice => "cp .env.example .env && cargo run",
        Template::Ios => "cargo test && bash build-rust.sh && xcodegen generate",
        _ => "cargo run",
    };
    println!(
        "\n\x1b[32m✓\x1b[0m Created \x1b[1m{}\x1b[0m ({} template, {files} files)",
        cfg.project_name,
        cfg.template.slug()
    );
    if cfg.hooks {
        println!("  • git pre-commit hooks installed (fmt, clippy, check)");
    }
    if cfg.ci {
        println!("  • GitHub Actions CI workflow added");
    }
    println!("\nNext steps:\n  cd {}\n  {next}", dest.display());
}
