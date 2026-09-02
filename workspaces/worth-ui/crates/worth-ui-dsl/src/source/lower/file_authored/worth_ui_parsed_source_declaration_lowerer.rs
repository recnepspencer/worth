use crate::source::{
    WorthUiArtifactInputBlockNode, WorthUiArtifactInputBodyAtom, WorthUiArtifactInputImportNode,
    WorthUiArtifactInputNode, WorthUiArtifactInputProvenance, WorthUiArtifactInputReference,
    WorthUiArtifactInputSemanticArtifactNode, WorthUiArtifactInputTokenNode,
    WorthUiDslCompileDiagnostic, WorthUiDslCompileDiagnosticCode, WorthUiDslCompileStopClass,
    WorthUiDslSourceSpan, WorthUiParsedBlockBody, WorthUiParsedSourceDeclaration,
    WorthUiSourceTokenKind,
};

use super::appearance_declaration_lowerer::lower_role;
use super::backdrop_declaration_lowerer::lower_backdrop;
use super::component_appearance_attachment::lower_component;
use super::WorthUiFileAuthoredLoweredDeclaration;

pub(crate) fn lower_parsed_source_declaration(
    declaration: &WorthUiParsedSourceDeclaration,
    declaration_index: usize,
) -> Result<WorthUiFileAuthoredLoweredDeclaration, WorthUiDslCompileDiagnostic> {
    let node = match declaration {
        WorthUiParsedSourceDeclaration::Import(import_declaration) => {
            WorthUiArtifactInputNode::Import(lower_import_declaration(
                import_declaration,
                declaration_index,
            ))
        }
        WorthUiParsedSourceDeclaration::Component(block_declaration) => {
            WorthUiArtifactInputNode::Component(lower_component(
                block_declaration,
                declaration_index,
            )?)
        }
        WorthUiParsedSourceDeclaration::Control(block_declaration) => {
            WorthUiArtifactInputNode::Component(lower_component(
                block_declaration,
                declaration_index,
            )?)
        }
        WorthUiParsedSourceDeclaration::Intent(block_declaration) => {
            lower_intent_declaration(block_declaration, declaration_index)?
        }
        WorthUiParsedSourceDeclaration::Portal(block_declaration) => lower_service_declaration(
            block_declaration,
            declaration_index,
            crate::WorthUiServiceFamily::Portal,
        )?,
        WorthUiParsedSourceDeclaration::Focus(block_declaration) => lower_service_declaration(
            block_declaration,
            declaration_index,
            crate::WorthUiServiceFamily::Focus,
        )?,
        WorthUiParsedSourceDeclaration::Motion(block_declaration) => lower_service_declaration(
            block_declaration,
            declaration_index,
            crate::WorthUiServiceFamily::Motion,
        )?,
        WorthUiParsedSourceDeclaration::Command(block_declaration) => lower_service_declaration(
            block_declaration,
            declaration_index,
            crate::WorthUiServiceFamily::CommandRouting,
        )?,
        WorthUiParsedSourceDeclaration::Scroll(block_declaration) => lower_service_declaration(
            block_declaration,
            declaration_index,
            crate::WorthUiServiceFamily::Scroll,
        )?,
        WorthUiParsedSourceDeclaration::Selection(block_declaration) => lower_service_declaration(
            block_declaration,
            declaration_index,
            crate::WorthUiServiceFamily::Selection,
        )?,
        WorthUiParsedSourceDeclaration::Surface(block_declaration) => {
            WorthUiArtifactInputNode::Surface(lower_parsed_block_declaration(
                block_declaration,
                declaration_index,
            ))
        }
        WorthUiParsedSourceDeclaration::Binding(block_declaration) => {
            WorthUiArtifactInputNode::Binding(lower_parsed_block_declaration(
                block_declaration,
                declaration_index,
            ))
        }
        WorthUiParsedSourceDeclaration::QueryScalar(block_declaration) => {
            WorthUiArtifactInputNode::QueryScalar(lower_parsed_block_declaration(
                block_declaration,
                declaration_index,
            ))
        }
        WorthUiParsedSourceDeclaration::QueryCollection(block_declaration) => {
            WorthUiArtifactInputNode::QueryCollection(lower_parsed_block_declaration(
                block_declaration,
                declaration_index,
            ))
        }
        WorthUiParsedSourceDeclaration::Token(token_declaration) => {
            WorthUiArtifactInputNode::Token(lower_token_declaration(
                token_declaration,
                declaration_index,
            ))
        }
        WorthUiParsedSourceDeclaration::AppearanceRole(role) => {
            return lower_role(role, declaration_index)
                .map(WorthUiFileAuthoredLoweredDeclaration::Artifact);
        }
        WorthUiParsedSourceDeclaration::Backdrop(backdrop) => {
            return lower_backdrop(backdrop, declaration_index).map(|(source, provenance)| {
                WorthUiFileAuthoredLoweredDeclaration::Backdrop { source, provenance }
            });
        }
    };
    Ok(WorthUiFileAuthoredLoweredDeclaration::Artifact(node))
}

