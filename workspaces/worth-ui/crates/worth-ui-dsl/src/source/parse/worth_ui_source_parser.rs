use std::collections::BTreeMap;

use super::appearance_parser::{parse_appearance_role_declaration, parse_backdrop_declaration};
use super::worth_ui_source_parser_expectations::{
    expect_identifier_token, expect_punctuation_token, expect_string_literal_token,
    span_from_bounds, token_identifier_text, token_string_literal_text,
    unexpected_token_diagnostic, TokenExpectation,
};
use super::worth_ui_source_token_stream::WorthUiSourceTokenStream;
use crate::source::{
    tokenize_module_source, WorthUiParseDiagnostic, WorthUiParseDiagnosticCode, WorthUiParseReport,
    WorthUiParsedBlockBody, WorthUiParsedBlockDeclaration, WorthUiParsedImportDeclaration,
    WorthUiParsedSourceDeclaration, WorthUiParsedSourceModule, WorthUiParsedSourcePackage,
    WorthUiParsedTokenDeclaration, WorthUiSourceModuleId, WorthUiSourceModuleRecord,
    WorthUiSourcePackage, WorthUiSourceSpan, WorthUiSourceToken, WorthUiSourceTokenKind,
};

#[derive(Clone, Debug, Default)]
pub(crate) struct WorthUiSourceParser;

impl WorthUiSourceParser {
    pub(crate) fn parse_package(
        source_package: &WorthUiSourcePackage,
    ) -> Result<WorthUiParsedSourcePackage, WorthUiParseReport> {
        let mut diagnostics = Vec::new();
        let mut modules = BTreeMap::new();

        for module_id in source_package.module_ids() {
            let module_record = source_package
                .module_record(module_id)
                .expect("canonical source package should contain every module record");
            match parse_source_module(module_record) {
                Ok(parsed_module) => {
                    modules.insert(module_id.clone(), parsed_module);
                }
                Err(mut module_diagnostics) => diagnostics.append(&mut module_diagnostics),
            }
        }

        if !diagnostics.is_empty() {
            return Err(WorthUiParseReport::new(diagnostics));
        }

        Ok(WorthUiParsedSourcePackage::new(
            modules,
            source_package.module_ids().to_vec(),
        ))
    }
}

fn parse_source_module(
    module_record: &WorthUiSourceModuleRecord,
) -> Result<WorthUiParsedSourceModule, Vec<WorthUiParseDiagnostic>> {
    let tokens = tokenize_module_source(module_record.module_id(), module_record.source_text())?;
    let declarations = parse_module_declarations(
        module_record.module_id(),
        module_record.source_text().len(),
        tokens,
    )?;
    Ok(WorthUiParsedSourceModule::new(
        module_record.module_id().clone(),
        declarations,
    ))
}

fn parse_module_declarations(
    module_id: &WorthUiSourceModuleId,
    source_length: usize,
    tokens: Vec<WorthUiSourceToken>,
) -> Result<Vec<WorthUiParsedSourceDeclaration>, Vec<WorthUiParseDiagnostic>> {
    let mut stream = WorthUiSourceTokenStream::new(tokens);
    let mut declarations = Vec::new();
    let mut diagnostics = Vec::new();

    while !stream.is_eof() {
        match parse_next_declaration(module_id, source_length, &mut stream) {
            Ok(declaration) => declarations.push(declaration),
            Err(diagnostic) => {
                diagnostics.push(diagnostic);
                recover_module_root(&mut stream);
            }
        }
    }

    if diagnostics.is_empty() {
        Ok(declarations)
    } else {
        Err(diagnostics)
    }
}

