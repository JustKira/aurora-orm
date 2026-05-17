# Releasing Aureline

Aureline uses manual synchronized versions and GitHub tags. There is no
automatic version bumping, changelog generation, or release-please state.

## Release Flow

1. Choose the next version, for example `0.1.0-dev.N`.
2. Update every repo version that should stay synchronized:
   - `[workspace.package].version` in `Cargo.toml`
   - workspace dependency pins in `Cargo.toml`
   - each Rust package `version` in `aureline-*/Cargo.toml`
   - `package.json`
   - `aureline-tree-sitter/package.json`
3. Open and merge a normal PR with those version changes.
4. Tag the merge commit on `main`:

```bash
git switch main
git pull --ff-only origin main
git tag v0.1.0-dev.N
git push origin v0.1.0-dev.N
```

Pushing the tag runs `.github/workflows/release.yml`. Before publishing, the
release workflow verifies that the tagged commit is contained on `main` and that
the `CI` workflow completed successfully for that exact commit. It then publishes
the workspace crates to crates.io and creates a GitHub Release with generated
notes for that tag.

The release workflow checks the tag against the synchronized repo versions
before creating the GitHub Release. For example, `v0.1.0-dev.N` only releases if
the tracked Rust and package manifests also say `0.1.0-dev.N`. If the versions
were not updated first, or if main CI has not passed for the tagged commit, the
workflow fails and no release is created.

If the tag already exists and the workflow needs to be retried, run the
`Release` workflow manually from GitHub Actions and pass the existing tag name.
The crates.io publish step skips crate versions that already exist, so reruns
can recover after a partial publish.

## Version Policy

Keep one repo-wide dev version while the package surface is still unstable.
Use explicit dev increments such as `0.1.0-dev.N`.

Do not depend on Conventional Commit bump rules for now. If a future package is
published independently, split it into its own documented release process then.

## Crates.io Publishing

The release workflow requires a repository secret named
`CARGO_REGISTRY_TOKEN`. Use a crates.io token that can publish all Aureline
crates.

Crates are published in dependency order:

```text
aureline-core
aureline-config
aureline-migrate
aureline-codegen
aureline-lsp
aureline-cli
```

Publishing all workspace crates lets users install the CLI from source with
Cargo once the version is available on crates.io.

```bash
cargo install aureline-cli
```

The installed binary is named `aureline`.

Package-manager publishing outside crates.io is intentionally not enabled yet.
The workflows do not call `npm publish` or upload package artifacts. No npm or
PyPI token is required for the current release flow.

`aureline-lsp` can be added later if we want to distribute it separately.
`aureline-zed` should not be published to Cargo; it follows Zed's extension
publishing flow. `aureline-tree-sitter` is version-synced with the repo for
consistency and CI testing, but it is not currently published from CI.

## Verification

Run the same phase checks locally that CI runs on Blacksmith for normal PR
validation:

```bash
moon ci repo:fmt
moon ci repo:clippy
moon ci repo:check tree-sitter:generated-check docs:typecheck
moon ci repo:test tree-sitter:test
```

Normal PR validation does not run the release build. Main/release validation
includes the release build phase:

```bash
moon ci repo:build-release
```

Release publishing requires successful main CI validation/checking for the exact
release commit before it publishes crates or creates the GitHub Release. Manual
retries fail fast when that CI validation is missing instead of waiting on an idle
runner.
