use crate::validate::ValidationError;
use pest::error::{ErrorVariant, LineColLocation};

#[derive(Debug, thiserror::Error)]
pub enum AuroraError {
    #[error("failed to parse Aurora schema: {0}")]
    Parse(ParseDiagnostic),
    #[error("failed to convert Aurora parse tree: {0}")]
    Convert(String),
    #[error("failed to serialize Aurora AST: {0}")]
    Json(serde_json::Error),
    #[error("validation failed: {}", .0.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("; "))]
    Validation(Vec<ValidationError>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseDiagnostic {
    pub message: String,
    pub range: SourceRange,
}

impl std::fmt::Display for ParseDiagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceRange {
    pub start: SourcePosition,
    pub end: SourcePosition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourcePosition {
    pub line: u32,
    pub character: u32,
}

impl ParseDiagnostic {
    pub(crate) fn from_pest(error: pest::error::Error<crate::grammar::Rule>) -> Self {
        Self {
            message: friendly_parse_message(&error),
            range: pest_error_range(&error),
        }
    }
}

fn pest_error_range(error: &pest::error::Error<crate::grammar::Rule>) -> SourceRange {
    match error.line_col {
        LineColLocation::Pos(pos) => {
            if missing_table_body_start(error.line()) {
                end_of_line_range(error.line(), pos.0)
            } else {
                token_or_single_character_range(error.line(), pos)
            }
        }
        LineColLocation::Span(start, end) => SourceRange {
            start: to_position(start),
            end: to_position(end),
        },
    }
}

fn token_or_single_character_range(line: &str, pos: (usize, usize)) -> SourceRange {
    expand_token_range(line, pos).unwrap_or_else(|| single_character_range(pos))
}

fn expand_token_range(line: &str, pos: (usize, usize)) -> Option<SourceRange> {
    let column = pos.1.saturating_sub(1);
    let bytes = line.as_bytes();
    if column >= bytes.len() || !is_token_byte(bytes[column]) {
        return None;
    }

    let mut start = column;
    while start > 0 && is_token_byte(bytes[start - 1]) {
        start -= 1;
    }

    let mut end = column;
    while end < bytes.len() && is_token_byte(bytes[end]) {
        end += 1;
    }

    Some(SourceRange {
        start: SourcePosition {
            line: pos.0.saturating_sub(1) as u32,
            character: start as u32,
        },
        end: SourcePosition {
            line: pos.0.saturating_sub(1) as u32,
            character: end as u32,
        },
    })
}

fn is_token_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

fn single_character_range(pos: (usize, usize)) -> SourceRange {
    let start = to_position(pos);
    SourceRange {
        start,
        end: SourcePosition {
            line: start.line,
            character: start.character.saturating_add(1),
        },
    }
}

fn friendly_parse_message(error: &pest::error::Error<crate::grammar::Rule>) -> String {
    if missing_table_body_start(error.line()) {
        return "expected `{` to start table body".to_string();
    }
    if let Some(message) = unknown_top_level_declaration(error.line()) {
        return message;
    }

    match &error.variant {
        ErrorVariant::ParsingError {
            positives,
            negatives,
        } => parse_error_message(positives, negatives, error.line()),
        ErrorVariant::CustomError { message } => message.clone(),
    }
}

fn missing_table_body_start(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("table ") && !trimmed.contains('{')
}

fn unknown_top_level_declaration(line: &str) -> Option<String> {
    let token = line.trim_start().split_whitespace().next()?;
    if token.is_empty() || token.starts_with("///") || matches!(token, "table" | "analyzer") {
        return None;
    }

    closest_keyword(token, &["table", "analyzer"]).map(|keyword| {
        format!("unknown top-level declaration `{token}`; did you mean `{keyword}`?")
    })
}

fn closest_keyword<'a>(target: &str, keywords: &'a [&str]) -> Option<&'a str> {
    let mut best = None;
    for keyword in keywords {
        let distance = levenshtein(target, keyword);
        if distance <= 2 && best.map_or(true, |(_, best_distance)| distance < best_distance) {
            best = Some((*keyword, distance));
        }
    }
    best.map(|(keyword, _)| keyword)
}

fn levenshtein(a: &str, b: &str) -> usize {
    let a = a.chars().collect::<Vec<_>>();
    let b = b.chars().collect::<Vec<_>>();
    let mut prev = (0..=b.len()).collect::<Vec<_>>();
    let mut curr = vec![0; b.len() + 1];
    for (i, ac) in a.iter().enumerate() {
        curr[0] = i + 1;
        for (j, bc) in b.iter().enumerate() {
            let cost = usize::from(ac != bc);
            curr[j + 1] = (curr[j] + 1).min(prev[j + 1] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b.len()]
}

fn end_of_line_range(line: &str, line_number: usize) -> SourceRange {
    let character = line.chars().count() as u32;
    SourceRange {
        start: SourcePosition {
            line: line_number.saturating_sub(1) as u32,
            character,
        },
        end: SourcePosition {
            line: line_number.saturating_sub(1) as u32,
            character: character.saturating_add(1),
        },
    }
}

#[derive(Debug, Clone, Copy)]
struct RuleDiagnostic {
    label: &'static str,
    detail: Option<&'static str>,
}

fn parse_error_message(
    positives: &[crate::grammar::Rule],
    negatives: &[crate::grammar::Rule],
    line: &str,
) -> String {
    let mut parts = vec![expected_unexpected_message(positives, negatives, line)];
    for detail in rule_details(positives, line)
        .into_iter()
        .chain(rule_details(negatives, line))
    {
        parts.push(detail.to_string());
    }
    parts.join(". ")
}

fn expected_unexpected_message(
    positives: &[crate::grammar::Rule],
    negatives: &[crate::grammar::Rule],
    line: &str,
) -> String {
    match (negatives.is_empty(), positives.is_empty()) {
        (false, false) => format!(
            "unexpected {}; expected {}",
            enumerate_rules(negatives, line),
            enumerate_rules(positives, line)
        ),
        (false, true) => format!("unexpected {}", enumerate_rules(negatives, line)),
        (true, false) => format!("expected {}", enumerate_rules(positives, line)),
        (true, true) => "unknown parsing error".to_string(),
    }
}

fn enumerate_rules(rules: &[crate::grammar::Rule], line: &str) -> String {
    let labels = rules
        .iter()
        .map(|rule| rule_diagnostic(*rule, line).label)
        .collect::<Vec<_>>();
    match labels.as_slice() {
        [] => String::new(),
        [one] => (*one).to_string(),
        [first, second] => format!("{first} or {second}"),
        many => {
            let last = many[many.len() - 1];
            let rest = many[..many.len() - 1].join(", ");
            format!("{rest}, or {last}")
        }
    }
}

fn rule_details(rules: &[crate::grammar::Rule], line: &str) -> Vec<&'static str> {
    let mut details = Vec::new();
    for rule in rules {
        let Some(detail) = rule_diagnostic(*rule, line).detail else {
            continue;
        };
        if !details.contains(&detail) {
            details.push(detail);
        }
    }
    details
}

fn rule_diagnostic(rule: crate::grammar::Rule, line: &str) -> RuleDiagnostic {
    use crate::grammar::Rule;

    match rule {
        Rule::schema => simple("schema"),
        Rule::schema_item => simple("top-level declaration"),
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
        Rule::option_type => with_detail(
            "`option<T>`",
            "option types look like `option<T>`, for example `option<string>`",
        ),
        Rule::array_type => with_detail(
            "`array<T>`",
            "array types look like `array<T>` or `array<T, N>`, for example `array<string>` or `array<string, 10>`",
        ),
        Rule::set_type => with_detail(
            "`set<T>`",
            "set types look like `set<T>` or `set<T, N>`, for example `set<string>` or `set<string, 10>`",
        ),
        Rule::record_type => with_detail(
            "`record<T>`",
            "record types look like `record` or `record<TableName>`, for example `record<User>`",
        ),
        Rule::geometry_type => with_detail(
            "`geometry<T>`",
            "geometry types require one or more geometry feature names, for example `geometry<point>` or `geometry<point|polygon>`",
        ),
        Rule::array_length => with_detail(
            "array length like `10`",
            "array types look like `array<T>` or `array<T, N>`, and set types look like `set<T>` or `set<T, N>`, for example `array<string>` or `array<string, 10>`",
        ),
        Rule::primitive_type => simple("primitive type"),
        Rule::identifier if line.contains("geometry<") => with_detail(
            "geometry feature name",
            "geometry types require one or more geometry feature names, for example `geometry<point>` or `geometry<point|polygon>`",
        ),
        Rule::identifier => simple("identifier"),
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

fn to_position((line, column): (usize, usize)) -> SourcePosition {
    SourcePosition {
        line: line.saturating_sub(1) as u32,
        character: column.saturating_sub(1) as u32,
    }
}
