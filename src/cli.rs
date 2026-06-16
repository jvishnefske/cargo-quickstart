use clap::{Parser, ValueEnum};

/// Scaffold a curated, best-practice Rust project.
///
/// Run interactively (`cargo quickstart`) to be prompted for choices, or pass
/// flags for fully non-interactive / scripted generation.
#[derive(Debug, Parser)]
#[command(
    name = "cargo-quickstart",
    bin_name = "cargo quickstart",
    version,
    about
)]
pub struct Cli {
    /// Name of the project (also the directory that will be created).
    pub name: Option<String>,

    /// Which kind of project to scaffold.
    #[arg(short, long, value_enum)]
    pub template: Option<Template>,

    /// One-line description for Cargo.toml / README.
    #[arg(long)]
    pub description: Option<String>,

    /// License to apply.
    #[arg(long, value_enum)]
    pub license: Option<License>,

    /// Rust edition for the generated crate.
    #[arg(long, value_enum)]
    pub edition: Option<Edition>,

    /// Add a GitHub Actions CI workflow.
    #[arg(long, overrides_with = "no_ci")]
    pub ci: bool,
    /// Skip the CI workflow.
    #[arg(long = "no-ci")]
    pub no_ci: bool,

    /// Install git pre-commit hooks (fmt, clippy, check).
    #[arg(long, overrides_with = "no_hooks")]
    pub hooks: bool,
    /// Skip git hooks.
    #[arg(long = "no-hooks")]
    pub no_hooks: bool,

    /// Do not run `git init` in the new project.
    #[arg(long)]
    pub no_git: bool,

    /// Accept all defaults without prompting (great for CI / scripts).
    #[arg(short = 'y', long)]
    pub yes: bool,

    /// List the available templates and exit.
    #[arg(long)]
    pub list: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Template {
    /// Command-line application built on clap.
    Cli,
    /// Reusable library crate with doctests and CI.
    Lib,
    /// Browser/WASM app served with Trunk.
    Wasm,
    /// Async web service built on axum + tokio.
    Axum,
    /// `no_std` embedded firmware for Cortex-M.
    Embedded,
    /// SwiftUI iOS app backed by a Rust core (C-ABI FFI).
    Ios,
}

impl Template {
    pub fn slug(self) -> &'static str {
        match self {
            Template::Cli => "cli",
            Template::Lib => "lib",
            Template::Wasm => "wasm",
            Template::Axum => "axum",
            Template::Embedded => "embedded",
            Template::Ios => "ios",
        }
    }

    pub fn blurb(self) -> &'static str {
        match self {
            Template::Cli => "Command-line app (clap, anyhow)",
            Template::Lib => "Library crate (docs, tests, CI)",
            Template::Wasm => "Browser/WASM app (Trunk, wasm-bindgen)",
            Template::Axum => "Async web service (axum, tokio)",
            Template::Embedded => "Embedded firmware (no_std, Cortex-M)",
            Template::Ios => "iOS app (SwiftUI front end, Rust core)",
        }
    }

    pub const ALL: [Template; 6] = [
        Template::Cli,
        Template::Lib,
        Template::Wasm,
        Template::Axum,
        Template::Embedded,
        Template::Ios,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum License {
    /// MIT OR Apache-2.0 (the Rust community default).
    Dual,
    Mit,
    #[value(name = "apache")]
    Apache,
    /// No license file.
    None,
}

impl License {
    pub fn spdx(self) -> &'static str {
        match self {
            License::Dual => "MIT OR Apache-2.0",
            License::Mit => "MIT",
            License::Apache => "Apache-2.0",
            License::None => "",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Edition {
    #[value(name = "2021")]
    E2021,
    #[value(name = "2024")]
    E2024,
}

impl Edition {
    pub fn as_str(self) -> &'static str {
        match self {
            Edition::E2021 => "2021",
            Edition::E2024 => "2024",
        }
    }
}