fn parse_next_declaration(
    module_id: &WorthUiSourceModuleId,
    source_length: usize,
    stream: &mut WorthUiSourceTokenStream,
) -> Result<WorthUiParsedSourceDeclaration, WorthUiParseDiagnostic> {
    match stream.peek().map(WorthUiSourceToken::kind) {
        Some(WorthUiSourceTokenKind::KeywordAppearance) => {
            parse_appearance_role_declaration(module_id, source_length, stream)
        }
        Some(WorthUiSourceTokenKind::KeywordBackdrop) => {
            parse_backdrop_declaration(module_id, source_length, stream)
        }
        Some(WorthUiSourceTokenKind::KeywordImport) => {
            parse_import_declaration(module_id, source_length, stream)
        }
        Some(WorthUiSourceTokenKind::KeywordComponent) => {
            parse_block_declaration(module_id, source_length, stream, BlockKind::Component)
        }
        Some(WorthUiSourceTokenKind::KeywordControl) => {
            parse_block_declaration(module_id, source_length, stream, BlockKind::Control)
        }
        Some(WorthUiSourceTokenKind::KeywordIntent) => {
            parse_block_declaration(module_id, source_length, stream, BlockKind::Intent)
        }
        Some(WorthUiSourceTokenKind::KeywordSurface) => {
            parse_block_declaration(module_id, source_length, stream, BlockKind::Surface)
        }
        Some(WorthUiSourceTokenKind::KeywordBinding) => {
            parse_block_declaration(module_id, source_length, stream, BlockKind::Binding)
        }
        Some(WorthUiSourceTokenKind::KeywordQueryScalar) => {
            parse_block_declaration(module_id, source_length, stream, BlockKind::QueryScalar)
        }
        Some(WorthUiSourceTokenKind::KeywordQueryCollection) => {
            parse_block_declaration(module_id, source_length, stream, BlockKind::QueryCollection)
        }
        Some(WorthUiSourceTokenKind::KeywordToken) => {
            parse_token_declaration(module_id, source_length, stream)
        }
        Some(WorthUiSourceTokenKind::Identifier(keyword))
            if service_block_kind(keyword).is_some() =>
        {
            parse_block_declaration(
                module_id,
                source_length,
                stream,
                service_block_kind(keyword).expect("guarded service keyword"),
            )
        }
        Some(_) => Err(unexpected_token_diagnostic(
            stream.next().expect("peeked token should exist"),
            "expected a top-level declaration keyword",
        )),
        None => unreachable!("declaration parse should not run at eof"),
    }
}

fn parse_import_declaration(
    module_id: &WorthUiSourceModuleId,
    source_length: usize,
    stream: &mut WorthUiSourceTokenStream,
) -> Result<WorthUiParsedSourceDeclaration, WorthUiParseDiagnostic> {
    let import_keyword = stream.next().expect("import token should exist");
    let import_path_token = expect_string_literal_token(
        module_id,
        source_length,
        stream,
        "import declaration requires a quoted module path",
    )?;
    let semicolon = expect_punctuation_token(
        module_id,
        source_length,
        stream,
        TokenExpectation::Semicolon,
        "import declaration must terminate with ';'",
    )?;

    Ok(WorthUiParsedSourceDeclaration::Import(
        WorthUiParsedImportDeclaration::new(
            token_string_literal_text(&import_path_token),
            span_from_bounds(import_keyword.span(), semicolon.span()),
        ),
    ))
}

fn parse_block_declaration(
    module_id: &WorthUiSourceModuleId,
    source_length: usize,
    stream: &mut WorthUiSourceTokenStream,
    block_kind: BlockKind,
) -> Result<WorthUiParsedSourceDeclaration, WorthUiParseDiagnostic> {
    let keyword = stream.next().expect("block keyword token should exist");
    let name_token = expect_identifier_token(
        module_id,
        source_length,
        stream,
        "named block declaration requires an identifier",
    )?;
    let left_brace = expect_punctuation_token(
        module_id,
        source_length,
        stream,
        TokenExpectation::LeftBrace,
        "named block declaration requires '{'",
    )?;
    let (body_tokens, right_brace) = parse_block_body_tokens(module_id, stream, left_brace.span())?;

    let declaration = WorthUiParsedBlockDeclaration::new(
        token_identifier_text(&name_token),
        span_from_bounds(keyword.span(), right_brace.span()),
        WorthUiParsedBlockBody::new(
            span_from_bounds(left_brace.span(), right_brace.span()),
            body_tokens,
        ),
    );

    Ok(match block_kind {
        BlockKind::Component => WorthUiParsedSourceDeclaration::Component(declaration),
        BlockKind::Control => WorthUiParsedSourceDeclaration::Control(declaration),
        BlockKind::Intent => WorthUiParsedSourceDeclaration::Intent(declaration),
        BlockKind::Portal => WorthUiParsedSourceDeclaration::Portal(declaration),
        BlockKind::Focus => WorthUiParsedSourceDeclaration::Focus(declaration),
        BlockKind::Motion => WorthUiParsedSourceDeclaration::Motion(declaration),
        BlockKind::Command => WorthUiParsedSourceDeclaration::Command(declaration),
        BlockKind::Scroll => WorthUiParsedSourceDeclaration::Scroll(declaration),
        BlockKind::Selection => WorthUiParsedSourceDeclaration::Selection(declaration),
        BlockKind::Surface => WorthUiParsedSourceDeclaration::Surface(declaration),
        BlockKind::Binding => WorthUiParsedSourceDeclaration::Binding(declaration),
        BlockKind::QueryScalar => WorthUiParsedSourceDeclaration::QueryScalar(declaration),
        BlockKind::QueryCollection => WorthUiParsedSourceDeclaration::QueryCollection(declaration),
    })
}

