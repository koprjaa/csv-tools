"""Shared CSV reader/writer with `#` comment handling.

Comment lines (starting with `#` in the first cell) are stripped from the data
area but preserved so the output can optionally round-trip them back.
"""
from __future__ import annotations

import csv
from dataclasses import dataclass, field
from pathlib import Path

# Leading characters that a spreadsheet (Excel, LibreOffice, Sheets) may
# interpret as the start of a formula. A cell beginning with one of these can
# execute on open — CSV injection / formula injection. See OWASP "CSV Injection".
_FORMULA_TRIGGERS = ("=", "+", "-", "@", "\t", "\r")


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


def sanitize_cell(value: str) -> str:
    """Neutralize a cell that a spreadsheet could interpret as a formula.

    Prefixes a single quote (`'`) to any value whose first character is a known
    formula trigger (`= + - @`, tab, or CR), defusing CSV/formula injection while
    keeping the original text visible to the user. Returns non-risky values
    unchanged. Opt-in only — see `write_csv(sanitize_formulas=...)`.
    """
    if value and value[0] in _FORMULA_TRIGGERS:
        return "'" + value
    return value


def write_csv(
    parsed: ParsedCsv,
    path: Path,
    delimiter: str = ";",
    encoding: str = "utf-8",
    write_comments: bool = False,
    sanitize_formulas: bool = False,
) -> None:
    """Write back a parsed CSV. Optionally includes the preserved `#` comment rows.

    When ``sanitize_formulas`` is True, cells starting with a formula trigger
    (`= + - @`, tab, CR) are prefixed with `'` to prevent CSV injection when the
    output is opened in a spreadsheet. Defaults to False to preserve data verbatim.
    """
    path.parent.mkdir(parents=True, exist_ok=True)

    def _row(row: list[str]) -> list[str]:
        return [sanitize_cell(c) for c in row] if sanitize_formulas else row

    with path.open("w", encoding=encoding, newline="") as f:
        writer = csv.writer(f, delimiter=delimiter)
        if write_comments:
            for comment_row in parsed.comments:
                writer.writerow(_row(comment_row))
        if parsed.header:
            writer.writerow(_row(parsed.header))
        writer.writerows(_row(r) for r in parsed.rows)


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
