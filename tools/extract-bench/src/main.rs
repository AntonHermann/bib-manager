//! Extraktions-Benchmark (Spec §15, Schritt 0a).

mod corpus;
mod metrics;
mod report;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

use anyhow::{Context, bail};
use bib_extract::Backend;
use bib_extract::backends::{mutool::Mutool, oxide::PdfOxide, pdf_extract::PdfExtract};
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
    /// Extrahiert alle Korpus-Dokumente mit allen verfügbaren Backends und schreibt einen Bericht.
    Run {
        /// Zieldatei des Markdown-Berichts, z. B. bench/results/2026-09-18.md
        #[arg(long)]
        out: PathBuf,
        /// Kommagetrennte Auswahl: pdf_oxide,pdf-extract,mutool,pdfium
        #[arg(long, value_delimiter = ',')]
        backends: Option<Vec<String>>,
    },
}

fn main() -> anyhow::Result<()> {
    match Cli::parse().command {
        Cmd::Fetch => fetch(),
        Cmd::Run { out, backends } => run(out, backends),
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
            let temp_path = path.with_extension("pdf.part");
            let status = Command::new("curl")
                .args(["-sSfL", "--retry", "3", "-o"])
                .arg(&temp_path)
                .arg(&doc.url)
                .status()
                .context("curl nicht ausführbar")?;
            if !status.success() {
                let _ = std::fs::remove_file(&temp_path);
                bail!("Download von {} fehlgeschlagen ({})", doc.id, doc.url);
            }
            std::fs::rename(&temp_path, &path)?;
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

fn available_backends(selection: Option<Vec<String>>) -> Vec<Box<dyn Backend>> {
    let mut all: Vec<Box<dyn Backend>> = vec![Box::new(PdfOxide), Box::new(PdfExtract)];
    let mutool = Mutool::default();
    if mutool.is_available() {
        all.push(Box::new(mutool));
    } else {
        eprintln!("mutool nicht installiert, übersprungen");
    }
    #[cfg(feature = "pdfium")]
    match bib_extract::backends::pdfium::Pdfium::from_env() {
        Some(pdfium) => all.push(Box::new(pdfium)),
        None => eprintln!("BIB_PDFIUM_LIB_DIR nicht gesetzt, pdfium übersprungen"),
    }
    match selection {
        Some(names) => all.into_iter().filter(|b| names.iter().any(|n| n == b.name())).collect(),
        None => all,
    }
}

fn run(out: PathBuf, selection: Option<Vec<String>>) -> anyhow::Result<()> {
    let corpus = load_corpus()?;
    let backends = available_backends(selection);
    let mut rows = Vec::new();
    for doc in &corpus.docs {
        let path = cache_path(&doc.id);
        anyhow::ensure!(path.exists(), "{} fehlt, zuerst `fetch` ausführen", doc.id);
        for backend in &backends {
            let started = Instant::now();
            let result = backend.extract(&path);
            let millis = started.elapsed().as_millis();
            let row = match result {
                Ok(extraction) => report::Row {
                    doc: doc.id.clone(),
                    backend: backend.name().to_string(),
                    scores: Some(metrics::evaluate(&doc.expect, &extraction.text())),
                    pages: extraction.pages.len(),
                    pages_with_geometry: extraction.pages.iter().filter(|p| p.has_geometry()).count(),
                    millis,
                    error: None,
                },
                Err(e) => report::Row {
                    doc: doc.id.clone(),
                    backend: backend.name().to_string(),
                    scores: None,
                    pages: 0,
                    pages_with_geometry: 0,
                    millis,
                    error: Some(e.to_string()),
                },
            };
            println!("{:12} {:12} {} ms", row.doc, row.backend, row.millis);
            rows.push(row);
        }
    }
    let mut header = String::from("# Extraktions-Benchmark\n\n## Backends\n\n");
    for backend in &backends {
        header.push_str(&format!("- {} {}\n", backend.name(), backend.version()));
    }
    header.push_str("\n## Dokumente\n\n");
    for doc in &corpus.docs {
        header.push_str(&format!("- {}: {}\n", doc.id, doc.category));
    }
    header.push('\n');
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&out, header + &report::render_markdown(&rows))?;
    println!("Bericht: {}", out.display());
    Ok(())
}
