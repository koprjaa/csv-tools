"""Column reordering — move named columns to the front, keep the rest in original order."""
from __future__ import annotations

import logging
from pathlib import Path

from csv_tools.io import ParsedCsv, read_csv, write_csv


def reorder_columns(
    input_path: Path,
    output_path: Path,
    first_columns: list[str],
    delimiter: str = ";",
    encoding: str = "utf-8-sig",
    write_comments: bool = False,
    sanitize_formulas: bool = False,
    logger: logging.Logger | None = None,
) -> int:
    """Reorder columns so `first_columns` appear first (in that order), rest kept original order.

    Returns the number of rows written.
    """
    log = logger or logging.getLogger(__name__)
    if not input_path.exists():
        raise FileNotFoundError(f"Input file not found: {input_path}")

    parsed = read_csv(input_path, delimiter=delimiter, encoding=encoding)
    if not parsed.header:
        raise ValueError(f"No header row found in {input_path}")

    first_indices = [_find_column(col, parsed.header) for col in first_columns]
    skip_set = set(first_indices)
    rest_indices = [i for i in range(len(parsed.header)) if i not in skip_set]
    new_order = first_indices + rest_indices

    new_header = [parsed.header[i] for i in new_order]
    new_rows: list[list[str]] = []
    for row in parsed.rows:
        new_rows.append([row[i] if i < len(row) else "" for i in new_order])

    log.info(f"Reordered header: {new_header}")

    result = ParsedCsv(
        header=new_header,
        rows=new_rows,
        comments=parsed.comments if write_comments else [],
    )
    write_csv(
        result,
        output_path,
        delimiter=delimiter,
        write_comments=write_comments,
        sanitize_formulas=sanitize_formulas,
    )
    log.info(f"Wrote {len(new_rows)} rows to {output_path}")
    return len(new_rows)


def _find_column(name: str, header: list[str]) -> int:
    try:
        return header.index(name)
    except ValueError as e:
        raise ValueError(f"Column {name!r} not found in header: {header}") from e
