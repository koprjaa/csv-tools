# CSV Tools

Python scripts for merging, deduplicating, and reordering semicolon-delimited CSV files.

## Features

- **merge_csv.py** -- Merge two CSV files with the same structure into one, preserving headers
- **remove_duplicates.py** -- Remove duplicate rows based on the first column (IC firmy)
- **reorder_csv.py** -- Reorder columns so that "IC firmy" becomes the first column
- Handles BOM-encoded files (`utf-8-sig`) and comment lines starting with `#`

## Installation

No dependencies required -- uses only Python standard library (`csv`, `pathlib`, `logging`).

```bash
python3 merge_csv.py
python3 remove_duplicates.py
python3 reorder_csv.py
```

## Usage

Edit the `__main__` block in each script to set input/output file paths, then run:

```bash
# 1. Merge two CSV files
python3 merge_csv.py

# 2. Remove duplicates from merged file
python3 remove_duplicates.py

# 3. Reorder columns
python3 reorder_csv.py
```

Default pipeline: `import_final_1.csv` + `import_final_2.csv` -> `import_final_merged.csv` (merged, deduplicated, reordered).
