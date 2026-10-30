//! Small text regions: discard Rust comments/literals without parsing Rust syntax.
#[derive(Clone, Copy, Debug)]
pub struct Token<'a> {
    pub text: &'a str,
    pub start: usize,
    pub end: usize,
}
pub fn tokens(source: &str) -> Vec<Token<'_>> {
    let b = source.as_bytes();
    let mut result = Vec::new();
    let mut i = 0;
    while i < b.len() {
        let start = i;
        let raw_identifier = b[i..].starts_with(b"r#")
            && b.get(i + 2)
                .is_some_and(|c| c.is_ascii_alphabetic() || *c == b'_' || *c >= 128);
        if b[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if b[i..].starts_with(b"//") {
            while i < b.len() && b[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if b[i..].starts_with(b"/*") {
            i += 2;
            let mut depth = 1;
            while i < b.len() && depth > 0 {
                if b[i..].starts_with(b"/*") {
                    depth += 1;
                    i += 2;
                } else if b[i..].starts_with(b"*/") {
                    depth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            continue;
        }
        let raw = if b[i..].starts_with(b"br") || b[i..].starts_with(b"cr") {
            i + 2
        } else if b[i] == b'r' {
            i + 1
        } else {
            i
        };
        let mut quote = raw;
        while quote < b.len() && b[quote] == b'#' {
            quote += 1;
        }
        if raw > i && b.get(quote) == Some(&b'"') {
            let end = format!("\"{}", "#".repeat(quote - raw));
            i = source[quote + 1..]
                .find(&end)
                .map_or(b.len(), |p| quote + 1 + p + end.len());
        } else if b[i] == b'"'
            || b[i] == b'\'' && (b.get(i + 2) == Some(&b'\'') || b.get(i + 1) == Some(&b'\\'))
        {
            let delimiter = b[i];
            i += 1;
            while i < b.len() {
                if b[i] == b'\\' {
                    i = (i + 2).min(b.len());
                } else if b[i] == delimiter {
                    i += 1;
                    break;
                } else {
                    i += 1;
                }
            }
        } else if raw_identifier {
            i += 2;
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_' || b[i] >= 128) {
                i += 1;
            }
        } else if b[i].is_ascii_alphanumeric() || b[i] == b'_' || b[i] >= 128 {
            i += 1;
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_' || b[i] >= 128) {
                i += 1;
            }
        } else {
            i += 1;
        }
        result.push(Token {
            text: &source[start + if raw_identifier { 2 } else { 0 }..i],
            start,
            end: i,
        });
    }
    result
}
pub fn close(tokens: &[Token<'_>], start: usize) -> usize {
    let (open, shut) = match tokens[start].text {
        "{" => ("{", "}"),
        "[" => ("[", "]"),
        "(" => ("(", ")"),
        _ => return start,
    };
    let mut depth = 0;
    for (i, t) in tokens.iter().enumerate().skip(start) {
        if t.text == open {
            depth += 1;
        }
        if t.text == shut {
            depth -= 1;
            if depth == 0 {
                return i;
            }
        }
    }
    tokens.len() - 1
}
/// Test regions include inline module bodies and file-mounted test modules.
pub fn test_modules<'a>(tokens: &'a [Token<'a>]) -> Vec<(&'a str, usize, usize)> {
    let mut result = Vec::new();
    for i in 0..tokens.len().saturating_sub(7) {
        if tokens[i].text != "#" || tokens[i + 1].text != "[" {
            continue;
        }
        let end = close(tokens, i + 1);
        if tokens[i + 2..end]
            .iter()
            .map(|t| t.text)
            .collect::<String>()
            != "cfg(test)"
        {
            continue;
        }
        let mut at = end + 1;
        while at + 1 < tokens.len() && tokens[at].text == "#" && tokens[at + 1].text == "[" {
            at = close(tokens, at + 1) + 1;
        }
        if tokens.get(at).is_some_and(|t| t.text == "pub") {
            at += 1;
            if tokens.get(at).is_some_and(|t| t.text == "(") {
                at = close(tokens, at) + 1;
            }
        }
        if tokens.get(at).is_some_and(|t| t.text == "mod") && at + 2 < tokens.len() {
            let body = at + 2;
            result.push((tokens[at + 1].text, body, close(tokens, body)));
        }
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn comments_and_literal_bodies_cannot_forge_identifiers() {
        let t = tokens("/* /* Bad */ */ r###\"Bad\"###; r#real(); // Bad\n");
        assert!(!t.iter().any(|t| t.text == "Bad"));
        assert!(t.iter().any(|t| t.text == "real"));
    }
    #[test]
    fn test_scope_ends_at_its_own_closing_brace() {
        let t = tokens("#[cfg(test)] mod a { fn x() { Root; } } fn y() { Root; }");
        let modules = test_modules(&t);
        assert_eq!(modules.len(), 1);
        assert_eq!(t[modules[0].2 + 1].text, "fn");
    }
}
