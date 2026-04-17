"""Tests for reorder_columns."""
import pytest

from csv_tools.io import read_csv
from csv_tools.reorder import reorder_columns


def _write(path, text):
    path.write_text(text, encoding="utf-8-sig")


def test_reorder_single_column_to_front(tmp_path):
    src = tmp_path / "in.csv"
    out = tmp_path / "out.csv"
    _write(src, "name;ico;city\nAcme;12345;Praha\nOther;67890;Brno\n")

    reorder_columns(src, out, first_columns=["ico"])
    parsed = read_csv(out)
    assert parsed.header == ["ico", "name", "city"]
    assert parsed.rows == [["12345", "Acme", "Praha"], ["67890", "Other", "Brno"]]


def test_reorder_multiple_columns_order_matters(tmp_path):
    src = tmp_path / "in.csv"
    out = tmp_path / "out.csv"
    _write(src, "a;b;c;d\n1;2;3;4\n")
    reorder_columns(src, out, first_columns=["c", "a"])
    parsed = read_csv(out)
    assert parsed.header == ["c", "a", "b", "d"]
    assert parsed.rows == [["3", "1", "2", "4"]]


def test_reorder_unknown_column_raises(tmp_path):
    src = tmp_path / "in.csv"
    out = tmp_path / "out.csv"
    _write(src, "a;b\n1;2\n")
    with pytest.raises(ValueError, match="not found"):
        reorder_columns(src, out, first_columns=["missing"])


def test_reorder_short_row_filled_with_empty(tmp_path):
    """Defensively fill missing cells when a data row is shorter than the header."""
    src = tmp_path / "in.csv"
    src.write_text("a;b;c\n1;2\n", encoding="utf-8-sig")
    out = tmp_path / "out.csv"
    reorder_columns(src, out, first_columns=["c"])
    parsed = read_csv(out)
    assert parsed.header == ["c", "a", "b"]
    assert parsed.rows == [["", "1", "2"]]


def test_reorder_missing_input_raises(tmp_path):
    with pytest.raises(FileNotFoundError):
        reorder_columns(tmp_path / "nope.csv", tmp_path / "out.csv", first_columns=["x"])
