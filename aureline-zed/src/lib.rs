use zed_extension_api::{
    self as zed, Command, Extension, LanguageServerId, LanguageServerInstallationStatus, Result,
    Worktree,
};

const LSP_BINARY: &str = "aureline-lsp";
const LSP_CRATE: &str = "aureline-lsp";

struct AurelineExtension;

impl Extension for AurelineExtension {
    fn new() -> Self {
        AurelineExtension
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &Worktree,
    ) -> Result<Command> {
        let existing_path = worktree.which(LSP_BINARY);

        if existing_path.is_some() {
            // Keep startup useful even when Cargo is unavailable: a discovered
            // LSP binary wins, and update checks are best-effort.
            let _ = ensure_latest_lsp(language_server_id, worktree, true);
        } else {
            ensure_latest_lsp(language_server_id, worktree, false)?;
        }

        let path = worktree
            .which(LSP_BINARY)
            .or(existing_path)
            .or_else(|| default_cargo_bin_path(worktree));

        let Some(path) = path else {
            zed::set_language_server_installation_status(
                language_server_id,
                &LanguageServerInstallationStatus::Failed(format!(
                    "{LSP_BINARY} was not found and Cargo did not report an install location"
                )),
            );
            return Err(format!(
                "{LSP_BINARY} was not found. Install it with `cargo install {LSP_CRATE}`."
            ));
        };

        zed::set_language_server_installation_status(
            language_server_id,
            &LanguageServerInstallationStatus::None,
        );

        Ok(Command {
            command: path,
            args: vec![],
            env: vec![],
        })
    }
}

fn ensure_latest_lsp(
    language_server_id: &LanguageServerId,
    worktree: &Worktree,
    best_effort: bool,
) -> Result<()> {
    let Some(cargo) = worktree.which("cargo") else {
        if best_effort {
            return Ok(());
        }

        let message = format!(
            "{LSP_BINARY} is not installed and `cargo` was not found on PATH. Install Rust/Cargo, then run `cargo install {LSP_CRATE}`."
        );
        zed::set_language_server_installation_status(
            language_server_id,
            &LanguageServerInstallationStatus::Failed(message.clone()),
        );
        return Err(message);
    };

    let status = if best_effort {
        LanguageServerInstallationStatus::CheckingForUpdate
    } else {
        LanguageServerInstallationStatus::Downloading
    };
    zed::set_language_server_installation_status(language_server_id, &status);

    let mut command = zed::process::Command::new(cargo).args(["install", LSP_CRATE]);
    let output = command.output()?;

    if output.status == Some(0) {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let details = if stderr.is_empty() { stdout } else { stderr };
    let message = if details.is_empty() {
        format!("failed to install/update {LSP_CRATE} with Cargo")
    } else {
        format!("failed to install/update {LSP_CRATE} with Cargo: {details}")
    };

    zed::set_language_server_installation_status(
        language_server_id,
        &LanguageServerInstallationStatus::Failed(message.clone()),
    );

    if best_effort {
        Ok(())
    } else {
        Err(message)
    }
}

fn default_cargo_bin_path(worktree: &Worktree) -> Option<String> {
    let env = worktree.shell_env();
    let cargo_home = env_value(&env, "CARGO_HOME").or_else(|| {
        env_value(&env, "HOME").map(|home| format!("{home}/.cargo"))
    })?;

    let binary = if matches!(zed::current_platform().0, zed::Os::Windows) {
        "aureline-lsp.exe"
    } else {
        LSP_BINARY
    };

    Some(format!("{cargo_home}/bin/{binary}"))
}

fn env_value(env: &[(String, String)], key: &str) -> Option<String> {
    env.iter()
        .find_map(|(name, value)| (name == key).then(|| value.clone()))
}

zed::register_extension!(AurelineExtension);
