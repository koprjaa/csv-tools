#!/usr/bin/env python3
"""
Script to remove duplicate rows based on IČ firmy (first column).
"""

import csv
import logging
import sys
from pathlib import Path

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)


def remove_duplicate_ico(input_path: str, output_path: str) -> None:
    """
    Remove duplicate rows based on IČ firmy (first column).
    Keeps the first occurrence of each IČO.

    Args:
        input_path: Path to the input CSV file
        output_path: Path to the output CSV file
    """
    input_file = Path(input_path)

    if not input_file.exists():
        logger.error(f"File {input_path} does not exist")
        sys.exit(1)

    logger.info(f"Reading {input_path}...")
    with open(input_file, 'r', encoding='utf-8-sig') as f:
        reader = csv.reader(f, delimiter=';')
        rows = list(reader)

    # Find header row
    header = None
    data_start = 0

    for i, row in enumerate(rows):
        if row and not row[0].startswith('#'):
            header = row
            data_start = i + 1
            break

    if not header:
        logger.error("Could not find header row")
        sys.exit(1)

    logger.info(f"Processing {len(rows) - data_start} data rows...")

    # Track seen IČO values and keep unique rows
    seen_ico = set()
    unique_rows = []

    duplicates_count = 0

    for row in rows[data_start:]:
        if not row:  # Skip empty rows
            continue

        ico = row[0].strip() if row[0] else ''

        if ico and ico not in seen_ico:
            seen_ico.add(ico)
            unique_rows.append(row)
        elif ico:
            duplicates_count += 1

    logger.info(f"Found {duplicates_count} duplicate rows")
    logger.info(f"Keeping {len(unique_rows)} unique rows")

    logger.info(f"Writing deduplicated data to {output_path}...")
    with open(output_path, 'w', encoding='utf-8', newline='') as out:
        writer = csv.writer(out, delimiter=';')

        # Write header
        writer.writerow(header)

        # Write unique rows
        for row in unique_rows:
            writer.writerow(row)

    logger.info(f"Successfully removed duplicates. Output: {len(unique_rows)} rows")


if __name__ == '__main__':
    remove_duplicate_ico(
        'import_final_merged.csv',
        'import_final_merged.csv'
    )

