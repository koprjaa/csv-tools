# csv-tools

**Small stdlib-only CLI for merging, deduplicating, and reordering CSVs that may start with `#` comment rows. Three subcommands, zero dependencies.**

![python](https://img.shields.io/badge/python-3.10+-3776AB?style=flat-square&logo=python&logoColor=white)
![license](https://img.shields.io/badge/license-MIT-A31F34?style=flat-square)
![status](https://img.shields.io/badge/status-stable-22863A?style=flat-square)
![stdlib-only](https://img.shields.io/badge/deps-stdlib%20only-555?style=flat-square)
![ruff](https://img.shields.io/badge/lint-ruff-D7FF64?style=flat-square)
![pytest](https://img.shields.io/badge/test-pytest%20(30%20tests)-0A9EDC?style=flat-square&logo=pytest&logoColor=white)

A one-file-per-tool pattern was the original (`merge_csv.py`, `remove_duplicates.py`, `reorder_csv.py`) — useful but every run needed you to open the script and edit hardcoded paths. This version packages the same logic as a real CLI with subcommands, generalises the column targeting, and keeps the stdlib-only promise.

## Run

```bash
uv venv
uv pip install -e .
csv-tools --help
```

### Three subcommands

```bash
# merge — concatenate two or more CSVs (header taken from first file)
csv-tools merge a.csv b.csv c.csv -o merged.csv

# dedupe — drop duplicate rows, by a named column, a 0-based index, or full row
csv-tools dedupe merged.csv -o unique.csv --column "IČ firmy"
csv-tools dedupe merged.csv -o unique.csv --index 0
csv-tools dedupe merged.csv -o unique.csv                     # full-row dedup

# reorder — pull one or more columns to the front (rest keeps original order)
csv-tools reorder unique.csv -o final.csv --first "IČ firmy"
csv-tools reorder unique.csv -o final.csv --first "IČ firmy" --first "Název"
```

Equivalents:
```bash
python -m csv_tools merge ...
```

## Options (shared across subcommands)

| flag | default | effect |
|------|---------|--------|
| `-d, --delimiter <char>` | `;` | field delimiter (use `,` for comma CSVs) |
| `--encoding <enc>` | `utf-8-sig` | input encoding (the default handles BOM files) |
| `--keep-comments` | off | preserve `#` comment rows in the output (default: strip) |
| `--log-level <level>` | INFO | DEBUG / INFO / WARNING / ERROR |

## `#` comment row handling

Czech ERP exports (and plenty of other tools) prefix CSVs with metadata lines like:

```csv
# generated: 2026-04-17 09:00
# source: ARES → IČ lookup
IČ firmy;Název;Sídlo
12345678;Acme s.r.o.;Praha
```

All three subcommands auto-detect the first row that doesn't start with `#` as the header; the comment lines are stripped by default. Pass `--keep-comments` to copy them through to the output.

## Programmatic use

```python
from pathlib import Path
from csv_tools import merge_files, dedupe_by_column, reorder_columns

merge_files(
    [Path("jan.csv"), Path("feb.csv"), Path("mar.csv")],
    Path("q1.csv"),
)

dedupe_by_column(
    Path("q1.csv"),
    Path("q1-unique.csv"),
    column="IČ firmy",
)

reorder_columns(
    Path("q1-unique.csv"),
    Path("q1-final.csv"),
    first_columns=["IČ firmy", "Název"],
)
```

Each function accepts `delimiter`, `encoding`, `write_comments`, and `logger` kwargs for drop-in customisation.

## Architecture

```
src/csv_tools/
├── io.py          ParsedCsv dataclass + read_csv / write_csv with # comment handling
├── merge.py       merge_files(inputs, output, …)
├── dedupe.py      dedupe_by_column(input, output, column=name|index|None, …)
├── reorder.py     reorder_columns(input, output, first_columns=[…], …)
├── cli.py         argparse subcommands (merge / dedupe / reorder)
├── __main__.py    python -m csv_tools
└── __init__.py

tests/             pytest — 30 tests, coverage 94%
.github/workflows/ci.yml    ruff + pytest × 3.10/3.11/3.12 × Linux/Windows
pyproject.toml     modern packaging with 'csv-tools' entry point
```

## Typical pipeline

The README's original workflow is now one shell line:

```bash
csv-tools merge jan.csv feb.csv -o merged.csv && \
csv-tools dedupe merged.csv -o unique.csv --column "IČ firmy" && \
csv-tools reorder unique.csv -o final.csv --first "IČ firmy"
```

## Development

```bash
uv venv
uv pip install -e ".[dev]"
pytest -q                # 30 tests
ruff check .
```

## License

[MIT](LICENSE)
