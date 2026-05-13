# Docs Grammars

Aurora docs use Shiki, which consumes TextMate grammars for syntax
highlighting.

- `aurora.tmLanguage.json` is the Aurora schema language grammar maintained in
  this repository.
- `surrealql.tmLanguage.json` is vendored from the official SurrealDB
  `surrealdb/surrealql-grammar` repository and is used for SurrealQL injection
  inside Aurora `#s` and `#surql` escape hatches.
