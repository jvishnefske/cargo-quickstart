# cargo-quickstart

A `cargo` subcommand that scaffolds **curated, best-practice Rust projects** that
build, lint, and test green from the very first commit. Pick a template, answer a
couple of prompts (or pass flags), and get a working project with CI, git hooks,
formatting, lints, and a license already wired up.

```sh
cargo install --path .          # or: cargo install cargo-quickstart
cargo quickstart                # interactive
cargo quickstart my-app -t cli  # or fully flag-driven
```

## Templates

| slug       | what you get                                             |
|------------|----------------------------------------------------------|
| `cli`      | Command-line app — `clap` derive, `anyhow`, a test       |
| `lib`      | Library crate — doctests, `missing_docs`, unit tests     |
| `wasm`     | Browser/WASM app — `Trunk`, `wasm-bindgen`, `web-sys`    |
| `axum`     | Async web service — `axum` + `tokio` + `tracing`         |
| `embedded` | `no_std` Cortex-M firmware — `cortex-m-rt`, `memory.x`    |
| `ios`      | SwiftUI iOS app + Rust core via C-ABI FFI — XCFramework, XcodeGen |
| `microservice` | 12-factor service — `axum`, SeaORM (multi-backend), transport-agnostic message bus (in-process / NATS) with pub-sub + request-response + command-reply, Dockerfile + k8s manifests |

Every generated project also gets, by default:

- **`rustfmt.toml`** and a moderate **clippy lint** profile (`unsafe_code = "forbid"` where it fits)
- **`rust-toolchain.toml`** pinned to stable, with the right cross-compile target for wasm/embedded
- **GitHub Actions CI** — fmt → clippy (`-D warnings`) → test, plus a target build for wasm/embedded
- **Git pre-commit hooks** via `core.hooksPath .githooks` running `cargo fmt --check`, `cargo clippy`, and `cargo check` — no external hook framework
- **Dual MIT/Apache-2.0 license**, a populated **README**, and a sensible **`.gitignore`**

## Usage

```text
cargo quickstart [NAME] [OPTIONS]

Arguments:
  [NAME]                 Project name (also the directory created)

Options:
  -t, --template <KIND>  cli | lib | wasm | axum | embedded | ios | microservice
      --description <S>   One-line description
      --license <L>      dual | mit | apache | none   (default: dual)
      --edition <E>      2021 | 2024                  (default: 2024)
      --ci / --no-ci     Add or skip the GitHub Actions workflow
      --hooks/--no-hooks Install or skip git pre-commit hooks
      --no-git           Skip `git init`
  -y, --yes              Accept all defaults (non-interactive / CI)
      --list             List templates and exit
```

Run with no flags in a terminal to be prompted for each choice. Pass `--yes`
(plus any overrides) for scripted, non-interactive generation.

## How it works

Templates are embedded into the binary at build time (`include_dir!`) and
rendered with [`minijinja`](https://docs.rs/minijinja). Placeholders use
`[[ var ]]` / `[% if %]` delimiters specifically so GitHub Actions' own
`${{ ... }}` expressions pass through untouched. Adding a template is just a
matter of dropping files under `templates/<slug>/` — `dot__` in a path becomes a
leading `.` on disk, and a `.jinja` suffix is stripped after rendering.

## Developing & testing

The promise that every template builds green is enforced by the test suite
(`tests/generate.rs`), which drives the real binary the way a user would:

```sh
cargo test                       # fast: scaffold every template, check for
                                 # unrendered placeholders, files, crate name
cargo test -- --include-ignored  # also compiles the `lib` + `cli` output
```

CI (`.github/workflows/ci.yml`) runs fmt, clippy, and the full suite (with
`--include-ignored`) on every push. Adding a template means dropping files under
`templates/<slug>/` and adding the slug to `cli::Template` and the `SLUGS` list
in the test — the placeholder/render checks then cover it automatically.

## License

Licensed under either of MIT or Apache-2.0 at your option.
