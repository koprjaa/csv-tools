"""Tests for io.read_csv / io.write_csv."""
from csv_tools.io import ParsedCsv, read_csv, write_csv


def _write(path, text):
    path.write_text(text, encoding="utf-8-sig")


def test_reads_basic_csv(tmp_path):
    csv_path = tmp_path / "in.csv"
    _write(csv_path, "name;age\nAlice;30\nBob;25\n")
    parsed = read_csv(csv_path)
    assert parsed.header == ["name", "age"]
    assert parsed.rows == [["Alice", "30"], ["Bob", "25"]]
    assert parsed.comments == []


def test_strips_leading_comment_rows(tmp_path):
    csv_path = tmp_path / "in.csv"
    _write(csv_path, "# generated 2026-04-17\n# source: ARES\nname;age\nAlice;30\n")
    parsed = read_csv(csv_path)
    assert parsed.header == ["name", "age"]
    assert parsed.rows == [["Alice", "30"]]
    assert len(parsed.comments) == 2
    assert parsed.comments[0][0].startswith("#")


def test_skips_empty_rows_between_data(tmp_path):
    csv_path = tmp_path / "in.csv"
    _write(csv_path, "name;age\nAlice;30\n\nBob;25\n")
    parsed = read_csv(csv_path)
    assert parsed.rows == [["Alice", "30"], ["Bob", "25"]]


def test_round_trip(tmp_path):
    csv_path = tmp_path / "in.csv"
    out_path = tmp_path / "out.csv"
    _write(csv_path, "name;age\nAlice;30\nBob;25\n")
    parsed = read_csv(csv_path)
    write_csv(parsed, out_path)
    reloaded = read_csv(out_path)
    assert reloaded.header == parsed.header
    assert reloaded.rows == parsed.rows


def test_write_csv_with_comments(tmp_path):
    out = tmp_path / "out.csv"
    parsed = ParsedCsv(
        header=["a", "b"],
        rows=[["1", "2"]],
        comments=[["# hello"]],
    )
    write_csv(parsed, out, write_comments=True)
    content = out.read_text(encoding="utf-8")
    assert content.startswith("# hello")


def test_write_csv_strips_comments_by_default(tmp_path):
    out = tmp_path / "out.csv"
    parsed = ParsedCsv(
        header=["a", "b"],
        rows=[["1", "2"]],
        comments=[["# hello"]],
    )
    write_csv(parsed, out)
    content = out.read_text(encoding="utf-8")
    assert "# hello" not in content


def test_handles_bom(tmp_path):
    csv_path = tmp_path / "in.csv"
    # Write with real BOM
    csv_path.write_bytes("\ufeffname;age\nAlice;30\n".encode())
    parsed = read_csv(csv_path)
    assert parsed.header == ["name", "age"]  # BOM stripped


def test_no_header_returns_empty(tmp_path):
    csv_path = tmp_path / "in.csv"
    _write(csv_path, "# just comments\n# all the way\n")
    parsed = read_csv(csv_path)
    assert parsed.header == []
    assert parsed.rows == []
