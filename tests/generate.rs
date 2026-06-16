//! Generation tests — the safety net behind the project's promise that every
//! template "builds, lints, and tests green from the very first commit".
//!
//! They drive the real binary (`CARGO_BIN_EXE_cargo-quickstart`) exactly as a
//! user would, then inspect the output. The fast tests are hermetic (no network,
//! no compilation) and catch the common template regressions: an unrendered
//! placeholder, a missing file, a wrong crate name. The `#[ignore]`d test
//! actually compiles a couple of templates and is meant for CI
//! (`cargo test -- --include-ignored`).

use std::path::{Path, PathBuf};
use std::process::Command;

/// Every selectable template slug. Keep in sync with `cli::Template::ALL`.
const SLUGS: &[&str] = &[
    "cli",
    "lib",
    "wasm",
    "axum",
    "embedded",
    "ios",
    "microservice",
];

/// Files the shared layer gives every project, regardless of template.
const SHARED_FILES: &[&str] = &[
    "Cargo.toml",
    "README.md",
    ".gitignore",
    "rustfmt.toml",
    "rust-toolchain.toml",
];

/// Run the binary to scaffold `demo-app` of the given template inside `dir`.
fn generate(dir: &Path, slug: &str) {
    let status = Command::new(env!("CARGO_BIN_EXE_cargo-quickstart"))
        .args(["demo-app", "--template", slug, "--yes", "--no-git"])
        .current_dir(dir)
        .status()
        .expect("failed to spawn cargo-quickstart");
    assert!(status.success(), "generation failed for template '{slug}'");
}

fn collect_files(dir: &Path, out: &mut Vec<(PathBuf, String)>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect_files(&path, out);
        } else if let Ok(text) = std::fs::read_to_string(&path) {
            out.push((path, text));
        }
    }
}

#[test]
fn every_template_scaffolds_a_clean_project() {
    for &slug in SLUGS {
        let tmp = tempfile::tempdir().unwrap();
        generate(tmp.path(), slug);
        let proj = tmp.path().join("demo-app");

        // The shared scaffolding is always present.
        for f in SHARED_FILES {
            assert!(proj.join(f).is_file(), "{slug}: missing '{f}'");
        }

        // The Cargo.toml carries the derived crate name (demo-app -> demo_app).
        let cargo = std::fs::read_to_string(proj.join("Cargo.toml")).unwrap();
        assert!(
            cargo.contains("name = \"demo_app\""),
            "{slug}: Cargo.toml is missing the derived crate name"
        );

        // CI is on by default, so the workflow should exist and be rendered.
        assert!(
            proj.join(".github/workflows/ci.yml").is_file(),
            "{slug}: CI workflow was not generated"
        );

        // Nothing should still contain a minijinja delimiter — that would mean a
        // placeholder failed to render. `[[ ` / `[%` / `[#` are the variable,
        // block, and comment openers; none are valid in finished output.
        let mut files = Vec::new();
        collect_files(&proj, &mut files);
        for (path, text) in &files {
            for opener in ["[[ ", "[%", "[#"] {
                assert!(
                    !text.contains(opener),
                    "{slug}: unrendered placeholder {opener:?} left in {}",
                    path.strip_prefix(tmp.path()).unwrap().display()
                );
            }
        }

        // Generated Rust must already be rustfmt-clean: the generated CI runs
        // `cargo fmt --check`, so any drift here breaks a fresh project's first
        // build. rustfmt is hermetic (no network), so we can check it cheaply.
        let edition = edition_of(&cargo);
        for (path, _) in &files {
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let out = Command::new("rustfmt")
                .args(["--edition", &edition, "--check"])
                .arg(path)
                .output()
                .expect("failed to spawn rustfmt");
            assert!(
                out.status.success(),
                "{slug}: rustfmt (edition {edition}) would reformat {}:\n{}",
                path.strip_prefix(tmp.path()).unwrap().display(),
                String::from_utf8_lossy(&out.stdout)
            );
        }
    }
}

/// Read the `edition = "…"` value from a generated Cargo.toml.
fn edition_of(cargo_toml: &str) -> String {
    cargo_toml
        .lines()
        .find_map(|l| l.trim().strip_prefix("edition = "))
        .map(|v| v.trim().trim_matches('"').to_string())
        .unwrap_or_else(|| "2021".to_string())
}

#[test]
fn refuses_to_overwrite_an_existing_directory() {
    let tmp = tempfile::tempdir().unwrap();
    generate(tmp.path(), "lib");
    // Second run with the same name must fail rather than clobber.
    let status = Command::new(env!("CARGO_BIN_EXE_cargo-quickstart"))
        .args(["demo-app", "--template", "lib", "--yes", "--no-git"])
        .current_dir(tmp.path())
        .status()
        .expect("failed to spawn cargo-quickstart");
    assert!(
        !status.success(),
        "generating into an existing directory should fail"
    );
}

/// Heavier check: a couple of templates must actually compile. Ignored by
/// default (needs a toolchain + network); CI runs it with --include-ignored.
#[test]
#[ignore = "compiles generated projects; run in CI with `cargo test -- --include-ignored`"]
fn generated_projects_compile() {
    for &slug in &["lib", "cli"] {
        let tmp = tempfile::tempdir().unwrap();
        generate(tmp.path(), slug);
        let proj = tmp.path().join("demo-app");
        let status = Command::new(env!("CARGO"))
            .args(["check", "--all-targets"])
            .current_dir(&proj)
            .status()
            .expect("failed to spawn cargo");
        assert!(
            status.success(),
            "{slug}: generated project failed to compile"
        );
    }
}
