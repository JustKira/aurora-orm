use aurora_core::{Diagnostic, DiagnosticCode, Severity, check};

pub(crate) fn diagnostics_for(source: &str) -> Vec<Diagnostic> {
    let report = check(source);
    eprintln!("source:\n{source}\n");
    for diagnostic in &report.diagnostics {
        eprintln!("diagnostic: {diagnostic:#?}");
    }
    report.diagnostics
}

pub(crate) struct ExpectedDiagnostic<'a> {
    pub(crate) code: DiagnosticCode,
    pub(crate) message: &'a str,
    pub(crate) start: (u32, u32),
    pub(crate) end: (u32, u32),
}

pub(crate) fn assert_single_diagnostic(
    diagnostics: &[Diagnostic],
    expected: ExpectedDiagnostic<'_>,
) {
    let diagnostic = only_diagnostic(diagnostics);
    assert_eq!(diagnostic.code, expected.code);
    assert_eq!(diagnostic.severity, Severity::Error);
    assert_eq!(diagnostic.message, expected.message);
    assert_range(diagnostic, expected.start, expected.end);
}

pub(crate) fn only_diagnostic(diagnostics: &[Diagnostic]) -> &Diagnostic {
    assert_eq!(
        diagnostics.len(),
        1,
        "expected one diagnostic, got {diagnostics:#?}"
    );
    &diagnostics[0]
}

pub(crate) fn assert_range(diagnostic: &Diagnostic, start: (u32, u32), end: (u32, u32)) {
    assert_eq!(diagnostic.range.start.line, start.0);
    assert_eq!(diagnostic.range.start.character, start.1);
    assert_eq!(diagnostic.range.end.line, end.0);
    assert_eq!(diagnostic.range.end.character, end.1);
}
