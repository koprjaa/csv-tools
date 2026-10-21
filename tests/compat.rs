// Project: csv-tools
// File: tests/compat.rs
// Description: Regression equivalents of the original tests and CLI edge cases.
// Author: Jan Alexandr Kopřiva
// License: MIT

use csv_tools::*;
use std::{fs, process::Command};
fn parse(s: &str) -> ParsedCsv {
    parse_csv(s, ';').unwrap()
}
fn roundtrip(p: &ParsedCsv, comments: bool, sanitize: bool) -> String {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nested/out.csv");
    write_csv(p, &path, ';', comments, sanitize).unwrap();
    fs::read_to_string(path).unwrap()
}
#[test]
fn reads_basic_csv() {
    let p = parse("name;age\nAlice;30\nBob;25\n");
    assert_eq!(p.header, ["name", "age"]);
    assert_eq!(p.rows, [vec!["Alice", "30"], vec!["Bob", "25"]]);
    assert!(p.comments.is_empty());
}
#[test]
fn strips_leading_comment_rows() {
    let p = parse("# generated\n# source\nname;age\nAlice;30\n");
    assert_eq!(p.comments.len(), 2);
    assert_eq!(p.header, ["name", "age"]);
    assert_eq!(p.rows, [vec!["Alice", "30"]]);
}
#[test]
fn skips_empty_rows_between_data() {
    assert_eq!(parse("a\nx\n\n \n; \ny\n").rows, [vec!["x"], vec!["y"]]);
}
#[test]
fn round_trip() {
    let p = parse("a;b\nx;1\ny;2\n");
    assert_eq!(parse(&roundtrip(&p, false, false)), p);
}
#[test]
fn write_csv_with_comments() {
    assert!(roundtrip(&parse("# hello\na;b\n1;2"), true, false).starts_with("# hello"));
}
#[test]
fn write_csv_strips_comments_by_default() {
    assert!(!roundtrip(&parse("# hello\na;b\n1;2"), false, false).contains("# hello"));
}
#[test]
fn handles_bom() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("in");
    fs::write(&p, "\u{feff}name;age\nAlice;30").unwrap();
    assert_eq!(
        read_csv(&p, ';', "utf-8-sig").unwrap().header,
        ["name", "age"]
    );
    assert_eq!(
        read_csv(&p, ';', "utf-8").unwrap().header[0],
        "\u{feff}name"
    );
}
#[test]
fn no_header_returns_empty() {
    let p = parse("# just comments\n# all the way");
    assert!(p.header.is_empty());
    assert!(p.rows.is_empty());
}
#[test]
fn sanitize_cell_prefixes_formula_triggers() {
    for c in ['=', '+', '-', '@', '\t', '\r'] {
        assert_eq!(sanitize_cell(&format!("{c}cmd")), format!("'{c}cmd"));
    }
}
#[test]
fn sanitize_cell_leaves_safe_values() {
    for s in ["Alice", "30", ""] {
        assert_eq!(sanitize_cell(s), s);
    }
}
#[test]
fn write_csv_default_does_not_sanitize() {
    assert_eq!(
        parse(&roundtrip(&parse("a\n=SUM(A1:A2)"), false, false)).rows,
        [vec!["=SUM(A1:A2)"]]
    );
}
#[test]
fn write_csv_sanitize_formulas_prefixes_risky_cells() {
    let p = ParsedCsv {
        header: vec!["=danger".into(), "safe".into()],
        rows: vec![
            vec!["=SUM(A1:A2)".into(), "ok".into()],
            vec!["+1".into(), "-2".into()],
        ],
        comments: vec![vec!["=evil".into()]],
    };
    assert_eq!(
        roundtrip(&p, true, true),
        "'=evil\r\n'=danger;safe\r\n'=SUM(A1:A2);ok\r\n'+1;'-2\r\n"
    );
}
#[test]
fn dedupe_by_column_name() {
    let mut p = parse("ico;name\n12345;Acme\n67890;Other\n12345;Duplicate");
    let i = find_column(&p.header, "ico").unwrap();
    assert_eq!(dedupe(&mut p, Some(i)), (2, 1));
    assert_eq!(p.rows, [vec!["12345", "Acme"], vec!["67890", "Other"]]);
}
#[test]
fn dedupe_by_column_index() {
    let mut p = parse("a;b\nx;1\ny;2\nx;3");
    assert_eq!(dedupe(&mut p, Some(0)), (2, 1));
}
#[test]
fn dedupe_full_row() {
    let mut p = parse("a;b\nx;1\nx;1\nx;2");
    assert_eq!(dedupe(&mut p, None), (2, 1));
}
#[test]
fn dedupe_skips_empty_keys() {
    let mut p = parse("ico;name\n;A\n;B\n123;C");
    assert_eq!(dedupe(&mut p, Some(0)), (1, 2));
    assert_eq!(p.rows, [vec!["123", "C"]]);
}
#[test]
fn dedupe_unknown_column_name_raises() {
    assert!(find_column(&parse("a;b").header, "missing")
        .unwrap_err()
        .contains("not found"));
}
#[test]
fn dedupe_index_out_of_range_raises() {
    for i in [-1, 5] {
        assert!(resolve_index(&parse("a;b").header, i)
            .unwrap_err()
            .contains("out of range"));
    }
}
fn cli(inputs: &[(&str, &str)], args: &[&str]) -> (std::process::Output, String) {
    let d = tempfile::tempdir().unwrap();
    for (name, text) in inputs {
        fs::write(d.path().join(name), text).unwrap();
    }
    let o = Command::new(env!("CARGO_BIN_EXE_csv-tools"))
        .current_dir(d.path())
        .args(args)
        .output()
        .unwrap();
    let data = fs::read_to_string(d.path().join("out")).unwrap_or_default();
    (o, data)
}
fn success(inputs: &[(&str, &str)], args: &[&str]) -> String {
    let (o, s) = cli(inputs, args);
    assert!(o.status.success(), "{:?}", o);
    s
}
#[test]
fn dedupe_missing_input_raises() {
    assert_eq!(
        cli(&[], &["dedupe", "missing", "-o", "out"])
            .0
            .status
            .code(),
        Some(2)
    );
}
#[test]
fn merge_two_files() {
    let s = success(
        &[("a", "name;age\nAlice;30"), ("b", "name;age\nBob;25")],
        &["merge", "a", "b", "-o", "out"],
    );
    assert_eq!(parse(&s).rows, [vec!["Alice", "30"], vec!["Bob", "25"]]);
}
#[test]
fn merge_preserves_first_header_on_mismatch() {
    let (o, s) = cli(
        &[("a", "name;age\nAlice;30"), ("b", "firstname;age\nBob;25")],
        &["merge", "a", "b", "-o", "out"],
    );
    assert_eq!(parse(&s).header, ["name", "age"]);
    assert!(String::from_utf8(o.stdout)
        .unwrap()
        .contains("[WARNING] Header mismatch in b: using header from a"));
}
#[test]
fn merge_three_files() {
    let s = success(
        &[
            ("a", "name\nAlice"),
            ("b", "name\nBob"),
            ("c", "name\nCarol"),
        ],
        &["merge", "a", "b", "c", "-o", "out"],
    );
    assert_eq!(parse(&s).rows, [vec!["Alice"], vec!["Bob"], vec!["Carol"]]);
}
#[test]
fn merge_missing_input_raises() {
    assert_eq!(
        cli(
            &[("a", "name\nAlice")],
            &["merge", "a", "missing", "-o", "out"]
        )
        .0
        .status
        .code(),
        Some(2)
    );
}
#[test]
fn merge_empty_list_raises() {
    assert_eq!(cli(&[], &["merge", "-o", "out"]).0.status.code(), Some(2));
}
#[test]
fn reorder_single_column_to_front() {
    let mut p = parse("name;ico;city\nAcme;12345;Praha\nOther;67890;Brno");
    reorder(&mut p, &["ico".into()]).unwrap();
    assert_eq!(p.header, ["ico", "name", "city"]);
    assert_eq!(
        p.rows,
        [
            vec!["12345", "Acme", "Praha"],
            vec!["67890", "Other", "Brno"]
        ]
    );
}
#[test]
fn reorder_multiple_columns_order_matters() {
    let mut p = parse("a;b;c;d\n1;2;3;4");
    reorder(&mut p, &["c".into(), "a".into()]).unwrap();
    assert_eq!(p.header, ["c", "a", "b", "d"]);
    assert_eq!(p.rows, [vec!["3", "1", "2", "4"]]);
}
#[test]
fn reorder_unknown_column_raises() {
    assert!(reorder(&mut parse("a;b\n1;2"), &["missing".into()])
        .unwrap_err()
        .contains("not found"));
}
#[test]
fn reorder_short_row_filled_with_empty() {
    let mut p = parse("a;b;c\n1;2");
    reorder(&mut p, &["c".into()]).unwrap();
    assert_eq!(p.rows, [vec!["", "1", "2"]]);
}
#[test]
fn reorder_missing_input_raises() {
    assert_eq!(
        cli(&[], &["reorder", "missing", "-o", "out", "--first", "x"])
            .0
            .status
            .code(),
        Some(2)
    );
}
#[test]
fn cli_merge() {
    assert_eq!(
        parse(&success(
            &[("a", "name\nAlice"), ("b", "name\nBob")],
            &["merge", "a", "b", "-o", "out"]
        ))
        .rows,
        [vec!["Alice"], vec!["Bob"]]
    );
}
#[test]
fn cli_dedupe_by_column() {
    assert_eq!(
        parse(&success(
            &[("in", "id;name\n1;a\n2;b\n1;c")],
            &["dedupe", "in", "-o", "out", "--column", "id"]
        ))
        .rows
        .len(),
        2
    );
}
#[test]
fn cli_reorder() {
    assert_eq!(
        parse(&success(
            &[("in", "a;b;c\n1;2;3")],
            &["reorder", "in", "-o", "out", "--first", "c"]
        ))
        .header[0],
        "c"
    );
}
#[test]
fn cli_missing_input_returns_code_2() {
    let (o, _) = cli(&[], &["dedupe", "nope.csv", "-o", "out", "--column", "x"]);
    assert_eq!(o.status.code(), Some(2));
    assert!(String::from_utf8(o.stdout)
        .unwrap()
        .contains("[ERROR] Input file not found: nope.csv"));
}
#[test]
fn cli_dedupe_mutex_flags() {
    assert_eq!(
        cli(
            &[("in", "a;b\n1;2")],
            &["dedupe", "in", "-o", "out", "--column", "a", "--index", "0"]
        )
        .0
        .status
        .code(),
        Some(2)
    );
}
#[test]
fn quoting_blank_records_and_comments_inside_data() {
    let p = parse(
        "\n 
 # comment\n\"a;b\";c\r\n\"x\r\ny\";\"q\"\"z\"\r\n# data;v\n;\n",
    );
    assert_eq!(p.comments.len(), 3);
    assert_eq!(p.header, ["a;b", "c"]);
    assert_eq!(p.rows, [vec!["x\r\ny", "q\"z"], vec!["# data", "v"]]);
    assert_eq!(parse(&roundtrip(&p, true, false)), p);
}
#[test]
fn repeated_first_and_long_rows() {
    let mut p = parse("a;b\nx;y;z");
    reorder(&mut p, &["b".into(), "b".into()]).unwrap();
    assert_eq!(p.header, ["b", "a"]);
    assert_eq!(p.rows, [vec!["y", "x"]]);
}
#[test]
fn whitespace_keys_and_missing_cells() {
    let mut p = parse("a;b\nx; 1 \ny;1\nz\nw; ");
    assert_eq!(dedupe(&mut p, Some(1)), (1, 3));
}
#[test]
fn encoding_and_output_bytes() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("in");
    fs::write(&p, b"name\n\xe9").unwrap();
    let parsed = read_csv(&p, ';', "windows-1252").unwrap();
    assert_eq!(parsed.rows, [vec!["é"]]);
    assert_eq!(
        roundtrip(&parsed, false, false).as_bytes(),
        b"name\r\n\xc3\xa9\r\n"
    );
    fs::write(&p, b"\xff").unwrap();
    assert!(read_csv(&p, ';', "utf-8").is_err());
    assert!(read_csv(&p, ';', "unknown").is_err());
}
#[test]
fn cli_flags_and_logging() {
    let (o, s) = cli(
        &[("in", "# hi\na,b\n=1,x")],
        &[
            "--log-level",
            "ERROR",
            "dedupe",
            "in",
            "-o",
            "out",
            "-d",
            ",",
            "--keep-comments",
            "--sanitize-formulas",
        ],
    );
    assert!(o.status.success());
    assert!(o.stdout.is_empty());
    assert_eq!(s, "# hi\r\na,b\r\n'=1,x\r\n");
    let (o, _) = cli(&[("in", "a\nx")], &["dedupe", "in", "-o", "out"]);
    let log = String::from_utf8(o.stdout).unwrap();
    assert_eq!(&log[2..3], ":");
    assert_eq!(&log[5..6], ":");
    assert!(log.contains("[INFO] Deduping 1 rows by full row"));
}
#[test]
fn no_header_errors_and_single_file_merge() {
    for args in [
        vec!["dedupe", "in", "-o", "out"],
        vec!["reorder", "in", "-o", "out", "--first", "a"],
        vec!["merge", "in", "-o", "out"],
    ] {
        assert_eq!(cli(&[("in", "# only")], &args).0.status.code(), Some(2));
    }
    assert_eq!(
        success(&[("in", "a\nx")], &["merge", "in", "-o", "out"]),
        "a\r\nx\r\n"
    );
}
#[test]
fn unicode_delimiter_and_permissive_quotes() {
    let p = parse_csv("a💠b\n\"x\"tail💠\"unclosed", '💠').unwrap();
    assert_eq!(p.rows, [vec!["xtail", "unclosed"]]);
    let d = tempfile::tempdir().unwrap();
    let f = d.path().join("out");
    write_csv(&p, &f, '💠', false, false).unwrap();
    assert_eq!(parse_csv(&fs::read_to_string(f).unwrap(), '💠').unwrap(), p);
}

