use std::path::PathBuf;

/// Path of a corpus document; aborts with instructions if it is missing.
pub fn fixture(id: &str) -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../bench/cache")
        .join(format!("{id}.pdf"));
    assert!(
        path.exists(),
        "Fixture {id} missing: run `cargo run -p extract-bench -- fetch` first"
    );
    path
}
