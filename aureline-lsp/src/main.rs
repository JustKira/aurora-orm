//! Aureline language server.
//!
//! This binary speaks the Language Server Protocol over stdio and starts with
//! parse diagnostics only. Validation diagnostics, hover, go-to-definition,
//! completions, and other editor features can layer on after this minimal path.

use aureline_core::{Diagnostic as AurelineDiagnostic, Severity, SourcePosition, SourceRange};
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::{
    Diagnostic, DiagnosticSeverity, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, InitializeParams, InitializeResult, InitializedParams,
    NumberOrString, Position, Range, ServerCapabilities, ServerInfo, TextDocumentSyncCapability,
    TextDocumentSyncKind,
};
use tower_lsp_server::{Client, LanguageServer, LspService, Server};

struct AurelineLsp {
    client: Client,
}

impl LanguageServer for AurelineLsp {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "aureline-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            offset_encoding: None,
        })
    }

    async fn initialized(&self, _: InitializedParams) {}

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.publish_parse_diagnostics(
            params.text_document.uri,
            params.text_document.version,
            &params.text_document.text,
        )
        .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let Some(change) = params.content_changes.into_iter().last() else {
            return;
        };

        self.publish_parse_diagnostics(
            params.text_document.uri,
            params.text_document.version,
            &change.text,
        )
        .await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.publish_clear_diagnostics(params.text_document.uri)
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

impl AurelineLsp {
    async fn publish_parse_diagnostics(
        &self,
        uri: tower_lsp_server::ls_types::Uri,
        version: i32,
        source: &str,
    ) {
        let diagnostics = parse_diagnostics(source);
        self.client
            .publish_diagnostics(uri, diagnostics, Some(version))
            .await;
    }

    async fn publish_clear_diagnostics(&self, uri: tower_lsp_server::ls_types::Uri) {
        self.client
            .publish_diagnostics(uri, close_diagnostics(), None)
            .await;
    }
}

fn close_diagnostics() -> Vec<Diagnostic> {
    Vec::new()
}

fn parse_diagnostics(source: &str) -> Vec<Diagnostic> {
    aureline_core::check(source)
        .diagnostics
        .iter()
        .map(to_lsp_diagnostic)
        .collect()
}

fn to_lsp_diagnostic(diagnostic: &AurelineDiagnostic) -> Diagnostic {
    let mut lsp_diagnostic = Diagnostic::new(
        to_lsp_range(diagnostic.range),
        Some(to_lsp_severity(diagnostic.severity)),
        Some(NumberOrString::String(diagnostic.code.as_str().to_string())),
        Some("aureline".to_string()),
        diagnostic.to_string(),
        None,
        None,
    );
    lsp_diagnostic.data = Some(to_lsp_diagnostic_data(diagnostic));
    lsp_diagnostic
}

fn to_lsp_diagnostic_data(diagnostic: &AurelineDiagnostic) -> serde_json::Value {
    serde_json::json!({
        "code": diagnostic.code.as_str(),
        "help": diagnostic.help,
        "data": diagnostic.data,
    })
}

fn to_lsp_severity(severity: Severity) -> DiagnosticSeverity {
    match severity {
        Severity::Error => DiagnosticSeverity::ERROR,
        Severity::Warning => DiagnosticSeverity::WARNING,
        Severity::Info => DiagnosticSeverity::INFORMATION,
        Severity::Hint => DiagnosticSeverity::HINT,
    }
}

fn to_lsp_range(range: SourceRange) -> Range {
    Range::new(to_lsp_position(range.start), to_lsp_position(range.end))
}

fn to_lsp_position(position: SourcePosition) -> Position {
    Position::new(position.line, position.character)
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(|client| AurelineLsp { client });
    Server::new(stdin, stdout, socket).serve(service).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_diagnostics_returns_one_error_for_invalid_schema() {
        let diagnostics = parse_diagnostics("table { }");

        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].severity, Some(DiagnosticSeverity::ERROR));
        assert_eq!(
            diagnostics[0].code,
            Some(NumberOrString::String("parse_error".to_string()))
        );
        assert_eq!(diagnostics[0].source.as_deref(), Some("aureline"));
        assert!(!diagnostics[0].message.is_empty());
        assert_eq!(
            diagnostics[0]
                .data
                .as_ref()
                .and_then(|data| data.get("code")),
            Some(&serde_json::json!("parse_error"))
        );
    }

    #[test]
    fn parse_diagnostics_clears_for_valid_schema() {
        let diagnostics = parse_diagnostics(
            r#"
table User {
  name string
}
"#,
        );

        assert!(diagnostics.is_empty());
    }

    #[test]
    fn close_diagnostics_clears_editor_diagnostics() {
        assert!(close_diagnostics().is_empty());
    }
}
