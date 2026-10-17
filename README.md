# csv-tools

Command line tool that merges, deduplicates, and reorders CSV files. It reads files that start with `#` comment rows. It is implemented in Rust.

![rust](https://img.shields.io/badge/rust-stable-DEA584?style=flat-square&logo=rust&logoColor=white)
![license](https://img.shields.io/badge/license-MIT-A31F34?style=flat-square)
![status](https://img.shields.io/badge/status-stable-22863A?style=flat-square)
[![ci](https://github.com/koprjaa/csv-tools/actions/workflows/ci.yml/badge.svg)](https://github.com/koprjaa/csv-tools/actions/workflows/ci.yml)

## What it does

Czech ERP systems export CSV files with metadata lines above the header:

```csv
# generated: 2026-04-17 09:00
# source: ARES lookup
IČ firmy;Název;Sídlo
12345678;Acme s.r.o.;Praha
```

All three subcommands find the first row that does not start with `#` and use it as the header. The tool removes the comment rows by default. Use `--keep-comments` to copy them to the output.

## Install

```bash
cargo install --path .
csv-tools --help
```

## Use

Merge two or more files. The header comes from the first file.

```bash
csv-tools merge a.csv b.csv c.csv -o merged.csv
```

Remove duplicate rows. Select a column by name, by 0-based index, or compare the full row.

```bash
csv-tools dedupe merged.csv -o unique.csv --column "IČ firmy"
csv-tools dedupe merged.csv -o unique.csv --index 0
csv-tools dedupe merged.csv -o unique.csv
```

Move one or more columns to the front. The other columns keep their order.

```bash
csv-tools reorder unique.csv -o final.csv --first "IČ firmy"
csv-tools reorder unique.csv -o final.csv --first "IČ firmy" --first "Název"
```

Chain the three steps:

```bash
csv-tools merge jan.csv feb.csv -o merged.csv && \
csv-tools dedupe merged.csv -o unique.csv --column "IČ firmy" && \
csv-tools reorder unique.csv -o final.csv --first "IČ firmy"
```

Logs use local time and are written to stdout as `HH:MM:SS [LEVEL] message`. Put `--log-level` before the subcommand.

## Options

These flags work with all three subcommands.

| Flag | Default | Effect |
|---|---|---|
| `-d, --delimiter <char>` | `;` | Field delimiter. Use `,` for comma files. |
| `--encoding <enc>` | `utf-8-sig` | Input encoding. The default reads files with a BOM. |
| `--keep-comments` | off | Copy `#` rows to the output. |
| `--log-level <level>` | `INFO` | One of DEBUG, INFO, WARNING, ERROR. |

## CSV semantics

Leading blank records, single whitespace cells, and records whose trimmed first cell starts with `#` precede the header. A record with multiple empty fields can be the header. After the header, entirely blank or whitespace-only records are dropped; comment records remain data. Merge retains only the first input's leading comments and warns about incompatible headers without rearranging data. One input to merge is accepted.

Dedupe retains the first occurrence. Column keys are trimmed; empty or missing keys are dropped and counted as duplicates. Full-row comparison preserves whitespace. Reorder repeats columns when `--first` repeats, pads missing cells, and discards cells beyond the header width.

CSV output uses CRLF, minimal quoting, doubled quotes, and UTF-8 without BOM. The `--encoding` option controls input only, matching the original command behavior. Default `utf-8-sig` removes a UTF-8 BOM; explicit `utf-8` preserves it. Formula sanitization applies to comments, headers, and data.

| Flag | Default | Effect |
|---|---|---|
| `--sanitize-formulas` | off | Prefix cells starting with `= + - @`, tab, or CR with a single quote. |

## Changes from the Python version

- The executable and Rust library replace the Python package, Python imports, and `python -m csv_tools` entry point. Rust helpers use `Result` errors rather than Python exceptions.
- CLI help, usage, and argument-error text come from clap instead of argparse. Argparse's abbreviated long options are unsupported. Index arguments accept signs, surrounding whitespace, underscores between decimal digits, and Unicode decimal digits. Arbitrarily large indices are reported as out of range without integer overflow; unlike recent Python versions, Rust does not impose Python’s configurable decimal conversion digit limit.
- Input encoding labels follow encoding_rs / WHATWG, rather than Python codecs. Unsupported Python codecs (for example UTF-32) are rejected. ASCII and Latin-1 (including their standard aliases) retain Python semantics: strict 7-bit ASCII and direct byte-to-Unicode Latin-1. Other codecs retain encoding_rs / WHATWG semantics, which can differ from Python for legacy and multibyte encodings; not every Python codec alias is available. Labels normalize case and runs of punctuation or spaces; UTF-8 BOM and UTF-16 aliases are handled explicitly. Generic `utf-16` negotiates byte order from a required BOM (empty input is accepted); explicit `utf-16le` / `utf-16be` retain their byte order and BOM as data. Invalid byte sequences and undefined Python charmap entries in Windows-1250–1258, Windows-874, and ISO-8859-3/6/7/8 are rejected with the input filename and a zero-based byte position in the original file. In particular, cp1250 rejects `81 83 88 90 98` and cp1252 rejects `81 8D 8F 90 9D` (hex); ISO-8859-2 defines every byte. UTF-8 is strict; `utf-8-sig` strips only an initial UTF-8 BOM. This does not detect a wrongly selected encoding when all bytes are valid in it. Error wording differs from Python.
- File-system errors other than missing input and CSV decoding errors are reported as stdout error messages with exit code 2 instead of Python tracebacks. Unknown encoding labels also produce a logged error with exit code 2 instead of a LookupError traceback with exit code 1. Invalid delimiters are CLI argument errors. CSV field-limit errors are reported with exit code 2 instead of a traceback and exit code 1.
- Unicode whitespace follows Rust's Unicode tables with Python's additional ASCII information separators; Unicode-version differences can affect newly assigned characters. Diagnostic quoting of unusual nonprinting Unicode characters can differ.
- Output CSV encoding remains UTF-8 without BOM for every subcommand, including when a different input encoding is selected. This preserves the original implementation.
- Repeated scalar CLI options use the last value, repeated boolean flags are accepted, and repeated `--first` options append, matching Python. Clap accepts `--help` in addition to `-h`; its detailed option/value parsing and placement rules differ from argparse.
- Closed or failing stdout logging streams no longer panic: CSV work completes and logging write errors are ignored. Python emits logging tracebacks and can exit 120 on shutdown after a broken pipe. CSV output write failures still return exit code 2.
- Rust library helpers operate on `ParsedCsv` in memory; Python’s path-based transformation helpers, optional logger, configurable writer encoding, and mutable global CSV field limit are not exposed. Rust always writes UTF-8 and enforces 131072 Unicode characters per field.
- Non-UTF-8 command-line option values are rejected by clap; filesystem paths use native Rust paths. Displayed paths may preserve `./` or use replacement characters where Python normalizes paths or uses surrogate escapes.
- Files remain in memory as in the original implementation; output is buffered and written record by record. No streaming-input guarantee is added.

## Dependencies

`clap` provides CLI parsing and validation; `encoding_rs` decodes input labels; `chrono` formats local log timestamps. The test-only `tempfile` isolates test data. CSV parsing and writing use the standard library to preserve blank records, Unicode delimiters, and permissive Python quoting.

## Development

```bash
cargo build --release
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

CI runs formatting, clippy, and tests on stable Rust on Linux and Windows.

## Origin

Based on [csv-tools](https://github.com/koprjaa/csv-tools) by Jan Alexandr Kopřiva. This Rust implementation preserves the original project attribution and MIT copyright.

## License

[MIT](LICENSE)
