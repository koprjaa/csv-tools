#!/usr/bin/env python3
"""
Script to reorder CSV columns so that IČ firmy is first.
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


def reorder_csv_columns(input_path: str, output_path: str) -> None:
    """
    Reorder CSV columns so that IČ firmy is the first column.

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

    # Find index of IČ firmy column
    try:
        ico_index = header.index('IČ firmy')
    except ValueError:
        logger.error("Column 'IČ firmy' not found in header")
        sys.exit(1)

    # Create new column order: IČ firmy first, then rest
    new_header = ['IČ firmy'] + [col for i, col in enumerate(header) if i != ico_index]

    logger.info(f"Reordering columns: {new_header}")

    logger.info(f"Writing reordered data to {output_path}...")
    with open(output_path, 'w', encoding='utf-8', newline='') as out:
        writer = csv.writer(out, delimiter=';')

        # Write new header
        writer.writerow(new_header)

        # Write reordered data rows
        for row in rows[data_start:]:
            if row:  # Skip empty rows
                # Reorder columns: IČ firmy first, then rest
                ico_value = row[ico_index] if ico_index < len(row) else ''
                rest_values = [row[i] if i < len(row) else '' for i in range(len(header)) if i != ico_index]
                new_row = [ico_value] + rest_values
                writer.writerow(new_row)

    logger.info(f"Successfully reordered {len(rows) - data_start} rows")


if __name__ == '__main__':
    reorder_csv_columns(
        'import_final_merged.csv',
        'import_final_merged.csv'
    )