fn lower_service_declaration(
    declaration: &crate::source::WorthUiParsedBlockDeclaration,
    declaration_index: usize,
    family: crate::WorthUiServiceFamily,
) -> Result<WorthUiArtifactInputNode, WorthUiDslCompileDiagnostic> {
    let provenance = WorthUiArtifactInputProvenance::parsed_source(
        declaration.span().clone(),
        None,
        declaration_index,
    );
    let body = lower_parsed_block_body(declaration.body());
    let service =
        crate::WorthUiServiceDeclarationMeaning::parse(family, declaration.name_text(), &body)
            .map_err(|error| service_diagnostic(declaration, error))?;
    let semantic = crate::WorthUiSemanticArtifactDeclaration::new(
        crate::UiDslSemanticKey::new(declaration.name_text()),
        crate::UiDslSemanticFamily::RuntimeService,
    )
    .with_service_declaration(service);
    Ok(WorthUiArtifactInputNode::SemanticArtifact(
        WorthUiArtifactInputSemanticArtifactNode::new(semantic, provenance),
    ))
}

fn service_diagnostic(
    declaration: &crate::source::WorthUiParsedBlockDeclaration,
    error: crate::WorthUiServiceDeclarationParseError,
) -> WorthUiDslCompileDiagnostic {
    let span = declaration.span();
    WorthUiDslCompileDiagnostic::new(
        WorthUiDslCompileDiagnosticCode::InvalidServiceDeclaration,
        WorthUiDslCompileStopClass::LanguageLegality,
        error.detail(),
        Some(span.module_id().as_str().to_owned()),
        Some(WorthUiDslSourceSpan::new(
            span.module_id().as_str(),
            span.start_byte(),
            span.end_byte(),
        )),
    )
}

fn lower_import_declaration(
    declaration: &crate::source::WorthUiParsedImportDeclaration,
    declaration_index: usize,
) -> WorthUiArtifactInputImportNode {
    WorthUiArtifactInputImportNode::new(
        WorthUiArtifactInputReference::new(declaration.path_text()),
        WorthUiArtifactInputProvenance::parsed_source(
            declaration.span().clone(),
            None,
            declaration_index,
        ),
    )
}

fn lower_intent_declaration(
    declaration: &crate::source::WorthUiParsedBlockDeclaration,
    declaration_index: usize,
) -> Result<WorthUiArtifactInputNode, WorthUiDslCompileDiagnostic> {
    let provenance = WorthUiArtifactInputProvenance::parsed_source(
        declaration.span().clone(),
        None,
        declaration_index,
    );
    let body = lower_parsed_block_body(declaration.body());
    let semantic =
        crate::WorthUiIntentDeclarationSpec::parse_file_authored(declaration.name_text(), &body)
            .map_err(|error| intent_diagnostic(declaration, error))?
            .into_semantic_declaration();
    Ok(WorthUiArtifactInputNode::SemanticArtifact(
        WorthUiArtifactInputSemanticArtifactNode::new(semantic, provenance),
    ))
}

fn lower_token_declaration(
    declaration: &crate::source::WorthUiParsedTokenDeclaration,
    declaration_index: usize,
) -> WorthUiArtifactInputTokenNode {
    WorthUiArtifactInputTokenNode::new(
        declaration.name_text(),
        None,
        declaration.value_text(),
        WorthUiArtifactInputProvenance::parsed_source(
            declaration.span().clone(),
            Some(declaration.value_span().clone()),
            declaration_index,
        ),
    )
}

