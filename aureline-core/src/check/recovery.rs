use pest::iterators::{Pair, Pairs};

use super::diagnostics::{Diagnostic, DiagnosticCode, SourcePosition, SourceRange};
use super::keywords::TOP_LEVEL_DECLARATIONS;

const INVALID_SOURCE_ITEM_TAG: &str = "invalid_source_item";

pub(super) fn recovery_diagnostics(pairs: Pairs<'_, crate::grammar::Rule>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for pair in pairs {
        collect_recovery_diagnostics(pair, &mut diagnostics);
    }
    diagnostics
}

fn collect_recovery_diagnostics(
    pair: Pair<'_, crate::grammar::Rule>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if pair.as_node_tag() == Some(INVALID_SOURCE_ITEM_TAG) {
        diagnostics.push(invalid_source_item_diagnostic(&pair));
    }

    for child in pair.into_inner() {
        collect_recovery_diagnostics(child, diagnostics);
    }
}

fn invalid_source_item_diagnostic(pair: &Pair<'_, crate::grammar::Rule>) -> Diagnostic {
    Diagnostic::error(
        DiagnosticCode::ParseError,
        invalid_source_item_message(pair.as_str()),
        pair_range(pair),
    )
}

fn invalid_source_item_message(source: &str) -> String {
    let token = source.split_whitespace().next().unwrap_or("");
    if let Some(keyword) = closest_source_item_keyword(token) {
        return format!("unknown source item `{token}`; did you mean `{keyword}`?");
    }

    "invalid source item".to_string()
}

fn closest_source_item_keyword(token: &str) -> Option<&'static str> {
    // This recovery path is only for lines that did not start with a known
    // declaration. Keep this guard anyway so a grammar regression cannot emit
    // nonsense like `table`; did you mean `table`?
    TOP_LEVEL_DECLARATIONS
        .iter()
        .copied()
        .filter_map(|keyword| {
            let distance = levenshtein(token, keyword);
            (distance > 0 && distance <= 2).then_some((keyword, distance))
        })
        .min_by_key(|(_, distance)| *distance)
        .map(|(keyword, _)| keyword)
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

fn pair_range(pair: &Pair<'_, crate::grammar::Rule>) -> SourceRange {
    let span = pair.as_span();
    let (start_line, start_col) = span.start_pos().line_col();
    let (end_line, end_col) = span.end_pos().line_col();
    SourceRange {
        start: SourcePosition {
            line: start_line.saturating_sub(1) as u32,
            character: start_col.saturating_sub(1) as u32,
        },
        end: SourcePosition {
            line: end_line.saturating_sub(1) as u32,
            character: end_col.saturating_sub(1) as u32,
        },
    }
}
