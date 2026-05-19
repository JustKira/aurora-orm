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
the `CI` workflow completed successfully for that exact commit. The workflow then
runs in this order:

1. `gate` resolves and validates the tag, release SHA, and main CI status.
2. `validate-version` checks the tag against every synchronized repo version.
3. `build-cli-assets` builds, smoke-tests, packages, installer-tests, and uploads
   one workflow artifact for each supported CLI platform.
4. `publish-crates-and-release` publishes missing crates and creates the GitHub
   Release when it does not already exist.
5. `upload-cli-assets` verifies the exact asset set and uploads it to the GitHub
   Release with `gh release upload --clobber`.

The release workflow checks the tag against the synchronized repo versions
before creating the GitHub Release. For example, `v0.1.0-dev.N` only releases if
the tracked Rust and package manifests also say `0.1.0-dev.N`. Versions that
contain a hyphen, such as `0.1.0-dev.N`, are created as GitHub prereleases so the
installer's `--pre` flag can find them. If the versions were not updated first,
or if main CI has not passed for the tagged commit, the workflow fails and no
release is created.

If the tag already exists and the workflow needs to be retried, run the
`Release` workflow manually from GitHub Actions and pass the existing tag name.
The crates.io publish step skips crate versions that already exist. If a GitHub
Release for the tag already exists, only release creation is skipped; the CLI
binary assets are still rebuilt and uploaded with `--clobber`, so a release
created from the GitHub UI can still receive or replace the generated assets.

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

## Prebuilt CLI Binary

The release workflow publishes these user-platform CLI assets in addition to the
crates.io packages:

| User platform | Rust target | Runner | Asset |
| --- | --- | --- | --- |
| Linux x64 portable | `x86_64-unknown-linux-musl` | `blacksmith-8vcpu-ubuntu-2404` | `aureline-<VERSION>-x86_64-unknown-linux-musl.tar.gz` |
| Linux x64 GNU fallback | `x86_64-unknown-linux-gnu` | `blacksmith-8vcpu-ubuntu-2404` | `aureline-<VERSION>-x86_64-unknown-linux-gnu.tar.gz` |
| Linux ARM64 portable | `aarch64-unknown-linux-musl` | `blacksmith-8vcpu-ubuntu-2404-arm` | `aureline-<VERSION>-aarch64-unknown-linux-musl.tar.gz` |
| macOS ARM64 | `aarch64-apple-darwin` | `blacksmith-6vcpu-macos-15` | `aureline-<VERSION>-aarch64-apple-darwin.tar.gz` |
| Windows x64 | `x86_64-pc-windows-msvc` | `blacksmith-8vcpu-windows-2025` | `aureline-<VERSION>-x86_64-pc-windows-msvc.zip` |

Each archive root contains exactly one executable: `aureline` for `.tar.gz`
assets and `aureline.exe` for the Windows `.zip` asset. The Unix installer
defaults to Linux x64 musl on Linux x64, Linux ARM64 musl on Linux ARM64, and
macOS ARM64 on Apple Silicon macOS. If a compatibility issue appears on a
glibc-based x64 Linux image, install the GNU fallback with
`--target x86_64-unknown-linux-gnu`.

The public Unix installer is served by the docs app from:

```bash
curl -fsSL https://aureline.pixelscortex.com/install.sh | sh
```

To install a specific dev/stable version or the newest GitHub prerelease:

```bash
curl -fsSL https://aureline.pixelscortex.com/install.sh | sh -s -- --version 0.1.0-dev.N
curl -fsSL https://aureline.pixelscortex.com/install.sh | sh -s -- --pre
```

Windows x64 uses the PowerShell installer. It installs to the user-local
`$env:LOCALAPPDATA\Programs\Aureline\bin` directory by default and does not
require administrator privileges:

```powershell
irm https://aureline.pixelscortex.com/install.ps1 | iex
powershell -NoProfile -ExecutionPolicy Bypass -Command "& ([scriptblock]::Create((irm https://aureline.pixelscortex.com/install.ps1))) -Version 0.1.0-dev.N"
powershell -NoProfile -ExecutionPolicy Bypass -Command "& ([scriptblock]::Create((irm https://aureline.pixelscortex.com/install.ps1))) -Pre"
```

The `--pre` flag only considers releases marked as GitHub prereleases. Explicit
`--version` works for any existing `v*` release tag, including dev versions that
were published as normal releases.

Release toolchain caches use `cache-base: main` and release-target caching. The
asset build matrix uses `strategy.max-parallel: 2` to control cost while cache
hit rates and compile times are measured. Review cache restore/save logs and
per-target compile times after release runs before changing runner sizes, target
cache strategy, or adding `sccache`.

Package-manager publishing outside crates.io is intentionally not enabled yet.
The workflows do not call `npm publish` or upload package artifacts. No npm or
PyPI token is required for the current release flow. Signing, macOS notarization,
OS package managers, and `sccache` are also non-goals for this release support
change.

`aureline-lsp` prebuilt binaries are tracked as follow-up work and can be added
later if we want to distribute the language server separately.
`aureline-zed` should not be published to Cargo; it follows Zed's extension
publishing flow. `aureline-tree-sitter` is version-synced with the repo for
consistency and CI testing, but it is not currently published from CI.

## Verification

Run the same phase checks locally that CI runs on Blacksmith for normal PR
validation:

```bash
moon ci :workspace-format
moon ci :workspace-lint
moon ci :workspace-check tree-sitter:generated-check docs:typecheck
moon ci :workspace-test tree-sitter:test
```

Normal PR validation does not run the release build. Main/release validation
includes the release build phase:

```bash
moon ci :build-release
```

Release publishing requires successful main CI validation/checking for the exact
release commit before it publishes crates or creates the GitHub Release. Manual
retries fail fast when that CI validation is missing instead of waiting on an idle
runner.

After a release, verify the prebuilt CLI assets:

- Manual dispatch against an existing dev tag succeeds.
- Each matrix job uploads exactly one workflow artifact.
- The final release contains exactly the five expected assets listed above.
- Linux x64 default install works in a clean container.
- Linux ARM64 default install works on an ARM runner.
- macOS ARM64 default install works on an Apple Silicon runner.
- Windows PowerShell install works on a Windows runner.
- Cache restore/save logs and compile times are reviewed.

```bash
INSTALL_DIR=/tmp/aureline-bin sh -c 'curl -fsSL https://aureline.pixelscortex.com/install.sh | sh'
/tmp/aureline-bin/aureline --help
```
