// Project: csv-tools
// File: src/lib.rs
// Description: CSV compatibility and transformations.
// Author: Jan Alexandr Kopřiva
// License: AGPL-3.0-or-later
use std::{
    collections::HashSet,
    fs,
    io::{BufWriter, Write},
    path::Path,
};
pub type Result<T> = std::result::Result<T, String>;
pub type Row = Vec<String>;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct ParsedCsv {
    pub header: Row,
    pub rows: Vec<Row>,
    pub comments: Vec<Row>,
}

pub fn sanitize_cell(value: &str) -> String {
    if value.starts_with(['=', '+', '-', '@', '\t', '\r']) {
        format!("'{value}")
    } else {
        value.to_owned()
    }
}

pub fn read_csv(path: &Path, delimiter: char, encoding: &str) -> Result<ParsedCsv> {
    let bytes = fs::read(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            format!("Input file not found: {}", path.display())
        } else {
            e.to_string()
        }
    })?;
    let label = encoding
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
        .to_ascii_lowercase();
    let ascii = matches!(
        label.as_str(),
        "ascii"
            | "us-ascii"
            | "646"
            | "ansi-x3-4-1968"
            | "ansi-x3-4-1986"
            | "cp367"
            | "csascii"
            | "ibm367"
            | "iso646-us"
            | "iso-646-irv-1991"
            | "iso-ir-6"
            | "us"
    );
    let latin1 = matches!(
        label.as_str(),
        "latin1"
            | "latin-1"
            | "iso-8859-1"
            | "iso8859-1"
            | "l1"
            | "8859"
            | "cp819"
            | "csisolatin1"
            | "ibm819"
            | "iso8859"
            | "iso-8859-1-1987"
            | "iso-ir-100"
            | "latin"
    );
    // WHATWG aliases these codecs to Windows-1252, which changes valid data.
    if ascii || latin1 {
        if ascii {
            if let Some(position) = bytes.iter().position(|b| !b.is_ascii()) {
                return Err(format!(
                    "Cannot decode {} as {encoding}: invalid byte at position {position}",
                    path.display()
                ));
            }
        }
        let text: String = bytes.iter().map(|&b| char::from(b)).collect();
        return parse_csv(&text, delimiter);
    }
    let label = match label.as_str() {
        "utf16" => "utf-16",
        "utf8" | "cp65001" => "utf-8",
        "utf16le" | "utf-16-le" => "utf-16le",
        "utf16be" | "utf-16-be" => "utf-16be",
        _ => &label,
    };
    let sig = label == "utf-8-sig";
    let mut enc = encoding_rs::Encoding::for_label(if sig { b"utf-8" } else { label.as_bytes() })
        .ok_or_else(|| format!("unknown encoding: {encoding}"))?;
    let error = |position: usize| {
        format!(
            "Cannot decode {} as {encoding}: invalid byte at position {position}",
            path.display()
        )
    };
    // WHATWG accepts some entries that Python's strict charmaps leave undefined.
    let undefined: &[u8] = match enc.name() {
        "windows-1250" => &[129, 131, 136, 144, 152],
        "windows-1251" => &[152],
        "windows-1252" => &[129, 141, 143, 144, 157],
        "windows-1253" => &[
            129, 136, 138, 140, 141, 142, 143, 144, 152, 154, 156, 157, 158, 159, 170, 210, 255,
        ],
        "windows-1254" => &[129, 141, 142, 143, 144, 157, 158],
        "windows-1255" => &[
            129, 138, 140, 141, 142, 143, 144, 154, 156, 157, 158, 159, 202, 217, 218, 219, 220,
            221, 222, 223, 251, 252, 255,
        ],
        "windows-1256" => &[],
        "windows-1257" => &[129, 131, 136, 138, 140, 144, 152, 154, 156, 159, 161, 165],
        "windows-1258" => &[129, 138, 141, 142, 143, 144, 154, 157, 158],
        "windows-874" => &[
            129, 130, 131, 132, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 152, 153,
            154, 155, 156, 157, 158, 159, 219, 220, 221, 222, 252, 253, 254, 255,
        ],
        "ISO-8859-3" => &[165, 174, 190, 195, 208, 227, 240],
        "ISO-8859-6" => &[
            161, 162, 163, 165, 166, 167, 168, 169, 170, 171, 174, 175, 176, 177, 178, 179, 180,
            181, 182, 183, 184, 185, 186, 188, 189, 190, 192, 219, 220, 221, 222, 223, 243, 244,
            245, 246, 247, 248, 249, 250, 251, 252, 253, 254, 255,
        ],
        "ISO-8859-7" => &[174, 210, 255],
        "ISO-8859-8" | "ISO-8859-8-I" => &[
            161, 191, 192, 193, 194, 195, 196, 197, 198, 199, 200, 201, 202, 203, 204, 205, 206,
            207, 208, 209, 210, 211, 212, 213, 214, 215, 216, 217, 218, 219, 220, 221, 222, 251,
            252, 255,
        ],
        _ => &[],
    };
    if let Some(position) = bytes.iter().position(|b| undefined.contains(b)) {
        return Err(error(position));
    }
    let mut offset = 0;
    if sig && bytes.starts_with(b"\xef\xbb\xbf") {
        offset = 3;
    }
    if label == "utf-16" || label == "utf16" {
        if bytes.starts_with(b"\xff\xfe") {
            enc = encoding_rs::UTF_16LE;
            offset = 2;
        } else if bytes.starts_with(b"\xfe\xff") {
            enc = encoding_rs::UTF_16BE;
            offset = 2;
        } else if !bytes.is_empty() {
            return Err(format!(
                "{}: UTF-16 stream does not start with BOM (byte position 0)",
                path.display()
            ));
        }
    }
    let mut decoder = enc.new_decoder_without_bom_handling();
    let mut text = String::with_capacity(bytes.len().saturating_mul(4));
    let (result, read) =
        decoder.decode_to_string_without_replacement(&bytes[offset..], &mut text, true);
    match result {
        encoding_rs::DecoderResult::InputEmpty => {}
        encoding_rs::DecoderResult::Malformed(length, after) => {
            return Err(error(
                offset + read - usize::from(length) - usize::from(after),
            ));
        }
        encoding_rs::DecoderResult::OutputFull => return Err("Decoded input is too large".into()),
    }
    parse_csv(&text, delimiter)
}

