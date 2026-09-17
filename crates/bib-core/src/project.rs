//! Project configuration `bib.toml` (spec §10). The file is hand-written and versioned with the project.

use std::path::{Path, PathBuf};

use anyhow::{Context, bail};
use serde::Deserialize;

use crate::library::LibraryRef;

pub const CONFIG_FILE: &str = "bib.toml";

/// Sections not modelled yet (`[export]`, `[diagnostics]`) are accepted and ignored.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ProjectConfig {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub documents: Vec<DocumentConfig>,
    #[serde(default)]
    pub zotero: ZoteroConfig,
    pub bibliography: Option<BibliographyConfig>,
    #[serde(default)]
    pub ai: AiConfig,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct DocumentConfig {
    pub path: PathBuf,
    pub kind: DocumentKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DocumentKind {
    Paper,
    Slides,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ZoteroConfig {
    /// Resolution order for citation keys.
    #[serde(default = "default_libraries")]
    pub libraries: Vec<String>,
}

impl Default for ZoteroConfig {
    fn default() -> Self {
        Self {
            libraries: default_libraries(),
        }
    }
}

fn default_libraries() -> Vec<String> {
    vec!["user".to_string()]
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct BibliographyConfig {
    pub path: PathBuf,
    #[serde(default = "default_managed_by")]
    pub managed_by: String,
}

fn default_managed_by() -> String {
    "zotero".to_string()
}

/// `[ai]` policy (spec §12). Parsed now; enforced once LLM features exist.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct AiConfig {
    #[serde(default)]
    pub allowed: Vec<String>,
    #[serde(default)]
    pub cloud: bool,
    #[serde(default = "default_logging")]
    pub logging: String,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            allowed: Vec::new(),
            cloud: false,
            logging: default_logging(),
        }
    }
}

fn default_logging() -> String {
    "required".to_string()
}

impl ProjectConfig {
    pub fn parse(text: &str) -> anyhow::Result<ProjectConfig> {
        let config: ProjectConfig = toml::from_str(text)?;
        if config.id.trim().is_empty() {
            bail!("`id` must not be empty");
        }
        config.libraries()?;
        Ok(config)
    }

    pub fn load(root: &Path) -> anyhow::Result<ProjectConfig> {
        let path = root.join(CONFIG_FILE);
        let text = std::fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
        Self::parse(&text).with_context(|| format!("invalid {}", path.display()))
    }

    /// Libraries in resolution order.
    pub fn libraries(&self) -> anyhow::Result<Vec<LibraryRef>> {
        let libraries = self
            .zotero
            .libraries
            .iter()
            .map(|s| s.parse())
            .collect::<Result<Vec<LibraryRef>, _>>()?;
        if libraries.is_empty() {
            bail!("[zotero] libraries must name at least one library");
        }
        Ok(libraries)
    }
}

/// Nearest directory at or above `start` that contains `bib.toml`.
pub fn find_project_root(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|dir| dir.join(CONFIG_FILE).is_file())
        .map(Path::to_path_buf)
}

/// Writes a new `bib.toml` into `dir` and returns its path. Fails if the file exists.
pub fn init_project(dir: &Path) -> anyhow::Result<PathBuf> {
    let path = dir.join(CONFIG_FILE);
    if path.exists() {
        bail!("{} already exists", path.display());
    }
    let name = dir.file_name().and_then(|n| n.to_str()).unwrap_or("project");
    let text = skeleton(&uuid::Uuid::new_v4().to_string(), name);
    std::fs::write(&path, text).with_context(|| format!("writing {}", path.display()))?;
    Ok(path)
}

fn skeleton(id: &str, name: &str) -> String {
    format!(
        r#"id = "{id}"
name = {name}

# Typst entry points; further files follow from #include.
# [[documents]]
# path = "paper.typ"
# kind = "paper"

[zotero]
# Resolution order for citation keys: "user" and/or "group:<id>".
libraries = ["user"]

# [bibliography]
# path = "references.bib"
# managed_by = "zotero"

[ai]
allowed = []
cloud = false
logging = "required"
"#,
        name = toml_string(name)
    )
}

/// A TOML basic string literal.
fn toml_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if c.is_control() => out.push_str(&format!("\\u{:04X}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPEC_EXAMPLE: &str = r#"
id = "0193f2a1-0000-4000-8000-000000000000"
name = "EHR Privacy Seminar"

[[documents]]
path = "paper.typ"
kind = "paper"

[[documents]]
path = "presentation/slides.typ"
kind = "slides"

[zotero]
libraries = ["user", "group:6573630"]

[bibliography]
path = "ehr_privacy.bib"
managed_by = "zotero"

[export]
excerpts = "notes/quote_verification.json"

[diagnostics]
unverified_quote = "error"

[ai]
allowed = ["retrieval", "verification"]
cloud = false
logging = "required"
"#;

    #[test]
    fn parses_the_spec_example() {
        let config = ProjectConfig::parse(SPEC_EXAMPLE).unwrap();
        assert_eq!(config.name, "EHR Privacy Seminar");
        assert_eq!(config.documents.len(), 2);
        assert_eq!(config.documents[1].kind, DocumentKind::Slides);
        assert_eq!(
            config.libraries().unwrap(),
            vec![LibraryRef::User, LibraryRef::Group(6573630)]
        );
        assert_eq!(config.bibliography.unwrap().path, PathBuf::from("ehr_privacy.bib"));
        assert_eq!(config.ai.allowed, ["retrieval", "verification"]);
    }

    #[test]
    fn minimal_file_uses_defaults() {
        let config = ProjectConfig::parse("id = \"x\"\nname = \"n\"\n").unwrap();
        assert_eq!(config.libraries().unwrap(), vec![LibraryRef::User]);
        assert_eq!(config.bibliography, None);
        assert_eq!(config.ai, AiConfig::default());
    }

    #[test]
    fn invalid_library_is_an_error() {
        let err =
            ProjectConfig::parse("id = \"x\"\nname = \"n\"\n[zotero]\nlibraries = [\"group:abc\"]\n").unwrap_err();
        assert!(format!("{err:#}").contains("group:abc"), "{err:#}");
    }

    #[test]
    fn empty_library_list_and_empty_id_are_errors() {
        assert!(ProjectConfig::parse("id = \"x\"\nname = \"n\"\n[zotero]\nlibraries = []\n").is_err());
        assert!(ProjectConfig::parse("id = \" \"\nname = \"n\"\n").is_err());
    }

    #[test]
    fn init_writes_a_parseable_file_once() {
        let dir = tempfile::tempdir().unwrap();
        let project = dir.path().join("my \"quoted\" project");
        std::fs::create_dir(&project).unwrap();
        let path = init_project(&project).unwrap();
        let config = ProjectConfig::load(&project).unwrap();
        assert_eq!(config.name, "my \"quoted\" project");
        assert_eq!(config.id.len(), 36);
        assert_eq!(config.libraries().unwrap(), vec![LibraryRef::User]);
        assert!(path.ends_with(CONFIG_FILE));
        assert!(init_project(&project).is_err());
    }

    #[test]
    fn project_root_is_found_from_a_subdirectory() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(CONFIG_FILE), "id = \"x\"\nname = \"n\"\n").unwrap();
        let nested = dir.path().join("chapters/a");
        std::fs::create_dir_all(&nested).unwrap();
        assert_eq!(find_project_root(&nested).as_deref(), Some(dir.path()));
        let outside = tempfile::tempdir().unwrap();
        assert_eq!(find_project_root(outside.path()), None);
    }
}
