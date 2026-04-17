"""Shared CSV reader/writer with `#` comment handling.

Comment lines (starting with `#` in the first cell) are stripped from the data
area but preserved so the output can optionally round-trip them back.
"""
from __future__ import annotations

import csv
from dataclasses import dataclass, field
from pathlib import Path


@dataclass
class ParsedCsv:
    """A parsed CSV split into (comments, header, data rows)."""

    header: list[str]
    rows: list[list[str]]
    comments: list[list[str]] = field(default_factory=list)


def read_csv(
    path: Path,
    delimiter: str = ";",
    encoding: str = "utf-8-sig",
) -> ParsedCsv:
    """Read a CSV, separating leading `#` comment rows from the header + data."""
    with path.open("r", encoding=encoding, newline="") as f:
        all_rows = list(csv.reader(f, delimiter=delimiter))

    comments: list[list[str]] = []
    header: list[str] = []
    data: list[list[str]] = []

    header_index = _find_header_index(all_rows)
    if header_index is None:
        return ParsedCsv(header=[], rows=[], comments=all_rows)

    comments = all_rows[:header_index]
    header = all_rows[header_index]
    for row in all_rows[header_index + 1 :]:
        if row and any(cell.strip() for cell in row):
            data.append(row)

    return ParsedCsv(header=header, rows=data, comments=comments)


def write_csv(
    parsed: ParsedCsv,
    path: Path,
    delimiter: str = ";",
    encoding: str = "utf-8",
    write_comments: bool = False,
) -> None:
    """Write back a parsed CSV. Optionally includes the preserved `#` comment rows."""
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding=encoding, newline="") as f:
        writer = csv.writer(f, delimiter=delimiter)
        if write_comments:
            for comment_row in parsed.comments:
                writer.writerow(comment_row)
        if parsed.header:
            writer.writerow(parsed.header)
        writer.writerows(parsed.rows)


def _find_header_index(rows: list[list[str]]) -> int | None:
    """Return the index of the first row that is non-empty and not a `#` comment."""
    for i, row in enumerate(rows):
        if not row:
            continue
        first = row[0].strip() if row[0] else ""
        if first.startswith("#"):
            continue
        if not first and len(row) == 1:
            continue  # empty row
        return i
    return None
