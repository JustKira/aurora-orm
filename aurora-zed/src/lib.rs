use zed_extension_api::{self as zed, Command, Extension, LanguageServerId, Result, Worktree};

struct AuroraExtension;

impl Extension for AuroraExtension {
    fn new() -> Self {
        AuroraExtension
    }

    fn language_server_command(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Command> {
        // Resolve the aurora-lsp binary from $PATH; fall back to the bare
        // name so Zed surfaces a clear error if the binary isn't installed.
        // Install with `cargo install --path aurora-lsp` from the repo.
        let path = worktree
            .which("aurora-lsp")
            .unwrap_or_else(|| "aurora-lsp".to_string());

        Ok(Command {
            command: path,
            args: vec![],
            env: vec![],
        })
    }
}

zed::register_extension!(AuroraExtension);
