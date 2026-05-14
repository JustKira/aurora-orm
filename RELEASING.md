# Releasing Aureline

Aureline uses one synchronized version across the Rust workspace and npm package
manifests managed by release-please. The current beta line starts at
`0.1.0-dev.1`.

## Release Flow

Releases are managed by release-please, not by custom shell scripts.

On pushes to `main`, `.github/workflows/release-please.yml` runs on a
Blacksmith GitHub Actions runner. It reads Conventional Commits, opens or
updates a release PR, and updates:

- `[workspace.package].version` in `Cargo.toml`
- workspace dependency pins in `Cargo.toml`
- `Cargo.lock`
- `.release-please-manifest.json`
- `package.json`
- `aureline-tree-sitter/package.json`
- `CHANGELOG.md` once release-please creates it

Merging the release PR creates the GitHub release and semantic version tag.
Normal feature/fix PRs should not edit versions directly unless intentionally
bootstrapping or repairing release state.

If future package publishing is split into separate workflows triggered by
release or tag events, release-please must use a GitHub App token or PAT instead
of the default `GITHUB_TOKEN`. GitHub does not trigger follow-up workflows from
events created with the default workflow token. The alternative is to publish
Cargo/npm/PyPI packages in guarded jobs inside the same release-please workflow.

## Pushes To Main

Pushing to `main` without changing versions does not publish packages. It runs
CI and release-please. If releasable Conventional Commits exist, release-please
opens or updates a release PR. If there are no releasable commits, nothing is
released.

## Publishing Boundaries

Package-manager publishing is intentionally not enabled yet. The repo does not
currently have finalized Cargo/npm/PyPI publish metadata or registry secrets,
and the workflows do not call `cargo publish`, `npm publish`, or PyPI upload
actions. No Cargo, npm, or PyPI token is required for the current pipeline.

The intended public Cargo surface is currently the CLI first, with codegen as a
possible later package. `aureline-core` is explicitly private for now. It is the
engine API for future extension/internal use, not a supported public crate.

Today `aureline-cli` still depends on internal workspace crates
(`aureline-core`, `aureline-migrate`, and `aureline-config`). Cargo cannot publish a
crate to crates.io with unpublished registry dependencies. Before enabling
Cargo publishing for CLI-only distribution, either fold those internals behind
the CLI package boundary or intentionally publish the dependency crates under a
documented support policy.

`aureline-lsp` can be added later if we want to distribute it separately.
`aureline-zed` should not be published to Cargo; it follows Zed's extension
publishing flow. `aureline-tree-sitter` can be tested in CI and versioned with
the repo, but npm publishing should only be enabled when the grammar package is
ready to be public.

SurrealDB's release workflow is a useful reference for a later stage: it uses
scheduled nightlies, manual versioned releases, temporary release branches,
custom version bump scripts, release-plz for crates.io, binary publishing, and
Docker publishing. Aureline should not copy that whole system yet. Keep
release-please as the version/changelog gate until the package surface and
nightly release rules are stable, then add custom release jobs only where the
standard tools stop fitting.

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