fn intent_diagnostic(
    declaration: &crate::source::WorthUiParsedBlockDeclaration,
    error: crate::WorthUiIntentDeclarationParseError,
) -> WorthUiDslCompileDiagnostic {
    let span = declaration.span();
    WorthUiDslCompileDiagnostic::new(
        WorthUiDslCompileDiagnosticCode::InvalidIntentDeclaration,
        WorthUiDslCompileStopClass::LanguageLegality,
        error.detail(),
        Some(span.module_id().as_str().to_owned()),
        Some(WorthUiDslSourceSpan::new(
            span.module_id().as_str(),
            span.start_byte(),
            span.end_byte(),
        )),
    )
}

fn lower_parsed_block_declaration(
    block_declaration: &crate::source::WorthUiParsedBlockDeclaration,
    declaration_index: usize,
) -> WorthUiArtifactInputBlockNode {
    WorthUiArtifactInputBlockNode::new(
        block_declaration.name_text(),
        None,
        lower_parsed_block_body(block_declaration.body()),
        WorthUiArtifactInputProvenance::parsed_source(
            block_declaration.span().clone(),
            None,
            declaration_index,
        ),
    )
}

fn lower_parsed_block_body(body: &WorthUiParsedBlockBody) -> Vec<WorthUiArtifactInputBodyAtom> {
    body.tokens()
        .iter()
        .map(lower_token_kind_to_body_atom)
        .collect()
}

pub(super) fn lower_token_kind_to_body_atom(
    token_kind: &WorthUiSourceTokenKind,
) -> WorthUiArtifactInputBodyAtom {
    match token_kind {
        WorthUiSourceTokenKind::Identifier(text) => {
            WorthUiArtifactInputBodyAtom::Identifier(text.clone())
        }
        WorthUiSourceTokenKind::StringLiteral(text) => {
            WorthUiArtifactInputBodyAtom::StringLiteral(text.clone())
        }
        WorthUiSourceTokenKind::KeywordImport => WorthUiArtifactInputBodyAtom::KeywordImport,
        WorthUiSourceTokenKind::KeywordComponent => WorthUiArtifactInputBodyAtom::KeywordComponent,
        WorthUiSourceTokenKind::KeywordControl => WorthUiArtifactInputBodyAtom::KeywordControl,
        WorthUiSourceTokenKind::KeywordIntent => WorthUiArtifactInputBodyAtom::KeywordIntent,
        WorthUiSourceTokenKind::KeywordSurface => WorthUiArtifactInputBodyAtom::KeywordSurface,
        WorthUiSourceTokenKind::KeywordBinding => WorthUiArtifactInputBodyAtom::KeywordBinding,
        WorthUiSourceTokenKind::KeywordQueryScalar => {
            WorthUiArtifactInputBodyAtom::KeywordQueryScalar
        }
        WorthUiSourceTokenKind::KeywordQueryCollection => {
            WorthUiArtifactInputBodyAtom::KeywordQueryCollection
        }
        WorthUiSourceTokenKind::KeywordToken => WorthUiArtifactInputBodyAtom::KeywordToken,
        WorthUiSourceTokenKind::KeywordAppearance => {
            WorthUiArtifactInputBodyAtom::KeywordAppearance
        }
        WorthUiSourceTokenKind::KeywordBackdrop => WorthUiArtifactInputBodyAtom::KeywordBackdrop,
        WorthUiSourceTokenKind::NumberLiteral(value) => {
            WorthUiArtifactInputBodyAtom::NumberLiteral(value.clone())
        }
        WorthUiSourceTokenKind::LeftBrace => WorthUiArtifactInputBodyAtom::LeftBrace,
        WorthUiSourceTokenKind::RightBrace => WorthUiArtifactInputBodyAtom::RightBrace,
        WorthUiSourceTokenKind::LeftBracket => WorthUiArtifactInputBodyAtom::LeftBracket,
        WorthUiSourceTokenKind::RightBracket => WorthUiArtifactInputBodyAtom::RightBracket,
        WorthUiSourceTokenKind::LeftParen => WorthUiArtifactInputBodyAtom::LeftParen,
        WorthUiSourceTokenKind::RightParen => WorthUiArtifactInputBodyAtom::RightParen,
        WorthUiSourceTokenKind::Comma => WorthUiArtifactInputBodyAtom::Comma,
        WorthUiSourceTokenKind::Semicolon => WorthUiArtifactInputBodyAtom::Semicolon,
        WorthUiSourceTokenKind::Equals => WorthUiArtifactInputBodyAtom::Equals,
        WorthUiSourceTokenKind::Plus => WorthUiArtifactInputBodyAtom::Plus,
    }
}