#[test]
fn field_limit_and_python_whitespace() {
    assert!(parse_csv(&format!("a\n{}", "x".repeat(131072)), ';').is_ok());
    assert!(parse_csv(&format!("a\n{}", "x".repeat(131073)), ';').is_err());
    let mut p = parse("a;b\nx;\u{1c}1\u{1f}\ny;1");
    assert_eq!(dedupe(&mut p, Some(1)), (1, 1));
}
#[test]
fn merge_matches_reordered_columns_by_name() {
    let s = success(
        &[("a", "name;age\nAlice;30"), ("b", "age;name\n25;Bob")],
        &["merge", "a", "b", "-o", "out"],
    );
    assert_eq!(parse(&s).rows, [vec!["Alice", "30"], vec!["Bob", "25"]]);
}
#[test]
fn reorder_ignores_a_repeated_first_column() {
    let s = success(
        &[("in", "a;b;c\n1;2;3")],
        &["reorder", "in", "--first", "b", "--first", "b", "-o", "out"],
    );
    let p = parse(&s);
    assert_eq!(p.header, ["b", "a", "c"]);
    assert_eq!(p.rows, [vec!["2", "1", "3"]]);
}
#[test]
fn dedupe_warns_about_rows_with_an_empty_key() {
    let (o, s) = cli(
        &[("in", "id;v\n;x\n1;y")],
        &["dedupe", "in", "--column", "id", "-o", "out"],
    );
    assert!(String::from_utf8(o.stdout)
        .unwrap()
        .contains("[WARNING] 1 rows with an empty key are dropped"));
    assert_eq!(parse(&s).rows, [vec!["1", "y"]]);
}
