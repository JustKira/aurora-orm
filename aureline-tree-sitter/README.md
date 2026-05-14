# aureline-tree-sitter

Tree-sitter grammar for the [Aureline schema language](../aureline-core).

The grammar mirrors the pest grammar at `tools/aureline-core/src/aureline.pest` —
anything that parses there should parse here. Used by editor extensions
(`tools/aureline-zed`, future Neovim/Helix bindings) for syntax highlighting,
folding, and outline.

## Develop

```bash
# from the repository root
bun install                # one-time, installs tree-sitter-cli
moon run tree-sitter:generate   # regenerate src/parser.c from grammar.js
moon run tree-sitter:test       # run the corpus tests
moon run tree-sitter:parse -- examples/showcase.aureline
```

The generated `src/parser.c` and `src/grammar.json` are committed so
consumers don't need `tree-sitter-cli` installed. Re-run
`moon run tree-sitter:generate` whenever you change `grammar.js`.

## Layout

- `grammar.js` — grammar source (the only file you should hand-edit).
- `src/` — generated C parser. Don't edit by hand; regenerate.
- `queries/highlights.scm` — syntax highlighting query.
- `corpus/` — `tree-sitter test` fixtures.
