use crate::source::{
    WorthUiParseDiagnostic, WorthUiParseDiagnosticCode, WorthUiSourceModuleId, WorthUiSourceSpan,
    WorthUiSourceToken, WorthUiSourceTokenKind,
};

pub(crate) fn tokenize_module_source(
    module_id: &WorthUiSourceModuleId,
    source_text: &str,
) -> Result<Vec<WorthUiSourceToken>, Vec<WorthUiParseDiagnostic>> {
    let mut diagnostics = Vec::new();
    let mut tokens = Vec::new();
    let mut position = 0;

    while position < source_text.len() {
        let slice = &source_text[position..];
        let next = slice.chars().next().expect("slice should be non-empty");

        if next.is_whitespace() {
            position += next.len_utf8();
            continue;
        }

        if slice.starts_with("//") {
            position = advance_to_next_line(source_text, position);
            continue;
        }

        if next == '"' {
            match consume_string_literal(module_id, source_text, position) {
                Ok((token, next_position)) => {
                    tokens.push(token);
                    position = next_position;
                }
                Err(diagnostic) => diagnostics.push(diagnostic),
            }
            if !diagnostics.is_empty() {
                break;
            }
            continue;
        }

        if let Some((token, next_position)) = consume_punctuation(module_id, position, next) {
            tokens.push(token);
            position = next_position;
            continue;
        }

        if is_identifier_start(next) {
            let (token, next_position) = consume_identifier(module_id, source_text, position);
            tokens.push(token);
            position = next_position;
            continue;
        }

        if next.is_ascii_digit() {
            let (token, next_position) = consume_number(module_id, source_text, position);
            tokens.push(token);
            position = next_position;
            continue;
        }

        diagnostics.push(WorthUiParseDiagnostic::new(
            WorthUiParseDiagnosticCode::InvalidCharacter,
            format!("invalid source character '{next}'"),
            WorthUiSourceSpan::new(module_id.clone(), position, position + next.len_utf8()),
        ));
        break;
    }

    if diagnostics.is_empty() {
        Ok(tokens)
    } else {
        Err(diagnostics)
    }
}

fn consume_identifier(
    module_id: &WorthUiSourceModuleId,
    source_text: &str,
    start: usize,
) -> (WorthUiSourceToken, usize) {
    let mut end = start;
    for character in source_text[start..].chars() {
        if !is_identifier_continue(character) {
            break;
        }
        end += character.len_utf8();
    }
    let raw_text = &source_text[start..end];
    let kind = match raw_text {
        "import" => WorthUiSourceTokenKind::KeywordImport,
        "component" => WorthUiSourceTokenKind::KeywordComponent,
        "control" => WorthUiSourceTokenKind::KeywordControl,
        "intent" => WorthUiSourceTokenKind::KeywordIntent,
        "surface" => WorthUiSourceTokenKind::KeywordSurface,
        "binding" => WorthUiSourceTokenKind::KeywordBinding,
        "query_scalar" => WorthUiSourceTokenKind::KeywordQueryScalar,
        "query_collection" => WorthUiSourceTokenKind::KeywordQueryCollection,
        "token" => WorthUiSourceTokenKind::KeywordToken,
        "appearance" => WorthUiSourceTokenKind::KeywordAppearance,
        "backdrop" => WorthUiSourceTokenKind::KeywordBackdrop,
        _ => WorthUiSourceTokenKind::Identifier(raw_text.to_owned()),
    };
    (
        WorthUiSourceToken::new(kind, WorthUiSourceSpan::new(module_id.clone(), start, end)),
        end,
    )
}

fn consume_number(
    module_id: &WorthUiSourceModuleId,
    source_text: &str,
    start: usize,
) -> (WorthUiSourceToken, usize) {
    let mut end = start;
    for character in source_text[start..].chars() {
        if !character.is_ascii_digit() {
            break;
        }
        end += character.len_utf8();
    }
    (
        WorthUiSourceToken::new(
            WorthUiSourceTokenKind::NumberLiteral(source_text[start..end].to_owned()),
            WorthUiSourceSpan::new(module_id.clone(), start, end),
        ),
        end,
    )
}

fn consume_string_literal(
    module_id: &WorthUiSourceModuleId,
    source_text: &str,
    start: usize,
) -> Result<(WorthUiSourceToken, usize), WorthUiParseDiagnostic> {
    let mut value = String::new();
    let mut position = start + 1;
    let mut escaped = false;

    while position < source_text.len() {
        let character = source_text[position..]
            .chars()
            .next()
            .expect("string slice should contain a character");
        position += character.len_utf8();

        if escaped {
            value.push(character);
            escaped = false;
            continue;
        }

        match character {
            '\\' => escaped = true,
            '"' => {
                return Ok((
                    WorthUiSourceToken::new(
                        WorthUiSourceTokenKind::StringLiteral(value),
                        WorthUiSourceSpan::new(module_id.clone(), start, position),
                    ),
                    position,
                ));
            }
            _ => value.push(character),
        }
    }

    Err(WorthUiParseDiagnostic::new(
        WorthUiParseDiagnosticCode::UnterminatedStringLiteral,
        "string literal reached end of module without a closing quote",
        WorthUiSourceSpan::new(module_id.clone(), start, source_text.len()),
    ))
}

fn consume_punctuation(
    module_id: &WorthUiSourceModuleId,
    position: usize,
    character: char,
) -> Option<(WorthUiSourceToken, usize)> {
    let kind = match character {
        '{' => WorthUiSourceTokenKind::LeftBrace,
        '}' => WorthUiSourceTokenKind::RightBrace,
        ';' => WorthUiSourceTokenKind::Semicolon,
        '=' => WorthUiSourceTokenKind::Equals,
        '+' => WorthUiSourceTokenKind::Plus,
        '[' => WorthUiSourceTokenKind::LeftBracket,
        ']' => WorthUiSourceTokenKind::RightBracket,
        '(' => WorthUiSourceTokenKind::LeftParen,
        ')' => WorthUiSourceTokenKind::RightParen,
        ',' => WorthUiSourceTokenKind::Comma,
        _ => return None,
    };
    let end = position + character.len_utf8();
    Some((
        WorthUiSourceToken::new(
            kind,
            WorthUiSourceSpan::new(module_id.clone(), position, end),
        ),
        end,
    ))
}

fn advance_to_next_line(source_text: &str, start: usize) -> usize {
    let remainder = &source_text[start..];
    match remainder.find('\n') {
        Some(offset) => start + offset + 1,
        None => source_text.len(),
    }
}

fn is_identifier_start(character: char) -> bool {
    character.is_ascii_alphabetic() || character == '_'
}

fn is_identifier_continue(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '_' | '.' | '-' | '/')
}