fn parse_token_declaration(
    module_id: &WorthUiSourceModuleId,
    source_length: usize,
    stream: &mut WorthUiSourceTokenStream,
) -> Result<WorthUiParsedSourceDeclaration, WorthUiParseDiagnostic> {
    let token_keyword = stream.next().expect("token keyword token should exist");
    let name_token = expect_identifier_token(
        module_id,
        source_length,
        stream,
        "token declaration requires an identifier",
    )?;
    expect_punctuation_token(
        module_id,
        source_length,
        stream,
        TokenExpectation::Equals,
        "token declaration requires '=' before its value",
    )?;
    let value_token = expect_string_literal_token(
        module_id,
        source_length,
        stream,
        "token declaration requires a quoted string value",
    )?;
    let semicolon = expect_punctuation_token(
        module_id,
        source_length,
        stream,
        TokenExpectation::Semicolon,
        "token declaration must terminate with ';'",
    )?;

    Ok(WorthUiParsedSourceDeclaration::Token(
        WorthUiParsedTokenDeclaration::new(
            token_identifier_text(&name_token),
            token_string_literal_text(&value_token),
            span_from_bounds(token_keyword.span(), semicolon.span()),
            value_token.span().clone(),
        ),
    ))
}

pub(super) fn parse_block_body_tokens(
    module_id: &WorthUiSourceModuleId,
    stream: &mut WorthUiSourceTokenStream,
    left_brace_span: &WorthUiSourceSpan,
) -> Result<(Vec<WorthUiSourceTokenKind>, WorthUiSourceToken), WorthUiParseDiagnostic> {
    let mut depth = 1usize;
    let mut body_tokens = Vec::new();

    while let Some(token) = stream.next() {
        match token.kind() {
            WorthUiSourceTokenKind::LeftBrace => {
                depth += 1;
                body_tokens.push(token.kind().clone());
            }
            WorthUiSourceTokenKind::RightBrace => {
                depth -= 1;
                if depth == 0 {
                    return Ok((body_tokens, token));
                }
                body_tokens.push(token.kind().clone());
            }
            _ => body_tokens.push(token.kind().clone()),
        }
    }

    Err(WorthUiParseDiagnostic::new(
        WorthUiParseDiagnosticCode::UnterminatedBlock,
        "block declaration reached end of module without a closing '}'",
        WorthUiSourceSpan::new(
            module_id.clone(),
            left_brace_span.start_byte(),
            left_brace_span.end_byte(),
        ),
    ))
}

fn recover_module_root(stream: &mut WorthUiSourceTokenStream) {
    while let Some(token) = stream.peek() {
        if matches!(token.kind(), WorthUiSourceTokenKind::Semicolon) {
            let _ = stream.next();
            break;
        }
        if matches!(
            token.kind(),
            WorthUiSourceTokenKind::KeywordImport
                | WorthUiSourceTokenKind::KeywordComponent
                | WorthUiSourceTokenKind::KeywordControl
                | WorthUiSourceTokenKind::KeywordIntent
                | WorthUiSourceTokenKind::KeywordSurface
                | WorthUiSourceTokenKind::KeywordBinding
                | WorthUiSourceTokenKind::KeywordQueryScalar
                | WorthUiSourceTokenKind::KeywordQueryCollection
                | WorthUiSourceTokenKind::KeywordToken
                | WorthUiSourceTokenKind::KeywordAppearance
                | WorthUiSourceTokenKind::KeywordBackdrop
        ) {
            break;
        }
        let _ = stream.next();
    }
}

#[derive(Clone, Copy)]
enum BlockKind {
    Component,
    Control,
    Intent,
    Portal,
    Focus,
    Motion,
    Command,
    Scroll,
    Selection,
    Surface,
    Binding,
    QueryScalar,
    QueryCollection,
}

fn service_block_kind(keyword: &str) -> Option<BlockKind> {
    Some(match keyword {
        "portal" => BlockKind::Portal,
        "focus" => BlockKind::Focus,
        "motion" => BlockKind::Motion,
        "command" => BlockKind::Command,
        "scroll" => BlockKind::Scroll,
        "selection" => BlockKind::Selection,
        _ => return None,
    })
}
