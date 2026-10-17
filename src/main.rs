// Project: csv-tools
// File: src/main.rs
// Description: Command line interface and stdout logging.
// Author: Jan Alexandr Kopřiva
// License: MIT

use clap::{Args, Parser, Subcommand};
use csv_tools::*;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "csv-tools",
    about = "Merge, dedupe, and reorder semicolon-delimited CSVs with optional `#` comment rows."
)]
struct Cli {
    #[arg(long, default_value = "INFO", value_parser = ["DEBUG", "INFO", "WARNING", "ERROR"])]
    log_level: String,
    #[command(subcommand)]
    command: Command,
}
#[derive(Args)]
struct Io {
    #[arg(short, long)]
    output: PathBuf,
    #[arg(short, long, default_value = ";")]
    delimiter: char,
    #[arg(long, default_value = "utf-8-sig")]
    encoding: String,
    #[arg(long)]
    keep_comments: bool,
    #[arg(long)]
    sanitize_formulas: bool,
}
#[derive(Subcommand)]
enum Command {
    Merge {
        #[arg(required = true, num_args = 1..)]
        inputs: Vec<PathBuf>,
        #[command(flatten)]
        io: Io,
    },
    Dedupe {
        input: PathBuf,
        #[arg(long, conflicts_with = "index")]
        column: Option<String>,
        #[arg(long, allow_hyphen_values = true)]
        index: Option<i64>,
        #[command(flatten)]
        io: Io,
    },
    Reorder {
        input: PathBuf,
        #[arg(long, required = true)]
        first: Vec<String>,
        #[command(flatten)]
        io: Io,
    },
}
fn log(level: &str, threshold: &str, message: impl AsRef<str>) {
    let rank = |s| match s {
        "DEBUG" => 0,
        "INFO" => 1,
        "WARNING" => 2,
        _ => 3,
    };
    if rank(level) >= rank(threshold) {
        println!(
            "{} [{level}] {}",
            chrono::Local::now().format("%H:%M:%S"),
            message.as_ref()
        );
    }
}
fn run(cli: &Cli) -> Result<()> {
    let info = |s: String| log("INFO", &cli.log_level, s);
    let (mut parsed, io) = match &cli.command {
        Command::Merge { inputs, io } => {
            let mut files = Vec::new();
            for p in inputs {
                if !p.exists() {
                    return Err(format!("Input file not found: {}", p.display()));
                }
                info(format!("Reading {}", p.display()));
                files.push(read_csv(p, io.delimiter, &io.encoding)?);
            }
            let mut files = files.into_iter();
            let mut first = files
                .next()
                .ok_or("merge_files requires at least one input path")?;
            if first.header.is_empty() {
                return Err(format!("No header row found in {}", inputs[0].display()));
            }
            for (p, other) in inputs[1..].iter().zip(files) {
                if other.header != first.header {
                    log(
                        "WARNING",
                        &cli.log_level,
                        format!(
                            "Header mismatch in {}: using header from {}",
                            p.display(),
                            inputs[0].display()
                        ),
                    );
                }
                first.rows.extend(other.rows);
            }
            (first, io)
        }
        Command::Dedupe { input, io, .. } | Command::Reorder { input, io, .. } => {
            let parsed = read_csv(input, io.delimiter, &io.encoding)?;
            if parsed.header.is_empty() {
                return Err(format!("No header row found in {}", input.display()));
            }
            (parsed, io)
        }
    };
    let message = match &cli.command {
        Command::Merge { inputs, .. } => format!(
            "Merged {} files → {} rows written to {}",
            inputs.len(),
            parsed.rows.len(),
            io.output.display()
        ),
        Command::Dedupe { column, index, .. } => {
            let index = if let Some(name) = column {
                Some(find_column(&parsed.header, name)?)
            } else {
                index
                    .map(|i| resolve_index(&parsed.header, i))
                    .transpose()?
            };
            let key = index
                .map(|i| format!("column index {i} ({})", repr(&parsed.header[i])))
                .unwrap_or("full row".into());
            info(format!("Deduping {} rows by {key}", parsed.rows.len()));
            let (unique, dropped) = dedupe(&mut parsed, index);
            format!(
                "Wrote {unique} unique rows to {} (dropped {dropped})",
                io.output.display()
            )
        }
        Command::Reorder { first, .. } => {
            reorder(&mut parsed, first)?;
            info(format!("Reordered header: {}", repr_row(&parsed.header)));
            format!(
                "Wrote {} rows to {}",
                parsed.rows.len(),
                io.output.display()
            )
        }
    };
    write_csv(
        &parsed,
        &io.output,
        io.delimiter,
        io.keep_comments,
        io.sanitize_formulas,
    )?;
    info(message);
    Ok(())
}
fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(&cli) {
        log("ERROR", &cli.log_level, e);
        std::process::exit(2);
    }
}
