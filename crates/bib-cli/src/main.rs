//! The `bib` command line (spec §10). Exit codes: 0 success, 1 problems found, 2 failure.

use std::path::Path;
use std::process::ExitCode;

use bib_core::db::{self, Connection, Opened};
use bib_core::doctor::{self, DoctorInput, Finding, Project, Severity};
use bib_core::paths;
use bib_core::project::{CONFIG_FILE, ProjectConfig, find_project_root, init_project};
use bib_core::zotero::sync::{LibraryReport, last_successful_sync_age, sync_all};
use bib_core::zotero::{ZoteroClient, ZoteroError};
use clap::{Parser, Subcommand};
use serde_json::json;

#[derive(Parser)]
#[command(
    name = "bib",
    version,
    about = "Bibliography manager: Zotero sync, quote verification, Typst integration"
)]
struct Cli {
    /// Print one JSON document instead of text.
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create or migrate the database and write a `bib.toml` into the current directory.
    Init {
        /// Only set up the database.
        #[arg(long)]
        no_project: bool,
    },
    /// Read all Zotero libraries (read-only) and update the database.
    Sync,
    /// Check the database, the Zotero sync and the current project.
    Doctor,
    /// Write a consistent copy of the database into the backup directory.
    Backup,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(&cli) {
        Ok(code) => code,
        Err(err) => {
            if cli.json {
                println!("{}", json!({"error": format!("{err:#}")}));
            } else {
                eprintln!("error: {err:#}");
            }
            ExitCode::from(2)
        }
    }
}

fn run(cli: &Cli) -> anyhow::Result<ExitCode> {
    let db_path = paths::db_path();
    let (mut conn, opened) = db::open(&db_path, &paths::backup_dir())?;
    if let Some(backup) = &opened.backup
        && !cli.json
    {
        eprintln!("backed up the database to {} before migrating", backup.display());
    }
    match &cli.command {
        Command::Init { no_project } => init(cli, &db_path, &opened, *no_project),
        Command::Sync => sync(cli, &mut conn),
        Command::Doctor => doctor(cli, &conn, &db_path),
        Command::Backup => backup(cli, &conn),
    }
}

fn init(cli: &Cli, db_path: &Path, opened: &Opened, no_project: bool) -> anyhow::Result<ExitCode> {
    let dir = std::env::current_dir()?;
    let config_path = dir.join(CONFIG_FILE);
    let created = if no_project || config_path.exists() {
        false
    } else {
        init_project(&dir)?;
        true
    };
    if cli.json {
        println!(
            "{}",
            json!({
                "database": db_path,
                "schema_version": opened.to_version,
                "project_file": (!no_project).then_some(&config_path),
                "created": created,
            })
        );
    } else {
        println!("database {} (schema version {})", db_path.display(), opened.to_version);
        if created {
            println!("created {}", config_path.display());
        } else if !no_project {
            println!("{} already exists, left unchanged", config_path.display());
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn sync(cli: &Cli, conn: &mut Connection) -> anyhow::Result<ExitCode> {
    let client = ZoteroClient::from_env();
    let reports = match sync_all(conn, &client) {
        Ok(reports) => reports,
        Err(err) if matches!(err.downcast_ref::<ZoteroError>(), Some(ZoteroError::Unreachable { .. })) => {
            let last = match last_successful_sync_age(conn)? {
                Some(age) => format!("{} ago", doctor::format_age(age)),
                None => "never".to_string(),
            };
            anyhow::bail!(
                "{err}. Last successful sync: {last}. Start Zotero and enable \"Allow other applications on this computer to communicate with Zotero\" (Settings → Advanced)."
            );
        }
        Err(err) => return Err(err),
    };
    if cli.json {
        println!("{}", json!({ "libraries": reports }));
    } else {
        for report in &reports {
            print_report(report);
        }
    }
    Ok(ExitCode::SUCCESS)
}

fn print_report(report: &LibraryReport) {
    let c = &report.counts;
    println!(
        "{} \"{}\": {} new, {} changed, {} retired, {} reactivated, {} PDF attachments",
        report.library,
        report.name,
        c.sources_new,
        c.sources_changed,
        c.sources_retired,
        c.sources_reactivated,
        c.attachments
    );
}

fn doctor(cli: &Cli, conn: &Connection, db_path: &Path) -> anyhow::Result<ExitCode> {
    let zotero = ZoteroClient::from_env().groups().map(|_| ()).map_err(|e| e.to_string());
    let root = find_project_root(&std::env::current_dir()?);
    let config = root.as_deref().map(ProjectConfig::load).transpose()?;
    let project = root
        .as_deref()
        .zip(config.as_ref())
        .map(|(root, config)| Project { root, config });
    let findings = doctor::run(&DoctorInput {
        conn,
        db_path,
        zotero,
        project,
    })?;
    if cli.json {
        println!("{}", serde_json::to_string(&findings)?);
    } else {
        findings.iter().for_each(print_finding);
    }
    let problems = findings.iter().any(|f| f.severity >= Severity::Warning);
    Ok(if problems { ExitCode::from(1) } else { ExitCode::SUCCESS })
}

fn print_finding(finding: &Finding) {
    let label = match finding.severity {
        Severity::Info => "info",
        Severity::Warning => "warning",
        Severity::Error => "error",
    };
    println!("[{label}] {}: {}", finding.code, finding.message);
}

fn backup(cli: &Cli, conn: &Connection) -> anyhow::Result<ExitCode> {
    let dir = paths::backup_dir();
    let path = db::backup(conn, &dir)?;
    db::prune_backups(&dir, db::KEEP_BACKUPS)?;
    if cli.json {
        println!("{}", json!({ "backup": path }));
    } else {
        println!("backup written to {}", path.display());
    }
    Ok(ExitCode::SUCCESS)
}
