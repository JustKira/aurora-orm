use super::context::SyntaxContext;

#[derive(Debug, Clone, Copy)]
pub(super) struct RuleDiagnostic {
    pub label: &'static str,
    pub detail: Option<&'static str>,
}

const OPTION_DETAIL: &str = "option types look like `option<T>`, for example `option<string>`";
const ARRAY_DETAIL: &str = "array types look like `array<T>` or `array<T, N>`, for example `array<string>` or `array<string, 10>`";
const SET_DETAIL: &str =
    "set types look like `set<T>` or `set<T, N>`, for example `set<string>` or `set<string, 10>`";
const RECORD_DETAIL: &str =
    "record types look like `record` or `record<TableName>`, for example `record<User>`";
const GEOMETRY_DETAIL: &str = "geometry types require one or more geometry feature names, for example `geometry<point>` or `geometry<point|polygon>`";
const ARRAY_LENGTH_DETAIL: &str = "array types look like `array<T>` or `array<T, N>`, and set types look like `set<T>` or `set<T, N>`, for example `array<string>` or `array<string, 10>`";

pub(super) fn rule_diagnostic(
    rule: crate::grammar::Rule,
    context: SyntaxContext,
) -> RuleDiagnostic {
    contextual_rule_diagnostic(rule, context).unwrap_or_else(|| default_rule_diagnostic(rule))
}

fn contextual_rule_diagnostic(
    rule: crate::grammar::Rule,
    context: SyntaxContext,
) -> Option<RuleDiagnostic> {
    use crate::grammar::Rule;

    match (rule, context) {
        (Rule::identifier, SyntaxContext::GeometryType) => {
            Some(with_detail("geometry feature name", GEOMETRY_DETAIL))
        }
        _ => None,
    }
}

fn default_rule_diagnostic(rule: crate::grammar::Rule) -> RuleDiagnostic {
    use crate::grammar::Rule;

    match rule {
        Rule::schema => simple("schema"),
        Rule::schema_item => simple("top-level declaration"),
        Rule::source_file => simple("source file"),
        Rule::source_item => simple("source item"),
        Rule::doc_comment => simple("doc comment"),
        Rule::doc_comment_line => simple("doc comment line"),
        Rule::COMMENT => simple("comment"),
        Rule::WHITESPACE => simple("whitespace"),
        Rule::analyzer_block => simple("analyzer declaration"),
        Rule::analyzer_clause => simple("analyzer clause"),
        Rule::analyzer_tokenizers => simple("`tokenizers` clause"),
        Rule::analyzer_filters => simple("`filters` clause"),
        Rule::filter_call => simple("filter call"),
        Rule::filter_arg => simple("filter argument"),
        Rule::table_block => simple("table declaration"),
        Rule::table_modifier => simple("`schemafull`, `schemaless`, or `drop`"),
        Rule::table_member => simple("field or block attribute"),
        Rule::field => simple("field"),
        Rule::type_expr => simple("type"),
        Rule::type_node => simple("type"),
        Rule::optional_marker => simple("`?`"),
        Rule::attribute => simple("field attribute"),
        Rule::block_attribute => simple("block attribute"),
        Rule::attr_call => simple("attribute arguments"),
        Rule::attr_kv_list => simple("attribute argument list"),
        Rule::attr_kv => simple("attribute argument"),
        Rule::attr_value => simple("attribute value"),
        Rule::attr_tuple => simple("tuple"),
        Rule::attr_array => simple("array"),
        Rule::attr_number => simple("number"),
        Rule::attr_bool => simple("boolean"),
        Rule::attr_ident => simple("identifier"),
        Rule::attr_string => simple("string"),
        Rule::option_type => with_detail("`option<T>`", OPTION_DETAIL),
        Rule::array_type => with_detail("`array<T>`", ARRAY_DETAIL),
        Rule::set_type => with_detail("`set<T>`", SET_DETAIL),
        Rule::record_type => with_detail("`record<T>`", RECORD_DETAIL),
        Rule::geometry_type => with_detail("`geometry<T>`", GEOMETRY_DETAIL),
        Rule::array_length => with_detail("array length like `10`", ARRAY_LENGTH_DETAIL),
        Rule::primitive_type => simple("primitive type"),
        Rule::identifier => simple("identifier"),
        Rule::INVALID_LINE => simple("invalid line"),
        Rule::block_start => simple("`{`"),
        Rule::block_end => simple("`}`"),
        Rule::type_args_start => simple("`<`"),
        Rule::type_args_end => simple("`>`"),
        Rule::EOI => simple("end of file"),
    }
}

fn simple(label: &'static str) -> RuleDiagnostic {
    RuleDiagnostic {
        label,
        detail: None,
    }
}

fn with_detail(label: &'static str, detail: &'static str) -> RuleDiagnostic {
    RuleDiagnostic {
        label,
        detail: Some(detail),
    }
}
