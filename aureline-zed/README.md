# aureline-zed

Zed editor extension for the Aureline schema language.

Provides syntax highlighting, brackets, indentation, and document outline
backed by `aureline-tree-sitter`. Wires up `aureline-lsp` from `PATH` when
available, and otherwise asks Cargo to install/update `aureline-lsp` before
starting the language server.

## Install as a dev extension

1. Make sure the WASM target Zed needs is installed:

   ```bash
   rustup target add wasm32-wasip2
   ```

   (Older Zed versions used `wasm32-wasip1`. If you see a `wasm32-wasip1`
   error instead, install that one too.)

2. Install Cargo and the LSP.

   The extension will try to run `cargo install aureline-lsp` automatically if
   `aureline-lsp` is not on `PATH`, and it performs a best-effort Cargo update
   check before startup when the binary already exists. For faster startup, you
   can install it yourself first:

   ```bash
   cargo install aureline-lsp
   ```

   When developing locally before publishing, install from this checkout instead:

   ```bash
   cargo install --path aureline-lsp
   # or symlink target/debug/aureline-lsp into ~/.local/bin
   ```

3. Two ways to install the extension:

   **UI**: open Zed's command palette (`Cmd+Shift+P`) → **"zed: install
   dev extension"** → pick `aureline-zed`.

   **CLI** (faster for iteration — Zed treats the install dir as a
   symlink target):

   ```bash
   ZED_EXT="$HOME/Library/Application Support/Zed/extensions/installed/aureline"
   rm -f "$ZED_EXT"
   ln -s "$(pwd)/aureline-zed" "$ZED_EXT"
   ```

   After that, restart Zed (or run **"zed: rebuild dev extension"** from
   the command palette).

4. Open `aureline-tree-sitter/examples/showcase.aureline`. Highlighting should turn
   on, and the language indicator in the status bar should read **Aureline**.
   If `aureline-lsp` is on `PATH`, Zed starts it directly. If not, Zed waits
   while the extension runs `cargo install aureline-lsp`, then starts the
   installed server.

## Testing local grammar changes

The committed `extension.toml` intentionally points at the GitHub repo so the
extension remains publishable. That means Zed will fetch the grammar from
`rev = "main"` and will not see unpushed local changes.

For local grammar work, switch the manifest to a local checkout:

```bash
moon run zed:setup -- --dev
```

The setup task prepares `aureline-zed/grammars/aureline` as the checkout Zed expects,
points it at this workspace, and overlays the current `aureline-tree-sitter` files
as uncommitted changes. This lets Zed rebuild against your working tree without
creating snapshot commits just to test grammar changes.

Then run **"zed: rebuild dev extension"** or restart Zed. Do not commit the
local `extension.toml` replacement or the ignored grammar checkout; restore the
remote manifest before commit:

```bash
moon run zed:setup -- --prod
```

This workflow is still a workaround. Zed requires grammar repositories to be
Git revisions, so the dev task creates the checkout shape Zed expects and keeps
the grammar edits as local working-tree changes in that checkout. It also clears
the known Aureline grammar/work caches before rebuilding. We are still looking for
a cleaner extension development experience; see
[issue #79](https://github.com/pixelscortex/aureline-orm/issues/79).

If Zed still shows old highlighting after running setup, rebuild the dev
extension again or fully restart Zed.

## Grammar reference

The `extension.toml` references the platform repo and points at the
`aureline-tree-sitter` subdirectory via the `path` field on
`[grammars.aureline]`. (That field is real but undocumented — see
`zed/crates/extension/src/extension_manifest.rs`.) Update `rev` when the
grammar changes; pin to a commit SHA for stable versions.

Shoutout to
[`surrealql-tree-sitter`](https://github.com/surrealdb/surrealql-tree-sitter),
which the extension references for SurrealQL highlighting/injection support.

## Layout

- `Cargo.toml` — WASM cdylib that gets compiled by Zed.
- `extension.toml` — manifest declaring language + grammar + LSP.
- `src/lib.rs` — extension entry point; resolves `aureline-lsp` from `PATH` or installs/updates it via Cargo.
- `languages/aureline/` — language config + tree-sitter queries:
  - `config.toml` — name, grammar, file suffixes, comment syntax.
  - `highlights.scm` — semantic highlighting.
  - `brackets.scm` — pairs for `{...}` and `<...>`.
  - `indents.scm` — indent inside table bodies.
  - `outline.scm` — outline panel entries (table definitions).

## Notes

- The cargo workspace **excludes** this crate (it's a WASM cdylib, not a
  normal binary), so `cargo build --workspace` won't try to compile it.
  Zed handles the WASM build internally when you install the extension.
- The project's `moon.yml` disables inherited Rust tasks so Moon does not
  run normal Cargo workspace builds for the Zed WASM extension. Zed's UI is
  the authoritative build path for this package.
- Path-based grammar references aren't supported by Zed's extension
  installer for non-dev installs. The `extension.toml` lists this repo
  as the grammar repository so contributors can publish later without
  changing the manifest.
