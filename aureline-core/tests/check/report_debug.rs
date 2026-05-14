use aureline_core::check;

#[test]
fn diagnostic_output_is_easy_to_inspect() {
    let report = check(
        r#"
table user {
  name string
}

tabl post schemafull
"#,
    );

    for diagnostic in &report.diagnostics {
        eprintln!("{diagnostic:#?}");
    }

    assert!(report.has_errors());
    assert_eq!(report.diagnostics.len(), 1);
}
