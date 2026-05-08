# Aurora

> ⚠️ **Highly experimental.** A lot of this code is AI-generated and needs careful human review.
> Expect rough edges, breaking changes, and incomplete features. Do not use in production yet.

A schema language for SurrealDB. Schema syntax inspired by **Prisma**; the future typed client API is inspired by **Drizzle**.

## Schema

```aurora
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

```aurora
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
import { db, user, lessonChunk } from "./aurora-generated";

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
use aurora_generated::{db, user, lesson_chunk};

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
from aurora_generated import db, user, lesson_chunk

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
| `aurora-core` | Pest parser, AST, validator, SurrealQL emitter | **Thin grammar, smart validator.** The grammar treats every `@ident(args)` as an opaque blob; the validator is the rule book that says which attributes mean what. Adding a new attribute = adding a validator arm, not changing the grammar. |
| `aurora-config` | Shared `aurora.toml` loader | Every tool reads the same config — URL, NS, DB, env path — in one place. No tool re-implements credential loading. |
| `aurora-migrate` | `diff` / `generate` / `apply` against live SurrealDB | Idempotent migrations tracked in `_aurora_migrations`. Talks to SurrealDB via the official Rust SDK, not raw HTTP. Drift detection compares snapshot checksums to refuse re-applying mutated migrations. |
| `aurora-cli` | The `aurora` binary | Thin wrapper around the libraries — one entrypoint per workflow. |
| `aurora-codegen` | Schema → typed clients (JS/TS, Rust, Python) | Single source of truth: the schema. Codegen reads the validated AST and emits a Drizzle-style fluent client per language. **Work in progress.** |
| `aurora-lsp` | Language server | Reads the *raw* AST (not the validated one), so editors can offer structure for in-progress / invalid schemas too. **Scaffolding.** |
| `aurora-tree-sitter` | Grammar + corpus tests + showcase | Editor-agnostic syntax highlighting + structure. Single source of highlight queries; per-editor extensions symlink to it. |
| `aurora-zed` | Zed editor extension | Wraps the grammar + LSP for Zed. Grammar URL points back at this repo. |

## Why

What I had before:

- Hand-written `createSchema()` TypeScript that built SurrealQL strings imperatively.
- Manual migration scripts — no diff, no rollback, no record of what was applied.
- Untyped client: `db.query("SELECT * FROM user WHERE email = $email", { email })`.

What Aurora replaces it with:

- One `schema.aurora` file. Declarative. Editor-tooled. Validated.
- `aurora migrate diff` shows what changed; `generate` writes a migration with checksums; `apply` runs it idempotently against live SurrealDB.
- Typed clients (eventually) so `db.select().from(user).where(user.email.eq(...))` is statically checked end-to-end.

## Design philosophies

**One canonical syntax per concept.** No "two ways to do X". Index args are keyword-only. Naming is always the `name:` keyword. `@@` is reserved for composites and table-level concepts only. The grammar accepts exactly one shape per declaration.

**Mirror SurrealDB, don't reinvent.** BM25 tunings are written `bm25: (1.2, 0.75)` because SurrealQL writes them `BM25(1.2, 0.75)`. Distance modes, vector types, count indexes — every Aurora concept maps directly to a SurrealDB primitive.

**Two-layer parser.** Raw AST (every attribute is a blob) for LSP/tooling — works on incomplete code. Validated AST for migrate/codegen — guaranteed-shape, ready to emit. Same parser pipeline, different consumers.

**Schema is the source of truth.** The schema produces SurrealQL DDL. The schema produces the codegen'd client. The schema produces editor highlights. Everything downstream is derived; nothing is hand-maintained twice.

## Quickstart

```bash
cargo build --workspace
cargo test -p aurora-core
cd aurora-tree-sitter && bunx tree-sitter test

# Install the CLI
cargo install --path aurora-cli
# Or from this repo
cargo install aurora-cli --git https://github.com/JustKira/aurora-orm --branch main
```

## Using Aurora in a project

```bash
# In your project root, alongside aurora.toml + schema.aurora
aurora migrate diff      # show pending changes vs. the live database
aurora migrate generate  # write a new migration file with checksums
```

See [`aurora-tree-sitter/examples/showcase.aurora`](aurora-tree-sitter/examples/showcase.aurora) for every syntax form in one place.
