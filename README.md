# Aureline

> ⚠️ **Highly experimental.** A lot of this code is AI-generated and needs careful human review.
> Expect rough edges, breaking changes, and incomplete features. Do not use in production yet.

A schema language for SurrealDB.

## Heavily inspired by

- [Prisma](https://www.prisma.io) — schema syntax, the `@field` / `@@table` annotation style, the "schema is the source of truth" philosophy.
- [Drizzle](https://orm.drizzle.team) — the future typed client API: fluent, schema-driven, statically checked end-to-end.
- [surrealql-tree-sitter](https://github.com/surrealdb/surrealql-tree-sitter) — SurrealQL grammar work used by Aureline editor integrations for highlighting/injection support.

## Schema

```aureline
analyzer edu_analyzer {
  tokenizers blank, class
  filters    lowercase, snowball(english)
}

table user schemafull {
  email     string @unique
  username  string @unique(name: "user_username_idx")
  status    string @index
  created   datetime
  metadata  object @flexible
}

table lesson_chunk schemafull {
  text       string @fulltext(analyzer: edu_analyzer, bm25: (1.2, 0.75))
  embedding  array<float> @hnsw(dimension: 1536, dist: cosine, type: f32)
  metadata   object @flexible
}
```

Two annotation styles — `@field` for single-field indexes and flags, `@@table` for composites and table-level concepts:

```aureline
table membership schemafull {
  account string
  user    string
  role    string

  @@unique(fields: [account, user])
  @@index(fields: [account, role], name: "idx_role_lookup")
}
```

## Future client (Drizzle-inspired)

The codegen pipeline (work in progress) will turn the schema above into typed clients in JS/TS, Rust, and Python. Same fluent feel across languages — the schema is the single source of truth.

> None of these snippets work yet. They show the target API.

### TypeScript / JavaScript

```ts
import { db, user, lessonChunk } from "./aureline-generated";

const alice = await db
  .select()
  .from(user)
  .where(user.email.eq("alice@example.com"))
  .limit(1);

// Vector search via @hnsw
const similar = await db
  .select()
  .from(lessonChunk)
  .nearest(lessonChunk.embedding, queryVec, { limit: 10 });
```

### Rust

```rust
use aureline_generated::{db, user, lesson_chunk};

let alice = db.select()
    .from(user)
    .filter(user::email.eq("alice@example.com"))
    .first()
    .await?;

let similar = db.select()
    .from(lesson_chunk)
    .nearest(lesson_chunk::embedding, &query_vec, 10)
    .await?;
```

### Python

```python
from aureline_generated import db, user, lesson_chunk

alice = await db.select().from_(user).where(user.email == "alice@example.com").first()

similar = await (
    db.select()
      .from_(lesson_chunk)
      .nearest(lesson_chunk.embedding, query_vec, limit=10)
)
```

## Crates

| Crate | What it is | Philosophy |
|---|---|---|
| `aureline-core` | Pest parser, AST, validator, SurrealQL emitter | **Thin grammar, smart validator.** The grammar treats every `@ident(args)` as an opaque blob; the validator is the rule book that says which attributes mean what. Adding a new attribute = adding a validator arm, not changing the grammar. |
| `aureline-config` | Shared `aureline.toml` loader | Every tool reads the same config — URL, NS, DB, env path — in one place. No tool re-implements credential loading. |
| `aureline-migrate` | `diff` / `generate` / `apply` against live SurrealDB | Idempotent migrations tracked in `_aureline_migrations`. Talks to SurrealDB via the official Rust SDK, not raw HTTP. Drift detection compares snapshot checksums to refuse re-applying mutated migrations. |
| `aureline-cli` | The `aureline` binary | Thin wrapper around the libraries — one entrypoint per workflow. |
| `aureline-codegen` | Schema → intermediate JSON → wasm-plugin generators | The codegen pipeline emits a stable JSON intermediate that any wasm plugin can consume to generate a typed client. Built-in plugins planned for TS, Rust, and Python; the plugin model means new target languages (Go, Kotlin, Elixir, anything) just need a wasm plugin — not a Rust contribution to Aureline itself. **Work in progress.** |
| `aureline-lsp` | Language server | Reads the *raw* AST (not the validated one), so editors can offer structure for in-progress / invalid schemas too. **Scaffolding.** |
| `aureline-tree-sitter` | Grammar + corpus tests + showcase | Editor-agnostic syntax highlighting + structure. Single source of highlight queries; per-editor extensions symlink to it. |
| `aureline-zed` | Zed editor extension | Wraps the grammar + LSP for Zed. Grammar URL points back at this repo. |

## Why

What I had before:

- Hand-written `createSchema()` TypeScript that built SurrealQL strings imperatively.
- Manual migration scripts — no diff, no rollback, no record of what was applied.
- Untyped client: `db.query("SELECT * FROM user WHERE email = $email", { email })`.

What Aureline replaces it with:

- One `schema.aureline` file. Declarative. Editor-tooled. Validated.
- `aureline migrate diff` shows what changed; `generate` writes a migration with checksums; `apply` runs it idempotently against live SurrealDB.
- Typed clients (eventually) so `db.select().from(user).where(user.email.eq(...))` is statically checked end-to-end.

## Design philosophies

**Two-layer parser.** Raw AST (every attribute is a blob) for LSP/tooling — works on incomplete code. Validated AST for migrate/codegen — guaranteed-shape, ready to emit. Same parser pipeline, different consumers.

**Schema is the source of truth.** The schema produces SurrealQL DDL. The schema produces editor highlights. The schema produces the JSON intermediate that codegen plugins consume. Everything downstream is derived; nothing is hand-maintained twice.

**Codegen via wasm plugins, not built-in language support.** Aureline itself doesn't ship a TS generator or a Rust generator — it ships a stable JSON schema intermediate and a wasm plugin host. Any language can be supported by a wasm plugin that reads the JSON and emits target code. Built-ins planned for TS, Rust, and Python; everything else (Go, Kotlin, Elixir, internal DSLs, custom ORMs) is a plugin, not a fork.

## Quickstart

```bash
cargo build --workspace
cargo test -p aureline-core
cd aureline-tree-sitter && bunx tree-sitter test

# Install the CLI
cargo install --path aureline-cli
# Or from this repo
cargo install aureline-cli --git https://github.com/pixelscortex/aureline-orm --branch main
```

## Using Aureline in a project

```bash
# In your project root, alongside aureline.toml + schema.aureline
aureline migrate diff      # show pending changes vs. the live database
aureline migrate generate  # write a new migration file with checksums
```

See [`aureline-tree-sitter/examples`](aureline-tree-sitter/examples) for focused syntax examples.
