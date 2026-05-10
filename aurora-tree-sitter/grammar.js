/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

// Tree-sitter grammar for the Aurora schema language. Mirrors the pest grammar
// at tools/aurora-core/src/aurora.pest — anything that parses there should
// parse here.

module.exports = grammar({
  name: "aurora",

  extras: ($) => [/\s/, $.line_comment],

  word: ($) => $.identifier,

  rules: {
    source_file: ($) => repeat($._definition),

    _definition: ($) =>
      choice($.table_definition, $.analyzer_definition, $.surql_block),

    // === Top-level: raw SurrealQL ===

    surql_block: ($) => seq("#surql", "{", repeat($._surql_chunk), "}"),

    _surql_chunk: ($) => choice($.surql_nested_block, $.surql_text),

    surql_nested_block: ($) => seq("{", repeat($._surql_chunk), "}"),

    surql_text: ($) => token.immediate(prec(1, /[^{}]+/)),

    // === Top-level: analyzer ===

    analyzer_definition: ($) =>
      seq(
        optional($.doc_comment),
        "analyzer",
        field("name", $.identifier),
        field("body", $.analyzer_body),
      ),

    analyzer_body: ($) => seq("{", repeat($._analyzer_clause), "}"),

    _analyzer_clause: ($) =>
      choice($.analyzer_tokenizers, $.analyzer_filters),

    analyzer_tokenizers: ($) =>
      seq(
        "tokenizers",
        field("name", $.identifier),
        repeat(seq(",", field("name", $.identifier))),
      ),

    analyzer_filters: ($) =>
      seq(
        "filters",
        field("call", $.filter_call),
        repeat(seq(",", field("call", $.filter_call))),
      ),

    filter_call: ($) =>
      seq(
        field("name", $.identifier),
        optional(
          seq(
            "(",
            field("arg", $.filter_arg),
            repeat(seq(",", field("arg", $.filter_arg))),
            ")",
          ),
        ),
      ),

    filter_arg: ($) => /[a-zA-Z0-9_]+/,

    // === Top-level: table ===

    table_definition: ($) =>
      seq(
        optional($.doc_comment),
        "table",
        field("name", $.identifier),
        optional(field("modifier", $.table_modifier)),
        field("body", $.table_body),
      ),

    table_modifier: ($) => choice("schemafull", "schemaless", "drop"),

    // Tables hold fields and block attributes (composite/table-level indexes).
    table_body: ($) =>
      seq("{", repeat(choice($.field_definition, $.block_attribute)), "}"),

    field_definition: ($) =>
      seq(
        field("name", $.identifier),
        field("type", $.type_expression),
        repeat(field("attribute", $.attribute)),
      ),

    type_expression: ($) =>
      seq(field("base", $.type_node), optional($.optional_marker)),

    optional_marker: ($) => "?",

    // Recursive type. Compound forms come first because their keywords
    // (option, array, set, record, geometry) would otherwise be matched by
    // primitive_type's keyword list — order matters in tree-sitter choices.
    type_node: ($) =>
      choice(
        $.option_type,
        $.array_type,
        $.set_type,
        $.record_type,
        $.geometry_type,
        $.primitive_type,
      ),

    option_type: ($) =>
      seq("option", "<", field("inner", $.type_node), ">"),

    array_type: ($) =>
      seq(
        "array",
        "<",
        field("inner", $.type_node),
        optional(seq(",", field("length", $.array_length))),
        ">",
      ),

    set_type: ($) =>
      seq(
        "set",
        "<",
        field("inner", $.type_node),
        optional(seq(",", field("length", $.array_length))),
        ">",
      ),

    record_type: ($) =>
      seq("record", optional(seq("<", field("table", $.identifier), ">"))),

    geometry_type: ($) =>
      seq(
        "geometry",
        "<",
        field("feature", $.identifier),
        repeat(seq("|", field("feature", $.identifier))),
        ">",
      ),

    primitive_type: ($) =>
      choice(
        "bool",
        "int",
        "float",
        "decimal",
        "number",
        "string",
        "datetime",
        "duration",
        "uuid",
        "bytes",
        "any",
        "regex",
        "object",
        "range",
      ),

    array_length: ($) => /\d+/,

    // === Attributes ===
    //
    // Generic shape: @ident or @@ident, optionally followed by (args). The
    // grammar doesn't know which attributes are valid where; the validator
    // (in aurora-core) is the rule book. This keeps the grammar tiny and lets
    // editors offer useful structure even for unknown / in-progress attributes.

    attribute: ($) =>
      seq(
        "@",
        field("name", alias(token.immediate(/[a-zA-Z][a-zA-Z0-9_]*/), $.identifier)),
        optional(field("args", $.attribute_args)),
      ),

    block_attribute: ($) =>
      seq(
        "@@",
        field("name", alias(token.immediate(/[a-zA-Z][a-zA-Z0-9_]*/), $.identifier)),
        optional(field("args", $.attribute_args)),
      ),

    // Top-level attribute args are keyword-only — every value has a name.
    // Mirrors SurrealDB's DDL where every HNSW/MTREE param is a `KEYWORD value`
    // pair. The single carve-out is `attribute_tuple` for kv values that mirror
    // SurrealDB's compound literals (currently just BM25(k1, b)).
    attribute_args: ($) =>
      seq(
        "(",
        optional(
          seq(
            $.attribute_kv,
            repeat(seq(",", $.attribute_kv)),
          ),
        ),
        ")",
      ),

    attribute_kv: ($) =>
      seq(
        field("key", $.identifier),
        ":",
        field("value", $._attribute_value),
      ),

    _attribute_value: ($) =>
      choice(
        $.attribute_array,
        $.attribute_tuple,
        $.attribute_number,
        $.attribute_bool,
        $.attribute_string,
        $.attribute_ident,
      ),

    // Parens-wrapped value list — mirrors SurrealDB's `BM25(1.2, 0.75)`.
    // Only appears as a kv value (e.g. `bm25: (1.2, 0.75)`).
    attribute_tuple: ($) =>
      seq(
        "(",
        optional(
          seq(
            $._attribute_value,
            repeat(seq(",", $._attribute_value)),
          ),
        ),
        ")",
      ),

    attribute_array: ($) =>
      seq(
        "[",
        optional(
          seq(
            $._attribute_value,
            repeat(seq(",", $._attribute_value)),
          ),
        ),
        "]",
      ),

    attribute_number: ($) => /\d+(\.\d+)?/,
    attribute_bool: ($) => choice("true", "false"),
    attribute_string: ($) => /"[^"]*"/,
    attribute_ident: ($) => $.identifier,

    // === Common ===

    identifier: ($) => /[a-zA-Z_][a-zA-Z0-9_]*/,

    doc_comment: ($) => prec(1, repeat1($.doc_comment_line)),
    doc_comment_line: ($) => token(prec(1, seq("///", /.*/))),
    line_comment: ($) => token(prec(-1, seq("//", /.*/))),
  },
});
