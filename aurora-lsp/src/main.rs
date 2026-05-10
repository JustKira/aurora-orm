//! Aurora language server.
//!
//! This binary speaks the Language Server Protocol over stdio and starts with
//! parse diagnostics only. Validation diagnostics, hover, go-to-definition,
//! completions, and other editor features can layer on after this minimal path.

use aurora_core::{AuroraError, ParseDiagnostic, SourcePosition, SourceRange};
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::{
    Diagnostic, DiagnosticSeverity, DidChangeTextDocumentParams, DidOpenTextDocumentParams,
    InitializeParams, InitializeResult, InitializedParams, Position, Range, ServerCapabilities,
    ServerInfo, TextDocumentSyncCapability, TextDocumentSyncKind,
};
use tower_lsp_server::{Client, LanguageServer, LspService, Server};

struct AuroraLsp {
    client: Client,
}

impl LanguageServer for AuroraLsp {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "aurora-lsp".to_string(),
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

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

impl AuroraLsp {
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
}

fn parse_diagnostics(source: &str) -> Vec<Diagnostic> {
    match aurora_core::parse_to_ast(source) {
        Ok(_) => Vec::new(),
        Err(AuroraError::Parse(error)) => vec![parse_error_diagnostic(&error)],
        Err(error) => vec![Diagnostic::new(
            first_character_range(),
            Some(DiagnosticSeverity::ERROR),
            None,
            Some("aurora".to_string()),
            error.to_string(),
            None,
            None,
        )],
    }
}

fn parse_error_diagnostic(error: &ParseDiagnostic) -> Diagnostic {
    Diagnostic::new(
        to_lsp_range(error.range),
        Some(DiagnosticSeverity::ERROR),
        None,
        Some("aurora".to_string()),
        error.message.clone(),
        None,
        None,
    )
}

fn first_character_range() -> Range {
    Range::new(Position::new(0, 0), Position::new(0, 1))
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
    let (service, socket) = LspService::new(|client| AuroraLsp { client });
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
        assert_eq!(diagnostics[0].source.as_deref(), Some("aurora"));
        assert!(!diagnostics[0].message.is_empty());
    }

    #[test]
    fn parse_diagnostics_clears_for_valid_schema() {
        let diagnostics = parse_diagnostics("table User { name string }");

        assert!(diagnostics.is_empty());
    }
}
