# Aurora

A schema language for SurrealDB. Inspired by Prisma, grounded by SurrealQL.

```aurora
analyzer edu_analyzer {
  tokenizers blank, class
  filters    lowercase, snowball(english)
}

table lesson_chunk schemafull {
  text       string @fulltext(analyzer: edu_analyzer, bm25: (1.2, 0.75))
  metadata   object @flexible
  embedding  array<float> @hnsw(dimension: 1536, dist: cosine, type: f32)
}
```

## Crates

| Crate | What it is |
|---|---|
| `aurora-core` | Parser (pest), AST, validator, SurrealQL emitter |
| `aurora-config` | Shared `aurora.toml` loader |
| `aurora-migrate` | `diff` / `generate` / `apply` against live SurrealDB |
| `aurora-cli` | The `aurora` binary |
| `aurora-codegen` | Schema → typed JS/TS clients (work in progress) |
| `aurora-lsp` | LSP for editor support (scaffolding) |
| `aurora-tree-sitter` | Tree-sitter grammar + queries |
| `aurora-zed` | Zed editor extension |

## Quickstart

```bash
# Build everything
cargo build --workspace

# Run aurora-core tests (parser + validator)
cargo test -p aurora-core

# Tree-sitter corpus tests
cd aurora-tree-sitter && bunx tree-sitter test

# Install the CLI
cargo install --path aurora-cli

# Or install from this repo
cargo install aurora-cli --git https://github.com/JustKira/aurora-orm --branch main
```

## Using `aurora` against your project

```bash
# In your project root, alongside aurora.toml + schema.aurora
aurora migrate diff      # show pending changes
aurora migrate generate  # create a new migration file
```

See `aurora-tree-sitter/examples/showcase.aurora` for every syntax form in one place.
