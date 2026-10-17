// Project: csv-tools
// File: tests/audit.rs
// Description: Differential fixtures and adversarial CLI regressions.
// Based on csv-tools by Jan Alexandr Kopřiva
// Author: Jan Alexandr Kopřiva
// License: MIT

use csv_tools::*;
use std::{fs, process::Command};

fn unhex(s: &str) -> Vec<u8> {
    s.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}

#[test]
fn python_generated_differential_cases() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input");
    let output = dir.path().join("output");
    for (n, line) in include_str!("fixtures/differential.tsv")
        .lines()
        .enumerate()
    {
        let fields: Vec<_> = line.split('\t').collect();
        fs::write(&input, unhex(fields[0])).unwrap();
        let delimiter = String::from_utf8(unhex(fields[1]))
            .unwrap()
            .chars()
            .next()
            .unwrap();
        let mut parsed = read_csv(&input, delimiter, "utf-8").unwrap();
        dedupe(&mut parsed, None);
        write_csv(&parsed, &output, delimiter, true, false).unwrap();
        assert_eq!(fs::read(&output).unwrap(), unhex(fields[2]), "case {n}");
    }
}

#[test]
fn strict_ascii_and_real_latin1() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input");
    for b in 128..=255u8 {
        fs::write(&input, [b'h', b'\n', b'x', b, b'y']).unwrap();
        for enc in ["ascii", "us-ascii", "646"] {
            assert!(read_csv(&input, ';', enc)
                .unwrap_err()
                .contains("position 3"));
        }
        for enc in ["latin1", "latin-1", "ISO-8859-1", "iso8859-1", "l1"] {
            assert_eq!(
                read_csv(&input, ';', enc).unwrap().rows,
                [vec![format!("x{}y", char::from(b))]]
            );
        }
    }
}

#[test]
fn normalized_encoding_aliases() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input");
    fs::write(&input, b"\xef\xbb\xbfh\nx").unwrap();
    for enc in ["UTF 8 SIG", "utf_8_sig", "UTF---8---SIG"] {
        assert_eq!(read_csv(&input, ';', enc).unwrap().header, ["h"]);
    }
    fs::write(&input, b"\xff\xfeh\0\n\0x\0").unwrap();
    assert_eq!(read_csv(&input, ';', "utf16").unwrap().header, ["h"]);
    assert_eq!(
        read_csv(&input, ';', "utf_16_le").unwrap().header,
        ["\u{feff}h"]
    );
}

#[test]
fn newline_delimiters_and_quote_state() {
    for delimiter in ['\n', '\r'] {
        assert_eq!(
            parse_csv("a\nb\r\nc\r", delimiter).unwrap().rows,
            [vec!["b"], vec!["c"]]
        );
        let text = format!("a\n\"x\"{delimiter}");
        assert_eq!(parse_csv(&text, delimiter).unwrap().rows, [vec!["x", ""]]);
    }
}

#[test]
fn repeated_options_and_python_integer_syntax() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("input"), "# hi\na;b\nx;1\nx;2\ny;1\n").unwrap();
    let run = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_csv-tools"))
            .current_dir(dir.path())
            .args(args)
            .output()
            .unwrap()
    };
    let out = run(&[
        "dedupe",
        "input",
        "-o",
        "unused",
        "-o",
        "out",
        "--column",
        "a",
        "--column",
        "b",
        "--keep-comments",
        "--keep-comments",
    ]);
    assert!(out.status.success(), "{out:?}");
    assert_eq!(
        fs::read_to_string(dir.path().join("out")).unwrap(),
        "# hi\r\na;b\r\nx;1\r\nx;2\r\n"
    );
    assert!(!dir.path().join("unused").exists());
    for index in [" 0 ", "0_0", "٠", "+０", "-0"] {
        let out = run(&["dedupe", "input", "-o", "out", "--index", index]);
        assert!(out.status.success(), "{index}: {out:?}");
        assert_eq!(
            fs::read_to_string(dir.path().join("out")).unwrap(),
            "a;b\r\nx;1\r\ny;1\r\n"
        );
    }
    let huge = "9".repeat(100);
    let out = run(&["dedupe", "input", "-o", "out", "--index", &huge]);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8(out.stdout)
        .unwrap()
        .contains(&format!("Column index {huge} out of range")));
    for index in ["_0", "0_", "0__0", "", "²"] {
        assert_eq!(
            run(&["dedupe", "input", "-o", "out", "--index", index])
                .status
                .code(),
            Some(2)
        );
    }
}

#[cfg(unix)]
#[test]
fn closed_logging_pipe_does_not_panic() {
    use std::{
        os::{fd::OwnedFd, unix::net::UnixStream},
        process::Stdio,
    };
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("input"), "a\nx\n").unwrap();
    let (writer, reader) = UnixStream::pair().unwrap();
    drop(reader);
    let fd: OwnedFd = writer.into();
    let out = Command::new(env!("CARGO_BIN_EXE_csv-tools"))
        .current_dir(dir.path())
        .args(["dedupe", "input", "-o", "out"])
        .stdout(Stdio::from(fd))
        .output()
        .unwrap();
    assert!(out.status.success(), "{out:?}");
    assert!(out.stderr.is_empty());
    assert_eq!(
        fs::read_to_string(dir.path().join("out")).unwrap(),
        "a\r\nx\r\n"
    );
}

#[test]
fn invalid_inputs_do_not_truncate_existing_output() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("out");
    for bytes in [
        b"h\n\xff".to_vec(),
        format!("h\n{}", "é".repeat(131073)).into_bytes(),
        b"# comments only".to_vec(),
    ] {
        fs::write(dir.path().join("input"), bytes).unwrap();
        for args in [
            vec!["merge", "input", "-o", "out"],
            vec!["dedupe", "input", "-o", "out"],
            vec!["reorder", "input", "-o", "out", "--first", "h"],
        ] {
            fs::write(&output, "keep").unwrap();
            let result = Command::new(env!("CARGO_BIN_EXE_csv-tools"))
                .current_dir(dir.path())
                .args(args)
                .output()
                .unwrap();
            assert_eq!(result.status.code(), Some(2), "{result:?}");
            assert!(result.stderr.is_empty());
            assert_eq!(fs::read_to_string(&output).unwrap(), "keep");
        }
    }
}
