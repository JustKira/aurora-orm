# Docs Grammars

Aureline docs use Shiki, which consumes TextMate grammars for syntax
highlighting.

- `aureline.tmLanguage.json` is the Aureline schema language grammar maintained in
  this repository.
- `surrealql.tmLanguage.json` is vendored from the official SurrealDB
  `surrealdb/surrealql-grammar` repository and is used for SurrealQL injection
  inside Aureline `#s` and `#surql` escape hatches.
