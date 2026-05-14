use zed_extension_api::{self as zed, Command, Extension, LanguageServerId, Result, Worktree};

struct AurelineExtension;

impl Extension for AurelineExtension {
    fn new() -> Self {
        AurelineExtension
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Command> {
        // Resolve the aureline-lsp binary from $PATH; fall back to the bare
        // name so Zed surfaces a clear error if the binary isn't installed.
        // Install with `cargo install --path aureline-lsp` from the repo.
        let path = worktree
            .which("aureline-lsp")
            .unwrap_or_else(|| "aureline-lsp".to_string());

        Ok(Command {
            command: path,
            args: vec![],
            env: vec![],
        })
    }
}

zed::register_extension!(AurelineExtension);
