# aurora-zed

Zed editor extension for the Aurora schema language.

Provides syntax highlighting, brackets, indentation, and document outline
backed by `aurora-tree-sitter`. Wires up `aurora-lsp` from `aurora-lsp`
if it's installed in `PATH`.

## Install as a dev extension

1. Make sure the WASM target Zed needs is installed:

   ```bash
   rustup target add wasm32-wasip2
   ```

   (Older Zed versions used `wasm32-wasip1`. If you see a `wasm32-wasip1`
   error instead, install that one too.)

2. Build the LSP and put it on your `PATH`:

   ```bash
   cargo install --path aurora-lsp
   # or symlink target/debug/aurora-lsp into ~/.local/bin
   ```

3. Two ways to install the extension:

   **UI**: open Zed's command palette (`Cmd+Shift+P`) → **"zed: install
   dev extension"** → pick `aurora-zed`.

   **CLI** (faster for iteration — Zed treats the install dir as a
   symlink target):

   ```bash
   ZED_EXT="$HOME/Library/Application Support/Zed/extensions/installed/aurora"
   rm -f "$ZED_EXT"
   ln -s "$(pwd)/aurora-zed" "$ZED_EXT"
   ```

   After that, restart Zed (or run **"zed: rebuild dev extension"** from
   the command palette).

4. Open `aurora-tree-sitter/examples/showcase.aurora`. Highlighting should turn
   on, and the language indicator in the status bar should read **Aurora**.
   If `aurora-lsp` resolved on `PATH`, Zed shows it as a running server
   in the language-server panel.

## Testing local grammar changes

The committed `extension.toml` intentionally points at the GitHub repo so the
extension remains publishable. That means Zed will fetch the grammar from
`rev = "main"` and will not see unpushed local changes.

For local grammar work, switch the manifest to your local grammar:

```bash
moon run aurora-zed:setup -- --dev
```

Then run **"zed: rebuild dev extension"** or restart Zed. Do not commit the
local `extension.toml` replacement; restore the remote manifest before commit:

```bash
moon run aurora-zed:setup -- --prod
```

This workflow is still a workaround. Zed requires grammar repositories to be
Git revisions, so the dev task snapshots the local grammar into an ignored
temporary repository. It also clears the known Aurora grammar/work caches before
rebuilding the local snapshot. We are still looking for a cleaner extension
development experience; see [issue #79](https://github.com/JustKira/aurora-orm/issues/79).

If you want to keep a hand-edited local manifest instead, copy
`extension.local.toml.example` to `extension.local.toml`. That file is ignored
by Git.

If Zed still shows old highlighting after running setup, rebuild the dev
extension again or fully restart Zed.

## Grammar reference

The `extension.toml` references the platform repo and points at the
`aurora-tree-sitter` subdirectory via the `path` field on
`[grammars.aurora]`. (That field is real but undocumented — see
`zed/crates/extension/src/extension_manifest.rs`.) Update `rev` when the
grammar changes; pin to a commit SHA for stable versions.

## Layout

- `Cargo.toml` — WASM cdylib that gets compiled by Zed.
- `extension.toml` — manifest declaring language + grammar + LSP.
- `extension.local.toml.example` — local grammar manifest template ignored by Git after copying.
- `src/lib.rs` — extension entry point; resolves `aurora-lsp` from `PATH`.
- `languages/aurora/` — language config + tree-sitter queries:
  - `config.toml` — name, grammar, file suffixes, comment syntax.
  - `highlights.scm` — semantic highlighting.
  - `brackets.scm` — pairs for `{...}` and `<...>`.
  - `indents.scm` — indent inside table bodies.
  - `outline.scm` — outline panel entries (table definitions).

## Notes

- The cargo workspace **excludes** this crate (it's a WASM cdylib, not a
  normal binary), so `cargo build --workspace` won't try to compile it.
  Zed handles the WASM build internally when you install the extension.
- The project's `moon.yml` declares `language: unknown` so Moon doesn't
  inherit `rust.yml` tasks (which would try `cargo build --workspace`)
  or the `tag-wasm.yml` tasks (which target `wasm32-unknown-unknown` —
  Zed's extension target is different and managed by Zed's UI).
- Path-based grammar references aren't supported by Zed's extension
  installer for non-dev installs. The `extension.toml` lists this repo
  as the grammar repository so contributors can publish later without
  changing the manifest.
