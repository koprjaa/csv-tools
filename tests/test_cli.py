"""End-to-end CLI tests via subprocess-free main() call."""
import sys

import pytest

from csv_tools.cli import main
from csv_tools.io import read_csv


def _write(path, text):
    path.write_text(text, encoding="utf-8-sig")


def _run(argv, monkeypatch) -> int:
    monkeypatch.setattr(sys, "argv", ["csv-tools", *argv])
    return main()


def test_cli_merge(tmp_path, monkeypatch):
    a = tmp_path / "a.csv"
    b = tmp_path / "b.csv"
    out = tmp_path / "out.csv"
    _write(a, "name\nAlice\n")
    _write(b, "name\nBob\n")
    rc = _run(["merge", str(a), str(b), "-o", str(out)], monkeypatch)
    assert rc == 0
    parsed = read_csv(out)
    assert [r[0] for r in parsed.rows] == ["Alice", "Bob"]


def test_cli_dedupe_by_column(tmp_path, monkeypatch):
    src = tmp_path / "in.csv"
    out = tmp_path / "out.csv"
    _write(src, "id;name\n1;a\n2;b\n1;c\n")
    rc = _run(["dedupe", str(src), "-o", str(out), "--column", "id"], monkeypatch)
    assert rc == 0
    parsed = read_csv(out)
    assert len(parsed.rows) == 2


def test_cli_reorder(tmp_path, monkeypatch):
    src = tmp_path / "in.csv"
    out = tmp_path / "out.csv"
    _write(src, "a;b;c\n1;2;3\n")
    rc = _run(["reorder", str(src), "-o", str(out), "--first", "c"], monkeypatch)
    assert rc == 0
    parsed = read_csv(out)
    assert parsed.header[0] == "c"


def test_cli_missing_input_returns_code_2(tmp_path, monkeypatch):
    out = tmp_path / "out.csv"
    rc = _run(
        ["dedupe", str(tmp_path / "nope.csv"), "-o", str(out), "--column", "x"],
        monkeypatch,
    )
    assert rc == 2


def test_cli_dedupe_mutex_flags(tmp_path, monkeypatch):
    src = tmp_path / "in.csv"
    out = tmp_path / "out.csv"
    _write(src, "a;b\n1;2\n")
    # --column and --index are mutually exclusive
    with pytest.raises(SystemExit):
        _run(
            ["dedupe", str(src), "-o", str(out), "--column", "a", "--index", "0"],
            monkeypatch,
        )
