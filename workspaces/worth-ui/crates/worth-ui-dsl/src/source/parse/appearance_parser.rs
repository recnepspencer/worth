use super::worth_ui_source_parser::parse_block_body_tokens;
use super::worth_ui_source_parser_expectations::{
    expect_identifier_token, expect_punctuation_token, span_from_bounds, token_identifier_text,
    unexpected_token_diagnostic, TokenExpectation,
};
use super::worth_ui_source_token_stream::WorthUiSourceTokenStream;
use crate::source::{
    WorthUiParseDiagnostic, WorthUiParsedAppearanceRoleDeclaration, WorthUiParsedBlockBody,
    WorthUiParsedBlockDeclaration, WorthUiParsedSourceDeclaration, WorthUiSourceModuleId,
    WorthUiSourceSpan, WorthUiSourceToken, WorthUiSourceTokenKind,
};

pub(super) fn parse_appearance_role_declaration(
    module_id: &WorthUiSourceModuleId,
    source_length: usize,
    stream: &mut WorthUiSourceTokenStream,
) -> Result<WorthUiParsedSourceDeclaration, WorthUiParseDiagnostic> {
    let appearance = stream.next().expect("appearance token should exist");
    let role_keyword = expect_identifier_token(
        module_id,
        source_length,
        stream,
        "appearance declaration requires the 'role' form",
    )?;
    if token_identifier_text(&role_keyword) != "role" {
        return Err(unexpected_token_diagnostic(
            role_keyword,
            "appearance declaration requires the 'role' form",
        ));
    }
    let name = expect_identifier_token(
        module_id,
        source_length,
        stream,
        "appearance role declaration requires a role identity",
    )?;
    let revision = optional_revision(module_id, source_length, stream)?;
    let applies_to = expect_word(
        module_id,
        source_length,
        stream,
        "appearance role declaration requires 'applies_to'",
    )?;
    if applies_to.0 != "applies_to" {
        return Err(unexpected_token_diagnostic(
            applies_to.1,
            "appearance role declaration requires 'applies_to'",
        ));
    }
    let target = expect_word(
        module_id,
        source_length,
        stream,
        "appearance role declaration requires an applicability target",
    )?;
    let left = expect_punctuation_token(
        module_id,
        source_length,
        stream,
        TokenExpectation::LeftBrace,
        "appearance role declaration requires '{'",
    )?;
    let (body_tokens, right) = parse_block_body_tokens(module_id, stream, left.span())?;
    let body = WorthUiParsedBlockBody::new_with_spans(
        span_from_bounds(left.span(), right.span()),
        body_tokens,
    );
    Ok(WorthUiParsedSourceDeclaration::AppearanceRole(
        WorthUiParsedAppearanceRoleDeclaration::new(
            token_identifier_text(&name),
            revision,
            target.0,
            span_from_bounds(appearance.span(), right.span()),
            body,
        ),
    ))
}

pub(super) fn parse_backdrop_declaration(
    module_id: &WorthUiSourceModuleId,
    source_length: usize,
    stream: &mut WorthUiSourceTokenStream,
) -> Result<WorthUiParsedSourceDeclaration, WorthUiParseDiagnostic> {
    let keyword = stream.next().expect("backdrop token should exist");
    let name = expect_identifier_token(
        module_id,
        source_length,
        stream,
        "backdrop declaration requires an identity",
    )?;
    let left = expect_punctuation_token(
        module_id,
        source_length,
        stream,
        TokenExpectation::LeftBrace,
        "backdrop declaration requires '{'",
    )?;
    let (body_tokens, right) = parse_block_body_tokens(module_id, stream, left.span())?;
    Ok(WorthUiParsedSourceDeclaration::Backdrop(
        WorthUiParsedBlockDeclaration::new(
            token_identifier_text(&name),
            span_from_bounds(keyword.span(), right.span()),
            WorthUiParsedBlockBody::new_with_spans(
                span_from_bounds(left.span(), right.span()),
                body_tokens,
            ),
        ),
    ))
}

fn optional_revision(
    module_id: &WorthUiSourceModuleId,
    source_length: usize,
    stream: &mut WorthUiSourceTokenStream,
) -> Result<u64, WorthUiParseDiagnostic> {
    let Some(token) = stream.peek() else {
        return Ok(1);
    };
    if !matches!(token.kind(), WorthUiSourceTokenKind::Identifier(value) if value == "revision") {
        return Ok(1);
    }
    let _ = stream.next();
    let Some(token) = stream.next() else {
        return Err(WorthUiParseDiagnostic::new(
            crate::source::WorthUiParseDiagnosticCode::MissingIdentifier,
            "revision requires a positive integer",
            WorthUiSourceSpan::new(module_id.clone(), source_length, source_length),
        ));
    };
    match token.kind() {
        WorthUiSourceTokenKind::NumberLiteral(value) => value.parse().map_err(|_| {
            unexpected_token_diagnostic(token, "revision requires a positive integer")
        }),
        _ => Err(unexpected_token_diagnostic(
            token,
            "revision requires a positive integer",
        )),
    }
}

fn expect_word(
    module_id: &WorthUiSourceModuleId,
    source_length: usize,
    stream: &mut WorthUiSourceTokenStream,
    message: &str,
) -> Result<(String, WorthUiSourceToken), WorthUiParseDiagnostic> {
    let Some(token) = stream.next() else {
        return Err(WorthUiParseDiagnostic::new(
            crate::source::WorthUiParseDiagnosticCode::MissingIdentifier,
            message,
            WorthUiSourceSpan::new(module_id.clone(), source_length, source_length),
        ));
    };
    let text = match token.kind() {
        WorthUiSourceTokenKind::Identifier(value) => value.clone(),
        WorthUiSourceTokenKind::KeywordBackdrop => "backdrop".to_owned(),
        _ => return Err(unexpected_token_diagnostic(token, message)),
    };
    Ok((text, token))
}
