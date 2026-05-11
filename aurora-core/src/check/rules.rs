use super::context::SyntaxContext;

#[derive(Debug, Clone)]
pub(super) struct RuleDiagnostic {
    pub label: RuleLabel,
    pub detail: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum RuleLabel {
    Static(&'static str),
    Unhandled(String),
}

impl std::fmt::Display for RuleLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Static(label) => write!(f, "{label}"),
            Self::Unhandled(rule) => write!(f, "unhandled parser rule `{rule}`"),
        }
    }
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
    contextual_rule_diagnostic(rule, context)
        .or_else(|| curated_rule_diagnostic(rule))
        .unwrap_or_else(|| fallback_rule_diagnostic(rule))
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

fn curated_rule_diagnostic(rule: crate::grammar::Rule) -> Option<RuleDiagnostic> {
    use crate::grammar::Rule;

    Some(match rule {
        Rule::analyzer_block => simple("analyzer declaration"),
        Rule::analyzer_member => simple("analyzer clause"),
        Rule::analyzer_clause => simple("analyzer clause"),
        Rule::table_block => simple("table declaration"),
        Rule::table_modifier => simple("`schemafull`, `schemaless`, or `drop`"),
        Rule::table_member => simple("field or block attribute"),
        Rule::field => simple("field"),
        Rule::type_expr | Rule::type_node => simple("type"),
        Rule::optional_marker => simple("`?`"),
        Rule::attribute => simple("field attribute"),
        Rule::block_attribute => simple("block attribute"),
        Rule::attr_call => simple("attribute arguments"),
        Rule::attr_ident | Rule::identifier => simple("identifier"),
        Rule::option_type => with_detail("`option<T>`", OPTION_DETAIL),
        Rule::array_type => with_detail("`array<T>`", ARRAY_DETAIL),
        Rule::set_type => with_detail("`set<T>`", SET_DETAIL),
        Rule::record_type => with_detail("`record<T>`", RECORD_DETAIL),
        Rule::geometry_type => with_detail("`geometry<T>`", GEOMETRY_DETAIL),
        Rule::array_length => with_detail("array length like `10`", ARRAY_LENGTH_DETAIL),
        Rule::primitive_type => simple("primitive type"),
        Rule::BLOCK_START => simple("`{`"),
        Rule::BLOCK_END => simple("`}`"),
        Rule::TYPE_ARGS_START => simple("`<`"),
        Rule::TYPE_ARGS_END => simple("`>`"),
        Rule::INVALID_SOURCE_ITEM => simple("invalid source item"),
        Rule::EOI => simple("end of file"),
        _ => return None,
    })
}

fn fallback_rule_diagnostic(rule: crate::grammar::Rule) -> RuleDiagnostic {
    RuleDiagnostic {
        label: RuleLabel::Unhandled(format!("{rule:?}")),
        detail: None,
    }
}

fn simple(label: &'static str) -> RuleDiagnostic {
    RuleDiagnostic {
        label: RuleLabel::Static(label),
        detail: None,
    }
}

fn with_detail(label: &'static str, detail: &'static str) -> RuleDiagnostic {
    RuleDiagnostic {
        label: RuleLabel::Static(label),
        detail: Some(detail),
    }
}
