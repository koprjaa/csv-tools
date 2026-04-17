"""Tests for merge_files."""
import pytest

from csv_tools.io import read_csv
from csv_tools.merge import merge_files


def _write(path, text):
    path.write_text(text, encoding="utf-8-sig")


def test_merge_two_files(tmp_path):
    a = tmp_path / "a.csv"
    b = tmp_path / "b.csv"
    out = tmp_path / "out.csv"
    _write(a, "name;age\nAlice;30\n")
    _write(b, "name;age\nBob;25\n")

    count = merge_files([a, b], out)
    assert count == 2

    parsed = read_csv(out)
    assert parsed.header == ["name", "age"]
    assert parsed.rows == [["Alice", "30"], ["Bob", "25"]]


def test_merge_preserves_first_header_on_mismatch(tmp_path):
    a = tmp_path / "a.csv"
    b = tmp_path / "b.csv"
    out = tmp_path / "out.csv"
    _write(a, "name;age\nAlice;30\n")
    _write(b, "firstname;age\nBob;25\n")  # different header

    merge_files([a, b], out)
    parsed = read_csv(out)
    assert parsed.header == ["name", "age"]  # from first file


def test_merge_three_files(tmp_path):
    files = []
    for i, val in enumerate(["Alice", "Bob", "Carol"]):
        p = tmp_path / f"{i}.csv"
        _write(p, f"name\n{val}\n")
        files.append(p)
    out = tmp_path / "out.csv"
    merge_files(files, out)
    parsed = read_csv(out)
    assert [r[0] for r in parsed.rows] == ["Alice", "Bob", "Carol"]


def test_merge_missing_input_raises(tmp_path):
    a = tmp_path / "a.csv"
    _write(a, "name\nAlice\n")
    with pytest.raises(FileNotFoundError):
        merge_files([a, tmp_path / "missing.csv"], tmp_path / "out.csv")


def test_merge_empty_list_raises(tmp_path):
    with pytest.raises(ValueError, match="at least one"):
        merge_files([], tmp_path / "out.csv")
