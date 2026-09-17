//! Extraktions-Benchmark (Spec §15, Schritt 0a).

mod corpus;

use std::collections::BTreeMap;
use std::process::Command;

use anyhow::{Context, bail};
use clap::{Parser, Subcommand};

use corpus::{LockStatus, bench_dir, cache_path, check_lock, parse_corpus, sha256_hex};

#[derive(Parser)]
#[command(about = "Misst PDF-Backends an einem kuratierten Korpus")]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Lädt alle Korpus-Dokumente nach bench/cache/ und prüft ihre SHA-256-Summen.
    Fetch,
}

fn main() -> anyhow::Result<()> {
    match Cli::parse().command {
        Cmd::Fetch => fetch(),
    }
}

fn load_corpus() -> anyhow::Result<corpus::Corpus> {
    let path = bench_dir().join("corpus.toml");
    parse_corpus(&std::fs::read_to_string(&path).with_context(|| format!("{}", path.display()))?)
}

fn fetch() -> anyhow::Result<()> {
    let corpus = load_corpus()?;
    let lock_path = bench_dir().join("corpus.lock");
    let mut lock: BTreeMap<String, String> = match std::fs::read_to_string(&lock_path) {
        Ok(s) => toml::from_str(&s)?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => BTreeMap::new(),
        Err(e) => return Err(e.into()),
    };
    std::fs::create_dir_all(bench_dir().join("cache"))?;
    for doc in &corpus.docs {
        let path = cache_path(&doc.id);
        if !path.exists() {
            println!("lade {} …", doc.id);
            let status = Command::new("curl")
                .args(["-sSfL", "--retry", "3", "-o"])
                .arg(&path)
                .arg(&doc.url)
                .status()
                .context("curl nicht ausführbar")?;
            if !status.success() {
                bail!("Download von {} fehlgeschlagen ({})", doc.id, doc.url);
            }
        }
        let sha = sha256_hex(&std::fs::read(&path)?);
        match check_lock(&lock, &doc.id, &sha) {
            LockStatus::Match => println!("ok    {}", doc.id),
            LockStatus::New => {
                println!("neu   {} {sha}", doc.id);
                lock.insert(doc.id.clone(), sha);
            }
            LockStatus::Mismatch { expected } => bail!(
                "Prüfsumme von {} weicht ab: erwartet {expected}, gefunden {sha}. \
                 Datei löschen und neu laden, oder corpus.lock bewusst anpassen.",
                doc.id
            ),
        }
    }
    std::fs::write(&lock_path, toml::to_string(&lock)?)?;
    Ok(())
}
