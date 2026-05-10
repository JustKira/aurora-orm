# aurora-tree-sitter

Tree-sitter grammar for the [Aurora schema language](../aurora-core).

The grammar mirrors the pest grammar at `tools/aurora-core/src/aurora.pest` —
anything that parses there should parse here. Used by editor extensions
(`tools/aurora-zed`, future Neovim/Helix bindings) for syntax highlighting,
folding, and outline.

## Develop

```bash
# from the repository root
bun install                # one-time, installs tree-sitter-cli
bun run tree-sitter:generate   # regenerate src/parser.c from grammar.js
bun run tree-sitter:test       # run the corpus tests
bun run tree-sitter:parse -- aurora-tree-sitter/examples/showcase.aurora
```

The generated `src/parser.c` and `src/grammar.json` are committed so
consumers don't need `tree-sitter-cli` installed. Re-run `bun run generate`
whenever you change `grammar.js`.

## Layout

- `grammar.js` — grammar source (the only file you should hand-edit).
- `src/` — generated C parser. Don't edit by hand; regenerate.
- `queries/highlights.scm` — syntax highlighting query.
- `corpus/` — `tree-sitter test` fixtures.