// Python also treats the four ASCII information separators as whitespace.
pub fn python_trim(s: &str) -> &str {
    s.trim_matches(|c: char| c.is_whitespace() || ('\u{1c}'..='\u{1f}').contains(&c))
}

// A small state machine preserves Python's permissive quotes, blank records,
// Unicode delimiters, and CR/LF inside quoted fields.
pub fn parse_csv(text: &str, delimiter: char) -> Result<ParsedCsv> {
    let mut chars = text.chars().peekable();
    let mut records = Vec::new();
    let mut row = Vec::new();
    let mut cell = String::new();
    let mut field_chars = 0usize;
    let mut quoted = false;
    let mut after_quote = false;
    let mut start = true;
    let mut active = false;
    while let Some(c) = chars.next() {
        let was_after_quote = std::mem::take(&mut after_quote);
        if quoted {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    cell.push('"');
                    field_chars += 1;
                } else {
                    quoted = false;
                    after_quote = true;
                }
            } else {
                cell.push(c);
                field_chars += 1;
            }
        } else if c == '"' && start {
            quoted = true;
            start = false;
            active = true;
        } else if c == delimiter && (!matches!(c, '\r' | '\n') || was_after_quote) {
            row.push(std::mem::take(&mut cell));
            field_chars = 0;
            start = true;
            active = true;
            // TextIO ends the physical line even when its terminator is a delimiter.
            if matches!(c, '\r' | '\n') {
                row.push(String::new());
                records.push(std::mem::take(&mut row));
                active = false;
                if c == '\r' && chars.peek() == Some(&'\n') {
                    chars.next();
                }
            }
        } else if c == '\r' || c == '\n' {
            if c == '\r' && chars.peek() == Some(&'\n') {
                chars.next();
            }
            if active {
                row.push(std::mem::take(&mut cell));
            }
            field_chars = 0;
            records.push(std::mem::take(&mut row));
            start = true;
            active = false;
        } else {
            cell.push(c);
            field_chars += 1;
            start = false;
            active = true;
        }
        if field_chars > 131072 {
            return Err("field larger than field limit (131072)".into());
        }
    }
    if active {
        row.push(cell);
        records.push(row);
    }
    let mut parsed = ParsedCsv::default();
    for row in records {
        if parsed.header.is_empty() {
            let first = row.first().map(|s| python_trim(s)).unwrap_or("");
            if row.is_empty() || first.starts_with('#') || (first.is_empty() && row.len() == 1) {
                parsed.comments.push(row);
            } else {
                parsed.header = row;
            }
        } else if row.iter().any(|c| !python_trim(c).is_empty()) {
            parsed.rows.push(row);
        }
    }
    Ok(parsed)
}

