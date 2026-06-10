"""CLI — three subcommands: merge, dedupe, reorder."""
from __future__ import annotations

import argparse
import logging
import sys
from pathlib import Path

from csv_tools.dedupe import dedupe_by_column
from csv_tools.merge import merge_files
from csv_tools.reorder import reorder_columns


def _setup_logging(level: str) -> logging.Logger:
    logger = logging.getLogger("csv_tools")
    logger.setLevel(getattr(logging, level.upper()))
    for handler in logger.handlers[:]:
        logger.removeHandler(handler)
    handler = logging.StreamHandler(sys.stdout)
    handler.setFormatter(
        logging.Formatter("%(asctime)s [%(levelname)s] %(message)s", datefmt="%H:%M:%S")
    )
    logger.addHandler(handler)
    return logger


def _add_common_io_args(parser: argparse.ArgumentParser) -> None:
    parser.add_argument("-d", "--delimiter", default=";", help="CSV field delimiter (default: ;)")
    parser.add_argument(
        "--encoding",
        default="utf-8-sig",
        help="Input encoding (default: utf-8-sig to handle BOM)",
    )
    parser.add_argument(
        "--keep-comments",
        action="store_true",
        help="Preserve leading `#` comment rows in the output (default: strip)",
    )
    parser.add_argument(
        "--sanitize-formulas",
        action="store_true",
        help=(
            "Prefix cells starting with = + - @ (or tab/CR) with a single quote to "
            "prevent CSV/formula injection in spreadsheets (default: off, write verbatim)"
        ),
    )


def _build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="csv-tools",
        description="Merge, dedupe, and reorder semicolon-delimited CSVs with optional `#` comment rows.",
    )
    parser.add_argument(
        "--log-level",
        default="INFO",
        choices=["DEBUG", "INFO", "WARNING", "ERROR"],
        help="Logging verbosity",
    )
    sub = parser.add_subparsers(dest="command", required=True)

    # merge
    p_merge = sub.add_parser(
        "merge", help="Concatenate two or more CSVs (header taken from the first)"
    )
    p_merge.add_argument("inputs", type=Path, nargs="+", help="Input CSV files (2 or more)")
    p_merge.add_argument("-o", "--output", type=Path, required=True, help="Output CSV file")
    _add_common_io_args(p_merge)

    # dedupe
    p_dedupe = sub.add_parser(
        "dedupe", help="Remove duplicate rows (by full row, a named column, or a column index)"
    )
    p_dedupe.add_argument("input", type=Path, help="Input CSV file")
    p_dedupe.add_argument("-o", "--output", type=Path, required=True, help="Output CSV file")
    group = p_dedupe.add_mutually_exclusive_group()
    group.add_argument("--column", help="Deduplicate by this column name")
    group.add_argument("--index", type=int, help="Deduplicate by column at this 0-based index")
    _add_common_io_args(p_dedupe)

    # reorder
    p_reorder = sub.add_parser(
        "reorder", help="Move named columns to the front (rest stays in original order)"
    )
    p_reorder.add_argument("input", type=Path, help="Input CSV file")
    p_reorder.add_argument("-o", "--output", type=Path, required=True, help="Output CSV file")
    p_reorder.add_argument(
        "--first",
        action="append",
        required=True,
        help="Column name to pull to the front (repeat for multiple; order matters)",
    )
    _add_common_io_args(p_reorder)

    return parser


def _run_merge(args: argparse.Namespace, logger: logging.Logger) -> int:
    merge_files(
        input_paths=args.inputs,
        output_path=args.output,
        delimiter=args.delimiter,
        encoding=args.encoding,
        write_comments=args.keep_comments,
        sanitize_formulas=args.sanitize_formulas,
        logger=logger,
    )
    return 0


def _run_dedupe(args: argparse.Namespace, logger: logging.Logger) -> int:
    column: str | int | None = args.column if args.column is not None else args.index
    dedupe_by_column(
        input_path=args.input,
        output_path=args.output,
        column=column,
        delimiter=args.delimiter,
        encoding=args.encoding,
        write_comments=args.keep_comments,
        sanitize_formulas=args.sanitize_formulas,
        logger=logger,
    )
    return 0


def _run_reorder(args: argparse.Namespace, logger: logging.Logger) -> int:
    reorder_columns(
        input_path=args.input,
        output_path=args.output,
        first_columns=args.first,
        delimiter=args.delimiter,
        encoding=args.encoding,
        write_comments=args.keep_comments,
        sanitize_formulas=args.sanitize_formulas,
        logger=logger,
    )
    return 0


def main() -> int:
    args = _build_parser().parse_args()
    logger = _setup_logging(args.log_level)

    handlers = {
        "merge": _run_merge,
        "dedupe": _run_dedupe,
        "reorder": _run_reorder,
    }

    try:
        return handlers[args.command](args, logger)
    except FileNotFoundError as e:
        logger.error(str(e))
        return 2
    except ValueError as e:
        logger.error(str(e))
        return 2


if __name__ == "__main__":
    sys.exit(main())
