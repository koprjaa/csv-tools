# csv-tools

Merge, deduplicate and reorder CSV files from the command line.

[![ci](https://github.com/koprjaa/csv-tools/actions/workflows/ci.yml/badge.svg)](https://github.com/koprjaa/csv-tools/actions/workflows/ci.yml)
[![license](https://img.shields.io/badge/license-MIT-A31F34?style=flat-square)](LICENSE)

## Commands

- `merge`: combine two or more files; the header comes from the first file.
- `dedupe`: drop duplicate rows by column name (`--column`), 0-based index (`--index`), or the full row.
- `reorder`: move columns to the front with `--first`, repeatable; other columns keep their order.

Rows starting with `#` above the header are skipped (`--keep-comments` keeps them). The default delimiter is `;`, change it with `-d`.

## Install

```bash
cargo install --path .
```

## Use

```bash
csv-tools merge jan.csv feb.csv -o merged.csv
csv-tools dedupe merged.csv -o unique.csv --column "IČ firmy"
csv-tools reorder unique.csv -o final.csv --first "IČ firmy" --first "Název"
```

Run `csv-tools --help` for the full option list.

## License

[MIT](LICENSE)
