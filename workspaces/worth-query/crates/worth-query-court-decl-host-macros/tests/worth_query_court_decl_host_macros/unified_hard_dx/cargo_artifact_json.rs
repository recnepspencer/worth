//! Structural RFC 8259 JSON parser for Cargo `--message-format=json` lines,
//! extracting `compiler-artifact` `.rlib` paths without bracket/substring hacks.
use std::path::PathBuf;

enum JsonValue {
    Null,
    Bool,
    Number,
    Str(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}

impl JsonValue {
    fn as_str(&self) -> Option<&str> {
        match self {
            Self::Str(s) => Some(s.as_str()),
            _ => None,
        }
    }

    fn as_array(&self) -> Option<&[JsonValue]> {
        match self {
            Self::Array(items) => Some(items.as_slice()),
            _ => None,
        }
    }

    fn field(&self, key: &str) -> Option<&JsonValue> {
        match self {
            Self::Object(entries) => entries.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }
}

struct JsonParser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> JsonParser<'a> {
    fn new(input: &'a str) -> Self {
        Self {
            bytes: input.as_bytes(),
            pos: 0,
        }
    }

    fn skip_ws(&mut self) {
        while let Some(&b) = self.bytes.get(self.pos) {
            if matches!(b, b' ' | b'\t' | b'\r' | b'\n') {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn expect(&mut self, expected: u8) -> Option<()> {
        if self.peek() == Some(expected) {
            self.pos += 1;
            Some(())
        } else {
            None
        }
    }

    fn parse_hex4(&mut self) -> Option<u16> {
        let slice = self.bytes.get(self.pos..self.pos + 4)?;
        let text = std::str::from_utf8(slice).ok()?;
        let value = u16::from_str_radix(text, 16).ok()?;
        self.pos += 4;
        Some(value)
    }

    fn parse_string(&mut self) -> Option<String> {
        self.expect(b'"')?;
        let mut out = String::new();
        while self.pos < self.bytes.len() {
            let b = self.bytes[self.pos];
            if b == b'"' {
                self.pos += 1;
                return Some(out);
            }
            if b == b'\\' {
                self.pos += 1;
                let esc = *self.bytes.get(self.pos)?;
                self.pos += 1;
                match esc {
                    b'"' => out.push('"'),
                    b'\\' => out.push('\\'),
                    b'/' => out.push('/'),
                    b'b' => out.push('\u{0008}'),
                    b'f' => out.push('\u{000C}'),
                    b'n' => out.push('\n'),
                    b'r' => out.push('\r'),
                    b't' => out.push('\t'),
                    b'u' => {
                        let first = self.parse_hex4()?;
                        if (0xD800..=0xDBFF).contains(&first) {
                            self.expect(b'\\')?;
                            self.expect(b'u')?;
                            let second = self.parse_hex4()?;
                            if !(0xDC00..=0xDFFF).contains(&second) {
                                return None;
                            }
                            let cp = 0x10000
                                + (((u32::from(first) - 0xD800) << 10)
                                    | (u32::from(second) - 0xDC00));
                            out.push(char::from_u32(cp)?);
                        } else {
                            out.push(char::from_u32(u32::from(first))?);
                        }
                    }
                    _ => return None,
                }
                continue;
            }
            let rest = std::str::from_utf8(&self.bytes[self.pos..]).ok()?;
            let ch = rest.chars().next()?;
            out.push(ch);
            self.pos += ch.len_utf8();
        }
        None
    }

    fn parse_array(&mut self) -> Option<Vec<JsonValue>> {
        self.expect(b'[')?;
        let mut items = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Some(items);
        }
        loop {
            items.push(self.parse_value()?);
            self.skip_ws();
            match self.peek()? {
                b',' => self.pos += 1,
                b']' => {
                    self.pos += 1;
                    return Some(items);
                }
                _ => return None,
            }
        }
    }

    fn parse_object(&mut self) -> Option<Vec<(String, JsonValue)>> {
        self.expect(b'{')?;
        let mut entries = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Some(entries);
        }
        loop {
            self.skip_ws();
            let key = self.parse_string()?;
            self.skip_ws();
            self.expect(b':')?;
            let value = self.parse_value()?;
            entries.push((key, value));
            self.skip_ws();
            match self.peek()? {
                b',' => self.pos += 1,
                b'}' => {
                    self.pos += 1;
                    return Some(entries);
                }
                _ => return None,
            }
        }
    }

    fn consume_literal(&mut self, lit: &[u8]) -> Option<()> {
        if self.bytes.get(self.pos..self.pos + lit.len()) == Some(lit) {
            self.pos += lit.len();
            Some(())
        } else {
            None
        }
    }

    fn parse_number(&mut self) -> Option<()> {
        if self.peek() == Some(b'-') {
            self.pos += 1;
        }
        let start = self.pos;
        while let Some(b) = self.peek() {
            if b.is_ascii_digit() || matches!(b, b'.' | b'e' | b'E' | b'+' | b'-') {
                self.pos += 1;
            } else {
                break;
            }
        }
        (self.pos > start).then_some(())
    }

    fn parse_value(&mut self) -> Option<JsonValue> {
        self.skip_ws();
        match self.peek()? {
            b'{' => Some(JsonValue::Object(self.parse_object()?)),
            b'[' => Some(JsonValue::Array(self.parse_array()?)),
            b'"' => Some(JsonValue::Str(self.parse_string()?)),
            b't' => {
                self.consume_literal(b"true")?;
                Some(JsonValue::Bool)
            }
            b'f' => {
                self.consume_literal(b"false")?;
                Some(JsonValue::Bool)
            }
            b'n' => {
                self.consume_literal(b"null")?;
                Some(JsonValue::Null)
            }
            b'-' | b'0'..=b'9' => {
                self.parse_number()?;
                Some(JsonValue::Number)
            }
            _ => None,
        }
    }
}

fn parse_json(input: &str) -> Option<JsonValue> {
    let mut parser = JsonParser::new(input);
    let value = parser.parse_value()?;
    parser.skip_ws();
    (parser.pos == parser.bytes.len()).then_some(value)
}

/// Extracts the `.rlib` path for the `worth-query-decl` library target from a
/// single Cargo `--message-format=json` output line using structural JSON parsing.
pub(super) fn extract_worth_query_decl_rlib(line: &str) -> Option<PathBuf> {
    let root = parse_json(line)?;
    if root.field("reason")?.as_str()? != "compiler-artifact" {
        return None;
    }
    let target = root.field("target")?;
    let name = target.field("name")?.as_str()?;
    if name != "worth_query_decl" && name != "worth-query-decl" {
        return None;
    }
    let is_lib = target
        .field("kind")?
        .as_array()?
        .iter()
        .any(|k| k.as_str() == Some("lib"));
    if !is_lib {
        return None;
    }
    root.field("filenames")?
        .as_array()?
        .iter()
        .filter_map(JsonValue::as_str)
        .find(|path| path.ends_with(".rlib"))
        .map(PathBuf::from)
}

#[test]
fn cargo_artifact_json_parser_handles_brackets_and_escapes_in_paths() {
    let bracket_line = r#"{"reason":"compiler-artifact","package_id":"path+file:///work/worth-query-decl#0.1.0","target":{"kind":["lib"],"crate_types":["lib"],"name":"worth_query_decl","src_path":"/work/lib.rs"},"profile":{"opt_level":"0","debuginfo":0,"test":false},"features":[],"filenames":["/tmp/query]target/debug/deps/libworth_query_decl-abc123.rlib","/tmp/query]target/debug/deps/libworth_query_decl-abc123.rmeta"],"executable":null,"fresh":true}"#;
    assert_eq!(
        extract_worth_query_decl_rlib(bracket_line),
        Some(PathBuf::from(
            "/tmp/query]target/debug/deps/libworth_query_decl-abc123.rlib"
        ))
    );

    // Escaped backslash, escaped quote, and `\u005d` (`]`) inside a path.
    let escaped_line = r#"{"reason":"compiler-artifact","target":{"kind":["lib"],"name":"worth-query-decl"},"filenames":["C:\\tmp\\q\u005d\"dir\\libworth_query_decl.rlib"]}"#;
    assert_eq!(
        extract_worth_query_decl_rlib(escaped_line),
        Some(PathBuf::from(r#"C:\tmp\q]"dir\libworth_query_decl.rlib"#))
    );

    // Non-lib target or different crate name must be ignored even if a string mentions worth_query_decl.
    let test_target_line = r#"{"reason":"compiler-artifact","target":{"kind":["test"],"name":"worth_query_decl"},"filenames":["/tmp/libworth_query_decl.rlib"]}"#;
    assert_eq!(extract_worth_query_decl_rlib(test_target_line), None);
    let other_crate_line = r#"{"reason":"compiler-artifact","target":{"kind":["lib"],"name":"other_crate"},"filenames":["/tmp/libworth_query_decl.rlib"]}"#;
    assert_eq!(extract_worth_query_decl_rlib(other_crate_line), None);
}
