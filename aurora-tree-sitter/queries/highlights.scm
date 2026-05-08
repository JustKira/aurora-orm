; Top-level keywords
"table" @keyword
"analyzer" @keyword

; Analyzer clause keywords
"tokenizers" @keyword
"filters" @keyword

; Table modifiers (schemafull / schemaless / drop)
(table_modifier) @keyword.modifier

; Compound type keywords
"option" @type.builtin
"array" @type.builtin
"set" @type.builtin
"record" @type.builtin
"geometry" @type.builtin

; Primitive types (bool, int, float, string, datetime, …)
(primitive_type) @type.builtin

; Optional marker (?)
(optional_marker) @operator

; Identifiers — context-aware
(table_definition name: (identifier) @type)
(analyzer_definition name: (identifier) @type)
(field_definition name: (identifier) @property)
(record_type table: (identifier) @type)
(geometry_type feature: (identifier) @attribute)

; Analyzer config — tokenizer names and filter calls
(analyzer_tokenizers name: (identifier) @constant.builtin)
(filter_call name: (identifier) @function)
(filter_arg) @string.special

; Attributes — mirrors the official Prisma Zed extension's choices, since the
; syntax is structurally identical. `@` / `@@` are operators, attribute names
; are `@label` (Prisma uses this so the decorator reads as a single colored
; unit), call names like `bm25(...)` are `@function`, bare-ident values are
; `@constant`, and kv keys are `@property`.
"@" @operator
"@@" @operator
(attribute name: (identifier) @label)
(block_attribute name: (identifier) @label)

(attribute_kv key: (identifier) @property)

(attribute_string) @string
(attribute_number) @number
(attribute_bool) @boolean

(attribute_ident (identifier) @constant)

; Numeric literals in type params (array<T, N>)
(array_length) @number

; Comments
(doc_comment) @comment.documentation
(line_comment) @comment

; Punctuation
"{" @punctuation.bracket
"}" @punctuation.bracket
"<" @punctuation.bracket
">" @punctuation.bracket
"(" @punctuation.bracket
")" @punctuation.bracket
"[" @punctuation.bracket
"]" @punctuation.bracket
"," @punctuation.delimiter
"|" @punctuation.delimiter
":" @punctuation.delimiter
