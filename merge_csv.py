#!/usr/bin/env python3
"""
Script to merge two CSV files with the same structure.
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


def merge_csv_files(file1_path: str, file2_path: str, output_path: str) -> None:
    """
    Merge two CSV files with the same structure into one output file.

    Args:
        file1_path: Path to the first CSV file
        file2_path: Path to the second CSV file
        output_path: Path to the output CSV file
    """
    file1 = Path(file1_path)
    file2 = Path(file2_path)

    if not file1.exists():
        logger.error(f"File {file1_path} does not exist")
        sys.exit(1)

    if not file2.exists():
        logger.error(f"File {file2_path} does not exist")
        sys.exit(1)

    logger.info(f"Reading {file1_path}...")
    with open(file1, 'r', encoding='utf-8-sig') as f1:
        reader1 = csv.reader(f1, delimiter=';')
        rows1 = list(reader1)

    logger.info(f"Reading {file2_path}...")
    with open(file2, 'r', encoding='utf-8-sig') as f2:
        reader2 = csv.reader(f2, delimiter=';')
        rows2 = list(reader2)

    # Find header row (skip comment lines starting with #)
    header1 = None
    header2 = None
    data_start1 = 0
    data_start2 = 0

    for i, row in enumerate(rows1):
        if row and not row[0].startswith('#'):
            header1 = row
            data_start1 = i + 1
            break

    for i, row in enumerate(rows2):
        if row and not row[0].startswith('#'):
            header2 = row
            data_start2 = i + 1
            break

    if header1 != header2:
        logger.warning("Headers differ between files, using header from first file")

    logger.info(f"Writing merged data to {output_path}...")
    with open(output_path, 'w', encoding='utf-8', newline='') as out:
        writer = csv.writer(out, delimiter=';')

        # Write header
        if header1:
            writer.writerow(header1)

        # Write data from first file
        for row in rows1[data_start1:]:
            if row:  # Skip empty rows
                writer.writerow(row)

        # Write data from second file
        for row in rows2[data_start2:]:
            if row:  # Skip empty rows
                writer.writerow(row)

    logger.info(f"Successfully merged {len(rows1) - data_start1} rows from first file "
                f"and {len(rows2) - data_start2} rows from second file")


if __name__ == '__main__':
    merge_csv_files(
        'import_final_1.csv',
        'import_final_2.csv',
        'import_final_merged.csv'
    )

