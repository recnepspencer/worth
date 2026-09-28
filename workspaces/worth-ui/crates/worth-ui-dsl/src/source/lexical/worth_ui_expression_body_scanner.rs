use crate::source::{
    WorthUiParseDiagnostic, WorthUiParseDiagnosticCode, WorthUiSourceModuleId, WorthUiSourceSpan,
    WorthUiSourceToken, WorthUiSourceTokenKind,
};
use crate::{WorthUiExpressionBody, WorthUiExpressionIntroducer};

/// Reads an expression body when the identifier at `start` is `when` or
/// `value` followed, after optional whitespace, by `(`.
///
/// The body is the raw text between the outer parentheses. The scan balances
/// `()[]{}`, skips kernel strings and kernel `//` line comments, and leaves
/// the text itself to the kernel. Any other use of the identifier returns
/// `None`, so appearance cells such as `cell hovered when hover = hovered`
/// keep lexing as ordinary identifiers.
pub(super) fn consume_expression_body(
    module_id: &WorthUiSourceModuleId,
    source_text: &str,
    start: usize,
    identifier_end: usize,
) -> Option<Result<(WorthUiSourceToken, usize), WorthUiParseDiagnostic>> {
    let introducer = match &source_text[start..identifier_end] {
        "when" => WorthUiExpressionIntroducer::When,
        "value" => WorthUiExpressionIntroducer::Value,
        _ => return None,
    };
    let open = identifier_end
        + source_text[identifier_end..].find(|character: char| !character.is_whitespace())?;
    if !source_text[open..].starts_with('(') {
        return None;
    }
    Some(scan_body(module_id, source_text, introducer, open))
}

fn scan_body(
    module_id: &WorthUiSourceModuleId,
    source_text: &str,
    introducer: WorthUiExpressionIntroducer,
    open: usize,
) -> Result<(WorthUiSourceToken, usize), WorthUiParseDiagnostic> {
    let opening_span = || WorthUiSourceSpan::new(module_id.clone(), open, open + 1);
    let mut expected_closers = Vec::new();
    let mut characters = source_text[open..].char_indices().peekable();
    while let Some((offset, character)) = characters.next() {
        match character {
            '(' => expected_closers.push(')'),
            '[' => expected_closers.push(']'),
            '{' => expected_closers.push('}'),
            ')' | ']' | '}' => {
                if expected_closers.pop() != Some(character) {
                    return Err(WorthUiParseDiagnostic::new(
                        WorthUiParseDiagnosticCode::UnterminatedExpressionBody,
                        format!("expression body has an unmatched '{character}'"),
                        opening_span(),
                    ));
                }
                if expected_closers.is_empty() {
                    let close = open + offset;
                    let body = WorthUiExpressionBody::new(
                        introducer,
                        &source_text[open + 1..close],
                        open + 1,
                    );
                    return Ok((
                        WorthUiSourceToken::new(
                            WorthUiSourceTokenKind::ExpressionBody(body),
                            WorthUiSourceSpan::new(module_id.clone(), open + 1, close),
                        ),
                        close + 1,
                    ));
                }
            }
            '"' => skip_string(&mut characters),
            '/' => {
                if characters.next_if(|(_, next)| *next == '/').is_some() {
                    while characters.next_if(|(_, next)| *next != '\n').is_some() {}
                }
            }
            _ => {}
        }
    }
    Err(WorthUiParseDiagnostic::new(
        WorthUiParseDiagnosticCode::UnterminatedExpressionBody,
        format!(
            "`{}(` expression body reached end of module without its closing ')'",
            introducer.keyword()
        ),
        opening_span(),
    ))
}

/// Skips to the closing quote of a kernel string, where a backslash escapes
/// the next character. An unterminated string consumes the rest of the
/// module, which the caller then reports as an unterminated body.
fn skip_string(characters: &mut impl Iterator<Item = (usize, char)>) {
    let mut escaped = false;
    for (_, character) in characters {
        match (escaped, character) {
            (true, _) => escaped = false,
            (false, '\\') => escaped = true,
            (false, '"') => return,
            (false, _) => {}
        }
    }
}
