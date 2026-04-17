"""Row deduplication by column name or index."""
from __future__ import annotations

import logging
from pathlib import Path

from csv_tools.io import ParsedCsv, read_csv, write_csv


def dedupe_by_column(
    input_path: Path,
    output_path: Path,
    column: str | int | None = None,
    delimiter: str = ";",
    encoding: str = "utf-8-sig",
    write_comments: bool = False,
    logger: logging.Logger | None = None,
) -> tuple[int, int]:
    """Remove duplicate rows based on a key column.

    The `column` argument is either a column name (matched against the header) or
    a 0-based integer index. If `None`, the full row is used as the key (standard
    deduplication).

    Returns `(unique_count, duplicate_count)`.
    """
    log = logger or logging.getLogger(__name__)
    if not input_path.exists():
        raise FileNotFoundError(f"Input file not found: {input_path}")

    parsed = read_csv(input_path, delimiter=delimiter, encoding=encoding)
    if not parsed.header:
        raise ValueError(f"No header row found in {input_path}")

    key_index = _resolve_column(column, parsed.header) if column is not None else None
    log.info(
        f"Deduping {len(parsed.rows)} rows by "
        f"{'full row' if key_index is None else f'column index {key_index} ({parsed.header[key_index]!r})'}"
    )

    seen: set[str | tuple] = set()
    unique_rows: list[list[str]] = []
    duplicates = 0

    for row in parsed.rows:
        key: str | tuple
        if key_index is None:
            key = tuple(row)
        else:
            key = row[key_index].strip() if key_index < len(row) and row[key_index] else ""
            if not key:
                duplicates += 1
                continue

        if key in seen:
            duplicates += 1
        else:
            seen.add(key)
            unique_rows.append(row)

    result = ParsedCsv(
        header=parsed.header,
        rows=unique_rows,
        comments=parsed.comments if write_comments else [],
    )
    write_csv(result, output_path, delimiter=delimiter, write_comments=write_comments)
    log.info(f"Wrote {len(unique_rows)} unique rows to {output_path} (dropped {duplicates})")
    return len(unique_rows), duplicates


def _resolve_column(column: str | int, header: list[str]) -> int:
    if isinstance(column, int):
        if column < 0 or column >= len(header):
            raise ValueError(f"Column index {column} out of range (header has {len(header)} columns)")
        return column
    try:
        return header.index(column)
    except ValueError as e:
        raise ValueError(f"Column {column!r} not found in header: {header}") from e
