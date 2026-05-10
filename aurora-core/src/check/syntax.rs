use pest::error::ErrorVariant;

use super::classifiers::{
    ClassifiedSyntaxDiagnostic, SyntaxDiagnosticInput, classify_syntax_diagnostic,
};
use super::context::SyntaxContext;
use super::diagnostics::{Diagnostic, DiagnosticCode};
use super::range::pest_error_range;
use super::rules::rule_diagnostic;

pub(crate) fn parse_diagnostic_from_pest(
    error: pest::error::Error<crate::grammar::Rule>,
) -> Diagnostic {
    let context = SyntaxContext::from_error(&error);
    let input = SyntaxDiagnosticInput {
        error: &error,
        context,
    };
    let classified = classify_syntax_diagnostic(&input)
        .unwrap_or_else(|| default_syntax_diagnostic(&error, context));
    let range = pest_error_range(&error, classified.highlight_line_end);

    Diagnostic::error(DiagnosticCode::ParseError, classified.message, range)
}

fn default_syntax_diagnostic(
    error: &pest::error::Error<crate::grammar::Rule>,
    context: SyntaxContext,
) -> ClassifiedSyntaxDiagnostic {
    match &error.variant {
        ErrorVariant::ParsingError {
            positives,
            negatives,
        } => ClassifiedSyntaxDiagnostic {
            message: parse_error_message(positives, negatives, context),
            highlight_line_end: false,
        },
        ErrorVariant::CustomError { message } => ClassifiedSyntaxDiagnostic {
            message: message.clone(),
            highlight_line_end: false,
        },
    }
}

fn parse_error_message(
    positives: &[crate::grammar::Rule],
    negatives: &[crate::grammar::Rule],
    context: SyntaxContext,
) -> String {
    let mut parts = vec![expected_unexpected_message(positives, negatives, context)];
    for detail in rule_details(positives, context)
        .into_iter()
        .chain(rule_details(negatives, context))
    {
        parts.push(detail.to_string());
    }
    parts.join(". ")
}

// Pest reports expected rules as `positives` and rejected rules as `negatives`.
// This turns that parser-oriented shape into one readable sentence.
fn expected_unexpected_message(
    positives: &[crate::grammar::Rule],
    negatives: &[crate::grammar::Rule],
    context: SyntaxContext,
) -> String {
    match (negatives.is_empty(), positives.is_empty()) {
        (false, false) => format!(
            "unexpected {}; expected {}",
            enumerate_rules(negatives, context),
            enumerate_rules(positives, context)
        ),
        (false, true) => format!("unexpected {}", enumerate_rules(negatives, context)),
        (true, false) => format!("expected {}", enumerate_rules(positives, context)),
        (true, true) => "unknown parsing error".to_string(),
    }
}

// Renders rule labels in prose: `a`, `a or b`, or `a, b, or c`.
fn enumerate_rules(rules: &[crate::grammar::Rule], context: SyntaxContext) -> String {
    let labels = rules
        .iter()
        .map(|rule| rule_diagnostic(*rule, context).label)
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

// Adds one explanatory sentence per involved grammar rule, de-duplicated.
fn rule_details(rules: &[crate::grammar::Rule], context: SyntaxContext) -> Vec<&'static str> {
    let mut details = Vec::new();
    for rule in rules {
        let Some(detail) = rule_diagnostic(*rule, context).detail else {
            continue;
        };
        if !details.contains(&detail) {
            details.push(detail);
        }
    }
    details
}
