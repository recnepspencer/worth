use crate::source::{
    WorthUiArtifactInputBlockNode, WorthUiArtifactInputProvenance, WorthUiDslCompileDiagnostic,
    WorthUiDslCompileDiagnosticCode, WorthUiDslCompileStopClass, WorthUiDslSourceSpan,
    WorthUiParsedBlockDeclaration, WorthUiSourceTokenKind,
};

use super::worth_ui_parsed_source_declaration_lowerer::lower_token_kind_to_body_atom;

pub(super) fn lower_component(
    declaration: &WorthUiParsedBlockDeclaration,
    declaration_index: usize,
) -> Result<WorthUiArtifactInputBlockNode, WorthUiDslCompileDiagnostic> {
    let tokens = declaration.body().tokens();
    let appearance_indices = tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| {
            matches!(token, WorthUiSourceTokenKind::KeywordAppearance).then_some(index)
        })
        .collect::<Vec<_>>();
    let (body, attachment) = match appearance_indices.as_slice() {
        [] => (tokens.to_vec(), None),
        [index] => {
            let (end, attachment) = parse_attachment(tokens, *index, declaration)?;
            let mut body = tokens.to_vec();
            body.drain(*index..end);
            (body, Some(attachment))
        }
        _ => {
            return Err(diagnostic(
                declaration,
                "component declares appearance attachment more than once",
            ));
        }
    };
    let node = WorthUiArtifactInputBlockNode::new(
        declaration.name_text(),
        None,
        body.iter().map(lower_token_kind_to_body_atom).collect(),
        WorthUiArtifactInputProvenance::parsed_source(
            declaration.span().clone(),
            None,
            declaration_index,
        ),
    );
    if let Some(attachment) = attachment {
        node.with_appearance_role_attachment(attachment)
            .map_err(|_| diagnostic(declaration, "component has duplicate appearance attachment"))
    } else {
        Ok(node)
    }
}

fn parse_attachment(
    tokens: &[WorthUiSourceTokenKind],
    start: usize,
    declaration: &WorthUiParsedBlockDeclaration,
) -> Result<(usize, crate::UiAppearanceRoleAttachmentDeclaration), WorthUiDslCompileDiagnostic> {
    let mut index = start + 1;
    if tokens.get(index) != Some(&WorthUiSourceTokenKind::LeftBrace) {
        return Err(diagnostic(
            declaration,
            "appearance attachment requires '{'",
        ));
    }
    index += 1;
    if !matches!(tokens.get(index), Some(WorthUiSourceTokenKind::Identifier(value)) if value == "role")
    {
        return Err(diagnostic(
            declaration,
            "appearance attachment requires 'role'",
        ));
    }
    index += 1;
    let Some(WorthUiSourceTokenKind::Identifier(role)) = tokens.get(index) else {
        return Err(diagnostic(
            declaration,
            "appearance attachment requires a role identity",
        ));
    };
    let role = crate::UiAppearanceRoleIdentity::new(role.clone()).ok_or_else(|| {
        diagnostic(
            declaration,
            "appearance attachment role identity is invalid",
        )
    })?;
    index += 1;
    let revision = if matches!(tokens.get(index), Some(WorthUiSourceTokenKind::Identifier(value)) if value == "revision")
    {
        index += 1;
        let Some(WorthUiSourceTokenKind::NumberLiteral(value)) = tokens.get(index) else {
            return Err(diagnostic(
                declaration,
                "appearance attachment revision is invalid",
            ));
        };
        let revision = value
            .parse::<u64>()
            .ok()
            .and_then(crate::UiAppearanceRoleRevision::new)
            .ok_or_else(|| diagnostic(declaration, "appearance attachment revision is invalid"))?;
        index += 1;
        revision
    } else {
        crate::UiAppearanceRoleRevision::new(1).expect("one is a valid revision")
    };
    if tokens.get(index) != Some(&WorthUiSourceTokenKind::RightBrace) {
        return Err(diagnostic(
            declaration,
            "appearance attachment has trailing tokens",
        ));
    }
    let mut end = index + 1;
    while tokens.get(end) == Some(&WorthUiSourceTokenKind::Semicolon) {
        end += 1;
    }
    Ok((
        end,
        crate::UiAppearanceRoleAttachmentDeclaration::new(role, revision),
    ))
}

fn diagnostic(
    declaration: &WorthUiParsedBlockDeclaration,
    message: impl Into<String>,
) -> WorthUiDslCompileDiagnostic {
    let span = declaration.span();
    WorthUiDslCompileDiagnostic::new(
        WorthUiDslCompileDiagnosticCode::InvalidAppearanceAttachment,
        WorthUiDslCompileStopClass::LanguageLegality,
        message,
        Some(span.module_id().as_str().to_owned()),
        Some(WorthUiDslSourceSpan::new(
            span.module_id().as_str(),
            span.start_byte(),
            span.end_byte(),
        )),
    )
}
