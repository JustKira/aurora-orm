//! Aurora language server — empty scaffold.
//!
//! This binary speaks the Language Server Protocol over stdio, but it does
//! nothing useful yet: it advertises empty capabilities on `initialize`,
//! accepts the standard `initialized` / `shutdown` lifecycle, and exits.
//!
//! Real features (diagnostics from `aurora-core::parse_to_ast`, hover, go-to-
//! definition for `record<table>` references, completions for primitive
//! types, etc.) will land in follow-up commits. Keep this file small until
//! then so the LSP scaffolding stays separable from the language logic.

use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::{
    InitializeParams, InitializeResult, InitializedParams, ServerInfo,
};
use tower_lsp_server::{Client, LanguageServer, LspService, Server};

struct AuroraLsp {
    #[allow(dead_code)]
    client: Client,
}

impl LanguageServer for AuroraLsp {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "aurora-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {}

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(|client| AuroraLsp { client });
    Server::new(stdin, stdout, socket).serve(service).await;
}
