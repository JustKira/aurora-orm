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

; Primitive types
(primitive_type) @type.builtin

; Optional marker (?)
(optional_marker) @operator

; Identifiers
(table_definition name: (identifier) @type)
(analyzer_definition name: (identifier) @type)
(field_definition name: (identifier) @property)
(record_type table: (identifier) @type)
(geometry_type feature: (identifier) @attribute)

; Analyzer config
(analyzer_tokenizers name: (identifier) @constant.builtin)
(filter_call name: (identifier) @function)
(filter_arg) @string.special

; Attributes
"#" @operator
"@" @operator
"@@" @operator
(surql_block name: (identifier) @label)
(attribute name: (identifier) @label)
(block_attribute name: (identifier) @label)
(attribute (identifier) @label)
(block_attribute (identifier) @label)

(attribute_kv key: (identifier) @property)

(attribute_string) @string
(attribute_number) @number
(attribute_bool) @boolean
(attribute_ident (identifier) @constant)

; Numeric literals in type params
(array_length) @number

; Raw SurrealQL escape hatch. This is intentionally coarse until nested
; SurrealQL injection is wired up.
(surql_text) @string.special

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
