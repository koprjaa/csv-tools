# csv-tools

Command line tool that merges, deduplicates, and reorders CSV files. It reads files that start with `#` comment rows. It uses the Python standard library only.

![python](https://img.shields.io/badge/python-3.10+-3776AB?style=flat-square&logo=python&logoColor=white)
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
uv venv
uv pip install -e .
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

`python -m csv_tools <subcommand>` does the same as `csv-tools <subcommand>`.

## Options

These flags work with all three subcommands.

| Flag | Default | Effect |
|---|---|---|
| `-d, --delimiter <char>` | `;` | Field delimiter. Use `,` for comma files. |
| `--encoding <enc>` | `utf-8-sig` | Input encoding. The default reads files with a BOM. |
| `--keep-comments` | off | Copy `#` rows to the output. |
| `--log-level <level>` | `INFO` | One of DEBUG, INFO, WARNING, ERROR. |

## Library use

```python
from pathlib import Path
from csv_tools import merge_files, dedupe_by_column, reorder_columns

merge_files([Path("jan.csv"), Path("feb.csv")], Path("q1.csv"))
dedupe_by_column(Path("q1.csv"), Path("q1-unique.csv"), column="IČ firmy")
reorder_columns(Path("q1-unique.csv"), Path("q1-final.csv"), first_columns=["IČ firmy"])
```

Each function accepts `delimiter`, `encoding`, `write_comments`, and `logger`.

## How it works

```
src/csv_tools/
  io.py         ParsedCsv dataclass, read_csv and write_csv with comment handling
  merge.py      merge_files(inputs, output)
  dedupe.py     dedupe_by_column(input, output, column=name or index or None)
  reorder.py    reorder_columns(input, output, first_columns=[...])
  cli.py        argparse subcommands
  __main__.py   entry point for python -m csv_tools
```

## Limits

The tool holds one file in memory at a time. It does not stream. Very large files need a different approach.

## Development

```bash
uv venv
uv pip install -e ".[dev]"
pytest -q
ruff check .
```

CI runs ruff and pytest on Python 3.10, 3.11, and 3.12, on Linux and Windows.

## License

[MIT](LICENSE)
