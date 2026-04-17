"""Small stdlib-only CLI for merging, deduplicating, and reordering CSVs."""
from csv_tools.dedupe import dedupe_by_column
from csv_tools.io import ParsedCsv, read_csv, write_csv
from csv_tools.merge import merge_files
from csv_tools.reorder import reorder_columns

__version__ = "0.2.0"

__all__ = [
    "ParsedCsv",
    "dedupe_by_column",
    "merge_files",
    "read_csv",
    "reorder_columns",
    "write_csv",
]
