pub(crate) use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum JsonValue {
    Object(BTreeMap<String, JsonValue>),
    Array(Vec<JsonValue>),
    String(String),
    Number(u64),
    Boolean(bool),
    Null,
}

impl JsonValue {
    pub(crate) fn field(&self, name: &str) -> &Self {
        let Self::Object(fields) = self else {
            panic!("JSON value is not an object")
        };
        fields.get(name).unwrap_or_else(|| panic!("missing {name}"))
    }

    pub(crate) fn array(&self) -> &[Self] {
        let Self::Array(values) = self else {
            panic!("JSON value is not an array")
        };
        values
    }

    pub(crate) fn string(&self) -> &str {
        let Self::String(value) = self else {
            panic!("JSON value is not a string")
        };
        value
    }

    pub(crate) fn number(&self) -> u64 {
        let Self::Number(value) = self else {
            panic!("JSON value is not a number")
        };
        *value
    }
}

pub(crate) fn parse_json(source: &str) -> JsonValue {
    let mut parser = Parser {
        bytes: source.as_bytes(),
        cursor: 0,
    };
    let value = parser.value();
    parser.whitespace();
    assert_eq!(parser.cursor, parser.bytes.len(), "trailing JSON bytes");
    value
}

pub(crate) struct Parser<'source> {
    pub(crate) bytes: &'source [u8],
    pub(crate) cursor: usize,
}

impl Parser<'_> {
    pub(crate) fn value(&mut self) -> JsonValue {
        self.whitespace();
        match self.peek() {
            Some(b'{') => self.object(),
            Some(b'[') => self.array_value(),
            Some(b'"') => crate::support::json::JsonValue::String(self.string_value()),
            Some(b't') => {
                self.literal(b"true");
                crate::support::json::JsonValue::Boolean(true)
            }
            Some(b'f') => {
                self.literal(b"false");
                crate::support::json::JsonValue::Boolean(false)
            }
            Some(b'n') => {
                self.literal(b"null");
                crate::support::json::JsonValue::Null
            }
            Some(b'0'..=b'9') => crate::support::json::JsonValue::Number(self.number_value()),
            other => panic!("unexpected JSON byte {other:?} at {}", self.cursor),
        }
    }

    pub(crate) fn object(&mut self) -> JsonValue {
        self.expect(b'{');
        let mut fields = std::collections::BTreeMap::new();
        self.whitespace();
        if self.consume(b'}') {
            return crate::support::json::JsonValue::Object(fields);
        }
        loop {
            self.whitespace();
            let name = self.string_value();
            self.whitespace();
            self.expect(b':');
            assert!(
                fields.insert(name, self.value()).is_none(),
                "duplicate field"
            );
            self.whitespace();
            if self.consume(b'}') {
                break;
            }
            self.expect(b',');
        }
        crate::support::json::JsonValue::Object(fields)
    }

    pub(crate) fn array_value(&mut self) -> JsonValue {
        self.expect(b'[');
        let mut values = Vec::new();
        self.whitespace();
        if self.consume(b']') {
            return crate::support::json::JsonValue::Array(values);
        }
        loop {
            values.push(self.value());
            self.whitespace();
            if self.consume(b']') {
                break;
            }
            self.expect(b',');
        }
        crate::support::json::JsonValue::Array(values)
    }

    pub(crate) fn string_value(&mut self) -> String {
        self.expect(b'"');
        let mut value = String::new();
        loop {
            let byte = self.next().expect("unterminated JSON string");
            match byte {
                b'"' => return value,
                b'\\' => value.push(self.escape()),
                0..=0x1f => panic!("unescaped JSON control byte"),
                0x20..=0x7f => value.push(byte as char),
                _ => {
                    self.cursor -= 1;
                    let remaining = std::str::from_utf8(&self.bytes[self.cursor..]).unwrap();
                    let character = remaining.chars().next().unwrap();
                    self.cursor += character.len_utf8();
                    value.push(character);
                }
            }
        }
    }

    pub(crate) fn escape(&mut self) -> char {
        match self.next().expect("missing JSON escape") {
            b'"' => '"',
            b'\\' => '\\',
            b'/' => '/',
            b'b' => '\u{0008}',
            b'f' => '\u{000c}',
            b'n' => '\n',
            b'r' => '\r',
            b't' => '\t',
            b'u' => {
                let mut value = 0_u32;
                for _ in 0..4 {
                    value = value * 16 + u32::from(hex(self.next().expect("short unicode escape")));
                }
                char::from_u32(value).expect("valid JSON unicode scalar")
            }
            other => panic!("invalid JSON escape {other}"),
        }
    }

    pub(crate) fn number_value(&mut self) -> u64 {
        let start = self.cursor;
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.cursor += 1;
        }
        std::str::from_utf8(&self.bytes[start..self.cursor])
            .unwrap()
            .parse()
            .unwrap()
    }

    pub(crate) fn literal(&mut self, expected: &[u8]) {
        assert_eq!(
            self.bytes.get(self.cursor..self.cursor + expected.len()),
            Some(expected)
        );
        self.cursor += expected.len();
    }

    pub(crate) fn whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.cursor += 1;
        }
    }

    pub(crate) fn expect(&mut self, expected: u8) {
        assert_eq!(
            self.next(),
            Some(expected),
            "JSON syntax at {}",
            self.cursor
        );
    }

    pub(crate) fn consume(&mut self, expected: u8) -> bool {
        if self.peek() == Some(expected) {
            self.cursor += 1;
            true
        } else {
            false
        }
    }

    pub(crate) fn peek(&self) -> Option<u8> {
        self.bytes.get(self.cursor).copied()
    }

    pub(crate) fn next(&mut self) -> Option<u8> {
        let value = self.peek()?;
        self.cursor += 1;
        Some(value)
    }
}

pub(crate) fn hex(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        b'a'..=b'f' => value - b'a' + 10,
        b'A'..=b'F' => value - b'A' + 10,
        _ => panic!("invalid hexadecimal digit"),
    }
}
