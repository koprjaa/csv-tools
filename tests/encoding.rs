// Project: csv-tools
// File: tests/encoding.rs
// Description: Encoding parity regression tests.
// Author: Jan Alexandr Kopřiva
// License: AGPL-3.0-or-later
use csv_tools::read_csv;
use std::{fs, path::Path, process::Command};

#[test]
fn single_byte_python_codec_tables() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("input.csv");
    // Generated using Python's strict bytes.decode for every byte, including C1.
    for line in include_str!("fixtures/python_codecs.txt").lines() {
        let mut fields = line.split_whitespace();
        let encoding = fields.next().unwrap();
        for (byte, expected) in fields.enumerate() {
            fs::write(&path, [b'h', b'\n', b'x', byte as u8, b'y']).unwrap();
            let result = read_csv(&path, ';', encoding);
            if expected == "-" {
                let error = result.unwrap_err();
                assert!(error.contains("position 3"), "{encoding} {byte}: {error}");
                assert!(error.contains(path.to_str().unwrap()));
            } else {
                let c = char::from_u32(u32::from_str_radix(expected, 16).unwrap()).unwrap();
                // Compare to the same parser, so CSV syntax bytes remain meaningful.
                let expected = csv_tools::parse_csv(&format!("h\nx{c}y"), ';').unwrap();
                assert_eq!(result.unwrap(), expected, "{encoding} byte {byte}");
            }
        }
    }
}

#[test]
fn unicode_strict_errors_and_bom() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("input.csv");
    for (encoding, bytes, position) in [
        ("utf-8", &b"h\n\xff"[..], 2),
        ("utf-8", &b"h\n\xe2\x82"[..], 2),
        ("utf-8-sig", &b"\xef\xbb\xbfh\n\xff"[..], 5),
        ("utf-16", &b"\xff\xfeh\0\n\0\0\xd8"[..], 6),
        ("utf-16", &b"\xfe\xff\0h\0\n\xdc\0"[..], 6),
        ("utf-16", &b"\xff\xfeh\0x"[..], 4),
    ] {
        fs::write(&path, bytes).unwrap();
        let error = read_csv(&path, ';', encoding).unwrap_err();
        assert!(error.contains(&format!("position {position}")), "{error}");
    }
    for bytes in [&b"\xff\xfeh\0\n\0x\0"[..], &b"\xfe\xff\0h\0\n\0x"[..]] {
        fs::write(&path, bytes).unwrap();
        assert_eq!(read_csv(&path, ';', "utf-16").unwrap().rows, [vec!["x"]]);
    }
    fs::write(&path, b"h\0").unwrap();
    assert!(read_csv(&path, ';', "utf-16").unwrap_err().contains("BOM"));
}

#[test]
fn mixed_encoding_merge_rejects_without_output() {
    let dir = tempfile::tempdir().unwrap();
    let output = dir.path().join("out.csv");
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let result = Command::new(env!("CARGO_BIN_EXE_csv-tools"))
        .arg("merge")
        .args(
            [
                "parity_utf8_bom.csv",
                "parity_utf8_crlf.csv",
                "parity_cp1250.csv",
            ]
            .map(|f| fixtures.join(f)),
        )
        .arg("-o")
        .arg(&output)
        .args(["--encoding", "cp1250"])
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(2));
    let log = String::from_utf8(result.stdout).unwrap();
    assert!(log.contains("[ERROR]"));
    assert!(log.contains("parity_utf8_bom.csv"));
    assert!(log.contains("position 132"), "{log}");
    assert!(!output.exists());
    assert!(read_csv(&fixtures.join("parity_cp1250.csv"), ';', "cp1250").is_ok());
    for name in ["parity_utf8_bom.csv", "parity_utf8_crlf.csv"] {
        assert!(read_csv(&fixtures.join(name), ';', "utf-8-sig").is_ok());
    }
}