pub fn write_csv(
    parsed: &ParsedCsv,
    path: &Path,
    delimiter: char,
    comments: bool,
    sanitize: bool,
) -> Result<()> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut out = BufWriter::new(fs::File::create(path).map_err(|e| e.to_string())?);
    let rows = parsed
        .comments
        .iter()
        .filter(|_| comments)
        .chain(std::iter::once(&parsed.header).filter(|r| !r.is_empty()))
        .chain(parsed.rows.iter());
    for row in rows {
        for (i, value) in row.iter().enumerate() {
            if i > 0 {
                write!(out, "{delimiter}").map_err(|e| e.to_string())?;
            }
            let value = if sanitize {
                sanitize_cell(value)
            } else {
                value.clone()
            };
            if value.contains([delimiter, '"', '\r', '\n']) || (row.len() == 1 && value.is_empty())
            {
                write!(out, "\"{}\"", value.replace('"', "\"\"")).map_err(|e| e.to_string())?;
            } else {
                out.write_all(value.as_bytes()).map_err(|e| e.to_string())?;
            }
        }
        out.write_all(b"\r\n").map_err(|e| e.to_string())?;
    }
    out.flush().map_err(|e| e.to_string())
}

pub fn repr(s: &str) -> String {
    let quote = if s.contains('\'') && !s.contains('"') {
        '"'
    } else {
        '\''
    };
    let mut out = String::from(quote);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == quote => {
                out.push('\\');
                out.push(c);
            }
            c if c.is_control() && (c as u32) <= 255 => {
                out.push_str(&format!("\\x{:02x}", c as u32))
            }
            c => out.push(c),
        }
    }
    out.push(quote);
    out
}
pub fn repr_row(row: &Row) -> String {
    format!(
        "[{}]",
        row.iter().map(|s| repr(s)).collect::<Vec<_>>().join(", ")
    )
}
pub fn find_column(header: &Row, name: &str) -> Result<usize> {
    header.iter().position(|c| c == name).ok_or_else(|| {
        format!(
            "Column {} not found in header: {}",
            repr(name),
            repr_row(header)
        )
    })
}
pub fn resolve_index(header: &Row, index: i64) -> Result<usize> {
    if index < 0 || index as usize >= header.len() {
        Err(format!(
            "Column index {index} out of range (header has {} columns)",
            header.len()
        ))
    } else {
        Ok(index as usize)
    }
}
pub fn dedupe(parsed: &mut ParsedCsv, index: Option<usize>) -> (usize, usize) {
    let count = parsed.rows.len();
    let mut seen = HashSet::new();
    parsed.rows.retain(|row| {
        let key = if let Some(i) = index {
            let value = row.get(i).map(|s| python_trim(s)).unwrap_or("");
            if value.is_empty() {
                return false;
            }
            vec![value.to_owned()]
        } else {
            row.clone()
        };
        seen.insert(key)
    });
    (parsed.rows.len(), count - parsed.rows.len())
}
pub fn reorder(parsed: &mut ParsedCsv, first: &[String]) -> Result<()> {
    let mut order = Vec::new();
    for c in first {
        let i = find_column(&parsed.header, c)?;
        if !order.contains(&i) {
            order.push(i);
        }
    }
    let rest = (0..parsed.header.len())
        .filter(|i| !order.contains(i))
        .collect::<Vec<_>>();
    order.extend(rest);
    parsed.header = order.iter().map(|&i| parsed.header[i].clone()).collect();
    for row in &mut parsed.rows {
        *row = order
            .iter()
            .map(|&i| row.get(i).cloned().unwrap_or_default())
            .collect();
    }
    Ok(())
}
