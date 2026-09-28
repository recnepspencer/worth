//! A small reader for the protobuf text format the vendored CEL cases use:
//! nested messages, bare words, and C-escaped strings. It keeps field order
//! and repetition and interprets nothing.

#[derive(Debug, Clone)]
pub(crate) enum Node {
    /// A bare word: an identifier, enum name, or number.
    Word(String),
    /// A quoted string after unescaping; adjacent strings concatenate.
    Text(Vec<u8>),
    Message(Message),
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Message(Vec<(String, Node)>);

impl Message {
    pub(crate) fn all<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Node> + 'a {
        self.0
            .iter()
            .filter(move |(field, _)| field == name)
            .map(|(_, node)| node)
    }

    pub(crate) fn get(&self, name: &str) -> Option<&Node> {
        self.0
            .iter()
            .find(|(field, _)| field == name)
            .map(|(_, node)| node)
    }

    pub(crate) fn message(&self, name: &str) -> Option<&Message> {
        match self.get(name)? {
            Node::Message(message) => Some(message),
            _ => None,
        }
    }

    pub(crate) fn text(&self, name: &str) -> Option<String> {
        match self.get(name)? {
            Node::Text(bytes) => Some(String::from_utf8(bytes.clone()).expect("UTF-8 text")),
            _ => None,
        }
    }

    pub(crate) fn word(&self, name: &str) -> Option<&str> {
        match self.get(name)? {
            Node::Word(word) => Some(word),
            _ => None,
        }
    }

    pub(crate) fn fields(&self) -> impl Iterator<Item = (&str, &Node)> {
        self.0.iter().map(|(field, node)| (field.as_str(), node))
    }
}

#[derive(Debug, PartialEq)]
enum Token {
    Open,
    Close,
    Colon,
    Word(String),
    Text(Vec<u8>),
}

pub(crate) fn parse(source: &str) -> Message {
    let tokens = tokenize(source.as_bytes());
    let mut at = 0;
    let message = fields(&tokens, &mut at);
    assert_eq!(at, tokens.len(), "unbalanced textproto");
    message
}

fn fields(tokens: &[Token], at: &mut usize) -> Message {
    let mut message = Message::default();
    while let Some(Token::Word(name)) = tokens.get(*at) {
        *at += 1;
        if tokens.get(*at) == Some(&Token::Colon) {
            *at += 1;
        }
        let node = match &tokens[*at] {
            Token::Open => {
                *at += 1;
                let inner = fields(tokens, at);
                assert_eq!(tokens.get(*at), Some(&Token::Close), "unclosed {name}");
                Node::Message(inner)
            }
            Token::Word(word) => Node::Word(word.clone()),
            Token::Text(bytes) => Node::Text(bytes.clone()),
            other => panic!("unexpected {other:?} after {name}"),
        };
        *at += 1;
        message.0.push((name.clone(), node));
    }
    message
}

fn tokenize(bytes: &[u8]) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut at = 0;
    while let Some(&byte) = bytes.get(at) {
        match byte {
            b'#' => {
                while bytes.get(at).is_some_and(|byte| *byte != b'\n') {
                    at += 1;
                }
            }
            b'{' | b'<' => {
                tokens.push(Token::Open);
                at += 1;
            }
            b'}' | b'>' => {
                tokens.push(Token::Close);
                at += 1;
            }
            b':' => {
                tokens.push(Token::Colon);
                at += 1;
            }
            b'"' | b'\'' => {
                let text = quoted(bytes, &mut at);
                match tokens.last_mut() {
                    Some(Token::Text(previous)) => previous.extend(text),
                    _ => tokens.push(Token::Text(text)),
                }
            }
            byte if byte.is_ascii_whitespace() || byte == b',' || byte == b';' => at += 1,
            _ => {
                let start = at;
                while bytes.get(at).is_some_and(|byte| {
                    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'+' | b'-')
                }) {
                    at += 1;
                }
                assert!(at > start, "unexpected byte {byte:#x}");
                let word = String::from_utf8(bytes[start..at].to_vec()).expect("ASCII word");
                tokens.push(Token::Word(word));
            }
        }
    }
    tokens
}

/// One quoted string with C escapes: simple escapes, octal, `\x`, `\u`, `\U`.
fn quoted(bytes: &[u8], at: &mut usize) -> Vec<u8> {
    let quote = bytes[*at];
    *at += 1;
    let mut out = Vec::new();
    loop {
        let byte = bytes[*at];
        *at += 1;
        match byte {
            _ if byte == quote => return out,
            b'\\' => {
                let escape = bytes[*at];
                *at += 1;
                match escape {
                    b'n' => out.push(b'\n'),
                    b'r' => out.push(b'\r'),
                    b't' => out.push(b'\t'),
                    b'a' => out.push(0x07),
                    b'b' => out.push(0x08),
                    b'f' => out.push(0x0C),
                    b'v' => out.push(0x0B),
                    b'0'..=b'7' => {
                        let digits = run(bytes, *at - 1, 3, |byte| matches!(byte, b'0'..=b'7'));
                        out.push(number(&bytes[*at - 1..*at - 1 + digits], 8) as u8);
                        *at += digits - 1;
                    }
                    b'x' | b'X' => {
                        let digits = run(bytes, *at, 2, |byte| byte.is_ascii_hexdigit());
                        out.push(number(&bytes[*at..*at + digits], 16) as u8);
                        *at += digits;
                    }
                    b'u' | b'U' => {
                        let digits = if escape == b'u' { 4 } else { 8 };
                        let scalar = number(&bytes[*at..*at + digits], 16);
                        let scalar = char::from_u32(scalar).expect("escaped scalar");
                        out.extend(scalar.to_string().as_bytes());
                        *at += digits;
                    }
                    other => out.push(other),
                }
            }
            _ => out.push(byte),
        }
    }
}

fn run(bytes: &[u8], start: usize, most: usize, accept: fn(&u8) -> bool) -> usize {
    bytes[start..]
        .iter()
        .take(most)
        .take_while(|byte| accept(byte))
        .count()
}

fn number(digits: &[u8], radix: u32) -> u32 {
    u32::from_str_radix(std::str::from_utf8(digits).expect("ASCII digits"), radix)
        .expect("escape digits")
}
