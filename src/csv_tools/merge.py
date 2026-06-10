"""Merge multiple CSVs with compatible headers."""
from __future__ import annotations

import logging
from pathlib import Path

from csv_tools.io import ParsedCsv, read_csv, write_csv


def merge_files(
    input_paths: list[Path],
    output_path: Path,
    delimiter: str = ";",
    encoding: str = "utf-8-sig",
    write_comments: bool = False,
    sanitize_formulas: bool = False,
    logger: logging.Logger | None = None,
) -> int:
    """Concatenate two or more CSVs into one, preserving the header from the first file.

    Returns the number of data rows written.
    """
    log = logger or logging.getLogger(__name__)
    if not input_paths:
        raise ValueError("merge_files requires at least one input path")

    parsed: list[ParsedCsv] = []
    for p in input_paths:
        if not p.exists():
            raise FileNotFoundError(f"Input file not found: {p}")
        log.info(f"Reading {p}")
        parsed.append(read_csv(p, delimiter=delimiter, encoding=encoding))

    first = parsed[0]
    header = first.header
    if not header:
        raise ValueError(f"No header row found in {input_paths[0]}")

    merged_rows: list[list[str]] = list(first.rows)
    for path, p in zip(input_paths[1:], parsed[1:], strict=True):
        if p.header != header:
            log.warning(f"Header mismatch in {path}: using header from {input_paths[0]}")
        merged_rows.extend(p.rows)

    result = ParsedCsv(
        header=header,
        rows=merged_rows,
        comments=first.comments if write_comments else [],
    )
    write_csv(
        result,
        output_path,
        delimiter=delimiter,
        write_comments=write_comments,
        sanitize_formulas=sanitize_formulas,
    )
    log.info(
        f"Merged {len(input_paths)} files → {len(merged_rows)} rows written to {output_path}"
    )
    return len(merged_rows)
