// Project: csv-tools
// File: src/main.rs
// Description: Command line interface and stdout logging.
// Author: Jan Alexandr Kopřiva
// License: AGPL-3.0-or-later
use clap::{Args, Parser, Subcommand};
use csv_tools::*;
use std::{io::Write, path::PathBuf};

#[derive(Parser)]
#[command(
    name = "csv-tools",
    args_override_self = true,
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
        #[arg(long, allow_hyphen_values = true, value_parser = parse_index)]
        index: Option<String>,
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
// Python accepts decimal Unicode digits, separators, and surrounding whitespace.
fn parse_index(value: &str) -> std::result::Result<String, String> {
    const ZEROS: &[u32] = &[
        0x30, 0x660, 0x6f0, 0x7c0, 0x966, 0x9e6, 0xa66, 0xae6, 0xb66, 0xbe6, 0xc66, 0xce6, 0xd66,
        0xde6, 0xe50, 0xed0, 0xf20, 0x1040, 0x1090, 0x17e0, 0x1810, 0x1946, 0x19d0, 0x1a80, 0x1a90,
        0x1b50, 0x1bb0, 0x1c40, 0x1c50, 0xa620, 0xa8d0, 0xa900, 0xa9d0, 0xa9f0, 0xaa50, 0xabf0,
        0xff10, 0x104a0, 0x10d30, 0x11066, 0x110f0, 0x11136, 0x111d0, 0x112f0, 0x11450, 0x114d0,
        0x11650, 0x116c0, 0x11730, 0x118e0, 0x11950, 0x11c50, 0x11d50, 0x11da0, 0x16a60, 0x16ac0,
        0x16b50, 0x1d7ce, 0x1d7d8, 0x1d7e2, 0x1d7ec, 0x1d7f6, 0x1e140, 0x1e2f0, 0x1e950, 0x1fbf0,
    ];
    let value = value.trim_matches(|c: char| c.is_whitespace() && !(''..='').contains(&c));
    let (negative, digits) = if let Some(s) = value.strip_prefix('-') {
        (true, s)
    } else {
        (false, value.strip_prefix('+').unwrap_or(value))
    };
    let mut out = String::new();
    let mut previous_digit = false;
    for c in digits.chars() {
        if c == '_' && previous_digit {
            previous_digit = false;
            continue;
        }
        let digit = ZEROS
            .iter()
            .find_map(|&zero| (c as u32).checked_sub(zero).filter(|&n| n < 10));
        if let Some(n) = digit {
            out.push(char::from(b'0' + n as u8));
            previous_digit = true;
        } else {
            return Err("invalid integer index".into());
        }
    }
    if !previous_digit {
        return Err("invalid integer index".into());
    }
    let significant = out.trim_start_matches('0');
    Ok(if significant.is_empty() {
        "0".into()
    } else if negative {
        format!("-{significant}")
    } else {
        significant.into()
    })
}

/// For a header holding the same unique names as `first` in another order,
/// the index in `other` of each column of `first`.
fn same_columns(first: &Row, other: &Row) -> Option<Vec<usize>> {
    if first.len() != other.len() {
        return None;
    }
    let order: Vec<usize> = first
        .iter()
        .map(|name| other.iter().position(|c| c == name))
        .collect::<Option<_>>()?;
    let mut seen = order.clone();
    seen.sort_unstable();
    seen.dedup();
    (seen.len() == order.len()).then_some(order)
}

fn log(level: &str, threshold: &str, message: impl AsRef<str>) {
    let rank = |s| match s {
        "DEBUG" => 0,
        "INFO" => 1,
        "WARNING" => 2,
        _ => 3,
    };
    if rank(level) >= rank(threshold) {
        // A closed logging pipe must not interrupt the CSV transformation.
        let _ = writeln!(
            std::io::stdout().lock(),
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
                    if let Some(order) = same_columns(&first.header, &other.header) {
                        info(format!(
                            "Columns of {} are in another order: matching them by name",
                            p.display()
                        ));
                        first.rows.extend(other.rows.into_iter().map(|row| {
                            order
                                .iter()
                                .map(|&i| row.get(i).cloned().unwrap_or_default())
                                .collect()
                        }));
                        continue;
                    }
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
                    .as_ref()
                    .map(|value| {
                        value
                            .parse::<usize>()
                            .ok()
                            .filter(|&i| i < parsed.header.len())
                            .ok_or_else(|| {
                                format!(
                                    "Column index {value} out of range (header has {} columns)",
                                    parsed.header.len()
                                )
                            })
                    })
                    .transpose()?
            };
            let key = index
                .map(|i| format!("column index {i} ({})", repr(&parsed.header[i])))
                .unwrap_or("full row".into());
            info(format!("Deduping {} rows by {key}", parsed.rows.len()));
            if let Some(i) = index {
                let blank = parsed
                    .rows
                    .iter()
                    .filter(|row| row.get(i).is_none_or(|v| python_trim(v).is_empty()))
                    .count();
                if blank > 0 {
                    log(
                        "WARNING",
                        &cli.log_level,
                        format!("{blank} rows with an empty key are dropped"),
                    );
                }
            }
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
