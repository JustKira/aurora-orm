use super::context::SyntaxContext;
use super::keywords::{DOC_COMMENT, TOP_LEVEL_DECLARATIONS};

pub(super) struct SyntaxDiagnosticInput<'a> {
    pub error: &'a pest::error::Error<crate::grammar::Rule>,
    pub context: SyntaxContext,
}

pub(super) struct ClassifiedSyntaxDiagnostic {
    pub message: String,
    pub highlight_line_end: bool,
}

trait SyntaxDiagnosticClassifier {
    fn classify(&self, input: &SyntaxDiagnosticInput<'_>) -> Option<ClassifiedSyntaxDiagnostic>;
}

pub(super) fn classify_syntax_diagnostic(
    input: &SyntaxDiagnosticInput<'_>,
) -> Option<ClassifiedSyntaxDiagnostic> {
    MissingBlockStartClassifier
        .classify(input)
        .or_else(|| UnknownTopLevelDeclarationClassifier.classify(input))
}

struct MissingBlockStartClassifier;

impl SyntaxDiagnosticClassifier for MissingBlockStartClassifier {
    fn classify(&self, input: &SyntaxDiagnosticInput<'_>) -> Option<ClassifiedSyntaxDiagnostic> {
        let block_kind = input.context.missing_block_start()?;
        Some(ClassifiedSyntaxDiagnostic {
            message: format!("expected `{{` to start {} body", block_kind.label()),
            highlight_line_end: true,
        })
    }
}

struct UnknownTopLevelDeclarationClassifier;

impl SyntaxDiagnosticClassifier for UnknownTopLevelDeclarationClassifier {
    fn classify(&self, input: &SyntaxDiagnosticInput<'_>) -> Option<ClassifiedSyntaxDiagnostic> {
        let token = input.error.line().trim_start().split_whitespace().next()?;
        if token.is_empty()
            || token.starts_with(DOC_COMMENT)
            || TOP_LEVEL_DECLARATIONS.contains(&token)
        {
            return None;
        }

        closest_keyword(token, TOP_LEVEL_DECLARATIONS).map(|keyword| ClassifiedSyntaxDiagnostic {
            message: format!("unknown top-level declaration `{token}`; did you mean `{keyword}`?"),
            highlight_line_end: false,
        })
    }
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
