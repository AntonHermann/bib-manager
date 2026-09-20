//! Runs the real `bib` binary against a temporary data directory and the fake Zotero.

use std::path::Path;
use std::process::{Command, Output};

use bib_core::zotero::fake::{self, FakeZotero};
use serde_json::Value;

struct Env {
    data: tempfile::TempDir,
    project: tempfile::TempDir,
    zotero_url: String,
}

impl Env {
    fn new(zotero_url: &str) -> Env {
        Env {
            data: tempfile::tempdir().unwrap(),
            project: tempfile::tempdir().unwrap(),
            zotero_url: zotero_url.to_string(),
        }
    }

    fn bib(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_bib"))
            .args(args)
            .current_dir(self.project.path())
            .env("BIB_DATA_DIR", self.data.path())
            .env("BIB_ZOTERO_URL", &self.zotero_url)
            .env_remove("XDG_DATA_HOME")
            .output()
            .expect("run bib")
    }
}

fn json(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|e| panic!("{e}: {}", String::from_utf8_lossy(&output.stdout)))
}

fn closed_port_url() -> String {
    let port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    format!("http://127.0.0.1:{port}")
}

fn write_config(dir: &Path, extra: &str) {
    std::fs::write(dir.join("bib.toml"), format!("id = \"p1\"\nname = \"Test\"\n{extra}")).unwrap();
}

#[test]
fn init_creates_database_and_project_file_once() {
    let env = Env::new(&closed_port_url());
    let first = env.bib(&["init"]);
    assert!(first.status.success(), "{}", String::from_utf8_lossy(&first.stderr));
    assert!(env.data.path().join("bib.db").exists());
    let config = std::fs::read_to_string(env.project.path().join("bib.toml")).unwrap();
    assert!(config.contains("libraries = [\"user\"]"));

    let second = env.bib(&["init", "--json"]);
    assert!(second.status.success());
    let report = json(&second);
    assert_eq!(report["created"], false);
    assert_eq!(report["schema_version"], 1);
}

#[test]
fn init_without_project_leaves_the_directory_alone() {
    let env = Env::new(&closed_port_url());
    assert!(env.bib(&["init", "--no-project"]).status.success());
    assert!(!env.project.path().join("bib.toml").exists());
}

#[test]
fn sync_then_doctor() {
    let zotero = FakeZotero::start();
    zotero.user_library(
        vec![
            fake::item("DWORK001", "dwork2006", "Differential Privacy", "2006", ""),
            fake::item("NOKEY001", "", "Untitled", "2020", ""),
        ],
        vec![],
        vec![],
    );
    let env = Env::new(zotero.url());

    let sync = env.bib(&["sync", "--json"]);
    assert!(sync.status.success(), "{}", String::from_utf8_lossy(&sync.stdout));
    let report = json(&sync);
    assert_eq!(report["libraries"][0]["library"], "user");
    assert_eq!(report["libraries"][0]["counts"]["sources_new"], 2);

    let doctor = env.bib(&["doctor", "--json"]);
    assert_eq!(doctor.status.code(), Some(1), "warnings exit with 1");
    let findings = json(&doctor);
    let codes: Vec<&str> = findings
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["code"].as_str().unwrap())
        .collect();
    assert!(codes.contains(&"missing_citation_key"), "{codes:?}");
    assert!(codes.contains(&"library"), "{codes:?}");
    assert!(!codes.contains(&"zotero_unreachable"), "{codes:?}");

    let text = env.bib(&["doctor"]);
    assert!(String::from_utf8_lossy(&text.stdout).contains("[warning] missing_citation_key:"));
}

#[test]
fn doctor_compares_the_project_bib_file() {
    let zotero = FakeZotero::start();
    zotero.user_library(
        vec![fake::item("DWORK001", "dwork2006", "Differential Privacy", "2006", "")],
        vec![],
        vec![],
    );
    let env = Env::new(zotero.url());
    write_config(env.project.path(), "[bibliography]\npath = \"refs.bib\"\n");
    std::fs::write(
        env.project.path().join("refs.bib"),
        "@article{ghost2019, title = {Not in Zotero}}\n",
    )
    .unwrap();
    let nested = env.project.path().join("chapters");
    std::fs::create_dir(&nested).unwrap();
    assert!(env.bib(&["sync"]).status.success());

    let doctor = Command::new(env!("CARGO_BIN_EXE_bib"))
        .args(["doctor", "--json"])
        .current_dir(&nested)
        .env("BIB_DATA_DIR", env.data.path())
        .env("BIB_ZOTERO_URL", zotero.url())
        .output()
        .unwrap();
    let codes: Vec<String> = json(&doctor)
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["code"].as_str().unwrap().to_string())
        .collect();
    assert!(
        codes.contains(&"project".to_string()),
        "project found from a subdirectory: {codes:?}"
    );
    assert!(codes.contains(&"bib_only".to_string()), "{codes:?}");
}

#[test]
fn sync_without_zotero_fails_with_the_last_sync_age() {
    let env = Env::new(&closed_port_url());
    let output = env.bib(&["sync"]);
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("not reachable"), "{stderr}");
    assert!(stderr.contains("Last successful sync: never"), "{stderr}");

    let json_output = env.bib(&["sync", "--json"]);
    assert_eq!(json_output.status.code(), Some(2));
    assert!(json(&json_output)["error"].as_str().unwrap().contains("not reachable"));
}

#[test]
fn invalid_project_file_is_a_failure() {
    let env = Env::new(&closed_port_url());
    write_config(env.project.path(), "[zotero]\nlibraries = [\"group:abc\"]\n");
    let output = env.bib(&["doctor"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("group:abc"));
}

#[test]
fn backup_writes_a_copy() {
    let env = Env::new(&closed_port_url());
    let output = env.bib(&["backup", "--json"]);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let path = json(&output)["backup"].as_str().unwrap().to_string();
    assert!(Path::new(&path).starts_with(env.data.path().join("backups")));
    assert!(Path::new(&path).exists());
}
