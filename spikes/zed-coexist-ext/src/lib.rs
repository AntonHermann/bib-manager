use zed_extension_api::{self as zed, LanguageServerId, Result};

struct BibSpike;

impl zed::Extension for BibSpike {
    fn new() -> Self {
        BibSpike
    }

    fn language_server_command(&mut self, _id: &LanguageServerId, worktree: &zed::Worktree) -> Result<zed::Command> {
        let command = worktree
            .which("lsp-coexist")
            .ok_or_else(|| "lsp-coexist not in PATH: `cargo install --path spikes/lsp-coexist`".to_string())?;
        Ok(zed::Command {
            command,
            args: vec![],
            env: vec![],
        })
    }
}

zed::register_extension!(BibSpike);
