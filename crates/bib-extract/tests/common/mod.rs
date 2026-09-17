use std::path::PathBuf;

/// Pfad eines Korpus-Dokuments; bricht mit Anleitung ab, wenn es fehlt.
pub fn fixture(id: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../bench/cache")
        .join(format!("{id}.pdf"));
    assert!(
        path.exists(),
        "Fixture {id} fehlt: zuerst `cargo run -p extract-bench -- fetch` ausführen"
    );
    path
}
