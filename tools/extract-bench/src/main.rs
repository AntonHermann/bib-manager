//! Extraction benchmark (Spec §15, step 0a).

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
#[command(about = "Measures PDF backends against a curated corpus")]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Downloads all corpus documents to bench/cache/ and checks their SHA-256 sums.
    Fetch,
    /// Extracts all corpus documents with all available backends and writes a report.
    Run {
        /// Target file for the Markdown report, e.g. bench/results/2026-09-18.md
        #[arg(long)]
        out: PathBuf,
        /// Comma-separated selection: pdf_oxide,pdf-extract,mutool,pdfium
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
            println!("downloading {} …", doc.id);
            let temp_path = path.with_extension("pdf.part");
            let status = Command::new("curl")
                .args(["-sSfL", "--retry", "3", "-o"])
                .arg(&temp_path)
                .arg(&doc.url)
                .status()
                .context("curl not executable")?;
            if !status.success() {
                let _ = std::fs::remove_file(&temp_path);
                bail!("download of {} failed ({})", doc.id, doc.url);
            }
            std::fs::rename(&temp_path, &path)?;
        }
        let sha = sha256_hex(&std::fs::read(&path)?);
        match check_lock(&lock, &doc.id, &sha) {
            LockStatus::Match => println!("ok    {}", doc.id),
            LockStatus::New => {
                println!("new   {} {sha}", doc.id);
                lock.insert(doc.id.clone(), sha);
            }
            LockStatus::Mismatch { expected } => bail!(
                "checksum of {} differs: expected {expected}, found {sha}. \
                 Delete the file and re-download, or deliberately adjust corpus.lock.",
                doc.id
            ),
        }
    }
    std::fs::write(&lock_path, toml::to_string(&lock)?)?;
    Ok(())
}

fn available_backends(selection: Option<Vec<String>>) -> anyhow::Result<Vec<Box<dyn Backend>>> {
    let mut all: Vec<Box<dyn Backend>> = vec![Box::new(PdfOxide), Box::new(PdfExtract)];
    let mutool = Mutool::default();
    if mutool.is_available() {
        all.push(Box::new(mutool));
    } else {
        eprintln!("mutool not installed, skipped");
    }
    #[cfg(feature = "pdfium")]
    match bib_extract::backends::pdfium::Pdfium::from_env() {
        Some(pdfium) => all.push(Box::new(pdfium)),
        None => eprintln!("BIB_PDFIUM_LIB_DIR not set, pdfium skipped"),
    }
    match selection {
        Some(names) => {
            let unknown: Vec<&str> =
                names.iter().filter(|n| !all.iter().any(|b| n.as_str() == b.name())).map(String::as_str).collect();
            if !unknown.is_empty() {
                bail!(
                    "unknown or unavailable backends: {} (available: {})",
                    unknown.join(", "),
                    all.iter().map(|b| b.name()).collect::<Vec<_>>().join(", ")
                );
            }
            Ok(all.into_iter().filter(|b| names.iter().any(|n| n == b.name())).collect())
        }
        None => Ok(all),
    }
}

fn run(out: PathBuf, selection: Option<Vec<String>>) -> anyhow::Result<()> {
    let corpus = load_corpus()?;
    let backends = available_backends(selection)?;
    let mut rows = Vec::new();
    for doc in &corpus.docs {
        let path = cache_path(&doc.id);
        anyhow::ensure!(path.exists(), "{} missing, run `fetch` first", doc.id);
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
    let mut header = String::from("# Extraction benchmark\n\n## Backends\n\n");
    for backend in &backends {
        header.push_str(&format!("- {} {}\n", backend.name(), backend.version()));
    }
    header.push_str("\n## Documents\n\n");
    for doc in &corpus.docs {
        header.push_str(&format!("- {}: {}\n", doc.id, doc.category));
    }
    header.push('\n');
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&out, header + &report::render_markdown(&rows))?;
    println!("report: {}", out.display());
    Ok(())
}
