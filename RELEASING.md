# Releasing Aureline

Aureline uses manual synchronized versions and GitHub tags. There is no
automatic version bumping, changelog generation, package publishing, or
release-please state.

## Release Flow

1. Choose the next version, for example `0.1.0-dev.2`.
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
git tag v0.1.0-dev.2
git push origin v0.1.0-dev.2
```

Pushing the tag runs `.github/workflows/release.yml`, which creates a GitHub
Release with generated notes for that tag.

The release workflow checks the tag against the synchronized repo versions
before creating the GitHub Release. For example, `v0.1.0-dev.2` only releases if
the tracked Rust and package manifests also say `0.1.0-dev.2`. If the versions
were not updated first, the workflow fails and no release is created.

If the tag already exists and the workflow needs to be retried, run the
`Release` workflow manually from GitHub Actions and pass the existing tag name.

## Version Policy

Keep one repo-wide dev version while the package surface is still unstable.
Use explicit dev increments such as:

```text
0.1.0-dev.1
0.1.0-dev.2
0.1.0-dev.3
```

Do not depend on Conventional Commit bump rules for now. If a future package is
published independently, split it into its own documented release process then.

## Publishing Boundaries

Package-manager publishing is intentionally not enabled yet. The workflows do
not call `cargo publish`, `npm publish`, or upload package artifacts. No Cargo,
npm, or PyPI token is required for the current release flow.

The intended public Cargo surface is currently the CLI first, with codegen as a
possible later package. `aureline-core` is explicitly private for now. It is the
engine API for future extension/internal use, not a supported public crate.

Today `aureline-cli` still depends on internal workspace crates
(`aureline-core`, `aureline-migrate`, and `aureline-config`). Cargo cannot
publish a crate to crates.io with unpublished registry dependencies. Before
enabling Cargo publishing for CLI-only distribution, either fold those internals
behind the CLI package boundary or intentionally publish the dependency crates
under a documented support policy.

`aureline-lsp` can be added later if we want to distribute it separately.
`aureline-zed` should not be published to Cargo; it follows Zed's extension
publishing flow. `aureline-tree-sitter` is version-synced with the repo for
consistency and CI testing, but it is not currently published from CI.

## Verification

Run the same checks locally that CI runs on Blacksmith:

```bash
moon ci \
  repo:fmt \
  repo:check \
  repo:clippy \
  repo:test \
  repo:build-release \
  tree-sitter:generated-check \
  tree-sitter:test
```
