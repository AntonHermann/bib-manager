//! `mutool` als externer Prozess (optional). mutool ist AGPL-lizenziert und wird
//! deshalb nie gelinkt, nur aufgerufen.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::backends::stext::parse_stext;
use crate::{Backend, ExtractError, Extraction};

pub struct Mutool {
    pub program: PathBuf,
}

impl Default for Mutool {
    fn default() -> Self {
        Self { program: PathBuf::from("mutool") }
    }
}

impl Mutool {
    pub fn is_available(&self) -> bool {
        Command::new(&self.program).arg("-v").output().is_ok()
    }
}

impl Backend for Mutool {
    fn name(&self) -> &'static str {
        "mutool"
    }

    fn version(&self) -> String {
        Command::new(&self.program)
            .arg("-v")
            .output()
            .ok()
            .and_then(|out| {
                let text = String::from_utf8_lossy(&out.stderr).into_owned() + &String::from_utf8_lossy(&out.stdout);
                text.split_whitespace().skip_while(|w| *w != "version").nth(1).map(str::to_string)
            })
            .unwrap_or_else(|| "unbekannt".to_string())
    }

    fn extract(&self, path: &Path) -> Result<Extraction, ExtractError> {
        let out_file = tempfile::Builder::new()
            .suffix(".xml")
            .tempfile()
            .map_err(|e| ExtractError::Unavailable(format!("Temporärdatei: {e}")))?;
        let output = Command::new(&self.program)
            .args(["draw", "-q", "-F", "stext", "-o"])
            .arg(out_file.path())
            .arg(path)
            .output()
            .map_err(|e| ExtractError::Unavailable(format!("mutool: {e}")))?;
        if !output.status.success() {
            return Err(ExtractError::Open(String::from_utf8_lossy(&output.stderr).into_owned()));
        }
        let xml = std::fs::read_to_string(out_file.path()).map_err(|e| ExtractError::Open(e.to_string()))?;
        Ok(Extraction { backend: self.name(), backend_version: self.version(), pages: parse_stext(&xml) })
    }
}
