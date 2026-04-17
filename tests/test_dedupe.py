"""Tests for dedupe_by_column."""
import pytest

from csv_tools.dedupe import dedupe_by_column
from csv_tools.io import read_csv


def _write(path, text):
    path.write_text(text, encoding="utf-8-sig")


def test_dedupe_by_column_name(tmp_path):
    src = tmp_path / "in.csv"
    out = tmp_path / "out.csv"
    _write(src, "ico;name\n12345;Acme\n67890;Other\n12345;Acme Duplicate\n")
    unique, dups = dedupe_by_column(src, out, column="ico")
    assert unique == 2
    assert dups == 1
    parsed = read_csv(out)
    assert parsed.rows == [["12345", "Acme"], ["67890", "Other"]]


def test_dedupe_by_column_index(tmp_path):
    src = tmp_path / "in.csv"
    out = tmp_path / "out.csv"
    _write(src, "a;b\nx;1\ny;2\nx;3\n")
    unique, dups = dedupe_by_column(src, out, column=0)
    assert unique == 2
    assert dups == 1


def test_dedupe_full_row(tmp_path):
    src = tmp_path / "in.csv"
    out = tmp_path / "out.csv"
    _write(src, "a;b\nx;1\nx;1\nx;2\n")
    unique, dups = dedupe_by_column(src, out)  # column=None → full row
    assert unique == 2
    assert dups == 1


def test_dedupe_skips_empty_keys(tmp_path):
    src = tmp_path / "in.csv"
    out = tmp_path / "out.csv"
    _write(src, "ico;name\n;A\n;B\n123;C\n")
    unique, dups = dedupe_by_column(src, out, column="ico")
    # Both empty-ico rows should be dropped as duplicates
    assert unique == 1
    assert dups == 2
    parsed = read_csv(out)
    assert parsed.rows == [["123", "C"]]


def test_dedupe_unknown_column_name_raises(tmp_path):
    src = tmp_path / "in.csv"
    out = tmp_path / "out.csv"
    _write(src, "a;b\n1;2\n")
    with pytest.raises(ValueError, match="not found"):
        dedupe_by_column(src, out, column="missing")


def test_dedupe_index_out_of_range_raises(tmp_path):
    src = tmp_path / "in.csv"
    out = tmp_path / "out.csv"
    _write(src, "a;b\n1;2\n")
    with pytest.raises(ValueError, match="out of range"):
        dedupe_by_column(src, out, column=5)


def test_dedupe_missing_input_raises(tmp_path):
    with pytest.raises(FileNotFoundError):
        dedupe_by_column(tmp_path / "nope.csv", tmp_path / "out.csv")
