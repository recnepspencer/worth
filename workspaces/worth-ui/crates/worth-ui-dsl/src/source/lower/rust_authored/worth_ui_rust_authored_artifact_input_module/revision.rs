use crate::source::WorthUiArtifactInputBodyAtom;

use super::WorthUiRustAuthoredDeclaration;

pub(super) fn module_digest(
    relative_module_path: &str,
    declarations: &[WorthUiRustAuthoredDeclaration],
) -> u64 {
    let mut digest = 0xcbf2_9ce4_8422_2325u64;
    fold_text(&mut digest, "worth-ui:rust-authored-module:v1");
    fold_text(&mut digest, relative_module_path);
    fold_u64(&mut digest, declarations.len() as u64);
    for declaration in declarations {
        fold_declaration(&mut digest, declaration);
    }
    digest
}

fn fold_declaration(digest: &mut u64, declaration: &WorthUiRustAuthoredDeclaration) {
    match declaration {
        WorthUiRustAuthoredDeclaration::Import { target_module_path } => {
            fold_text(digest, "import");
            fold_text(digest, target_module_path);
        }
        WorthUiRustAuthoredDeclaration::Component {
            name_text,
            authored_identity,
            body_atoms,
            appearance_role_attachment,
        } => {
            fold_block(
                digest,
                "component",
                name_text,
                authored_identity.as_deref(),
                body_atoms,
            );
            fold_attachment(digest, appearance_role_attachment.as_ref());
        }
        WorthUiRustAuthoredDeclaration::Surface {
            name_text,
            authored_identity,
            body_atoms,
        } => fold_block(
            digest,
            "surface",
            name_text,
            authored_identity.as_deref(),
            body_atoms,
        ),
        WorthUiRustAuthoredDeclaration::Binding {
            name_text,
            authored_identity,
            body_atoms,
        } => fold_block(
            digest,
            "binding",
            name_text,
            authored_identity.as_deref(),
            body_atoms,
        ),
        WorthUiRustAuthoredDeclaration::QueryScalar {
            name_text,
            body_atoms,
        } => fold_block(digest, "query-scalar", name_text, None, body_atoms),
        WorthUiRustAuthoredDeclaration::QueryCollection {
            name_text,
            body_atoms,
        } => fold_block(digest, "query-collection", name_text, None, body_atoms),
        WorthUiRustAuthoredDeclaration::Token {
            name_text,
            authored_identity,
            value_text,
        } => {
            fold_text(digest, "token");
            fold_text(digest, name_text);
            fold_optional_text(digest, authored_identity.as_deref());
            fold_text(digest, value_text);
        }
        WorthUiRustAuthoredDeclaration::SemanticArtifact(declaration) => {
            fold_text(digest, "semantic-artifact");
            declaration.fold_source_revision(digest);
        }
        WorthUiRustAuthoredDeclaration::AppearanceRole(role) => {
            fold_text(digest, "appearance-role");
            fold_text(digest, role.role().as_str());
            fold_u64(digest, role.revision().value());
            fold_u64(digest, role.canonical_bytes().len() as u64);
            for byte in role.canonical_bytes() {
                fold_u64(digest, u64::from(byte));
            }
        }
        WorthUiRustAuthoredDeclaration::Backdrop(declaration) => {
            fold_text(digest, "backdrop");
            let bytes = declaration.canonical_bytes();
            fold_u64(digest, bytes.len() as u64);
            for byte in bytes {
                fold_u64(digest, u64::from(byte));
            }
        }
    }
}

fn fold_attachment(
    digest: &mut u64,
    attachment: Option<&crate::UiAppearanceRoleAttachmentDeclaration>,
) {
    match attachment {
        Some(attachment) => {
            fold_text(digest, "attachment");
            fold_text(digest, attachment.role().as_str());
            fold_u64(digest, attachment.revision().value());
        }
        None => fold_text(digest, "no-attachment"),
    }
}

fn fold_block(
    digest: &mut u64,
    kind: &str,
    name_text: &str,
    authored_identity: Option<&str>,
    body_atoms: &[WorthUiArtifactInputBodyAtom],
) {
    fold_text(digest, kind);
    fold_text(digest, name_text);
    fold_optional_text(digest, authored_identity);
    fold_u64(digest, body_atoms.len() as u64);
    for atom in body_atoms {
        fold_atom(digest, atom);
    }
}

fn fold_atom(digest: &mut u64, atom: &WorthUiArtifactInputBodyAtom) {
    let token = match atom {
        WorthUiArtifactInputBodyAtom::Identifier(value) => {
            fold_text(digest, "identifier");
            return fold_text(digest, value);
        }
        WorthUiArtifactInputBodyAtom::StringLiteral(value) => {
            fold_text(digest, "string-literal");
            return fold_text(digest, value);
        }
        WorthUiArtifactInputBodyAtom::KeywordImport => "keyword-import",
        WorthUiArtifactInputBodyAtom::KeywordComponent => "keyword-component",
        WorthUiArtifactInputBodyAtom::KeywordControl => "keyword-control",
        WorthUiArtifactInputBodyAtom::KeywordIntent => "keyword-intent",
        WorthUiArtifactInputBodyAtom::KeywordSurface => "keyword-surface",
        WorthUiArtifactInputBodyAtom::KeywordBinding => "keyword-binding",
        WorthUiArtifactInputBodyAtom::KeywordQueryScalar => "keyword-query-scalar",
        WorthUiArtifactInputBodyAtom::KeywordQueryCollection => "keyword-query-collection",
        WorthUiArtifactInputBodyAtom::KeywordToken => "keyword-token",
        WorthUiArtifactInputBodyAtom::KeywordAppearance => "keyword-appearance",
        WorthUiArtifactInputBodyAtom::KeywordBackdrop => "keyword-backdrop",
        WorthUiArtifactInputBodyAtom::NumberLiteral(value) => {
            fold_text(digest, "number-literal");
            return fold_text(digest, value);
        }
        WorthUiArtifactInputBodyAtom::LeftBrace => "left-brace",
        WorthUiArtifactInputBodyAtom::RightBrace => "right-brace",
        WorthUiArtifactInputBodyAtom::LeftBracket => "left-bracket",
        WorthUiArtifactInputBodyAtom::RightBracket => "right-bracket",
        WorthUiArtifactInputBodyAtom::LeftParen => "left-paren",
        WorthUiArtifactInputBodyAtom::RightParen => "right-paren",
        WorthUiArtifactInputBodyAtom::Comma => "comma",
        WorthUiArtifactInputBodyAtom::Semicolon => "semicolon",
        WorthUiArtifactInputBodyAtom::Equals => "equals",
        WorthUiArtifactInputBodyAtom::Plus => "plus",
    };
    fold_text(digest, token);
}

fn fold_optional_text(digest: &mut u64, value: Option<&str>) {
    match value {
        Some(value) => {
            fold_text(digest, "some");
            fold_text(digest, value);
        }
        None => fold_text(digest, "none"),
    }
}

fn fold_text(digest: &mut u64, text: &str) {
    fold_u64(digest, text.len() as u64);
    for byte in text.as_bytes() {
        *digest ^= u64::from(*byte);
        *digest = digest.wrapping_mul(0x100_0000_01b3);
    }
}

fn fold_u64(digest: &mut u64, value: u64) {
    for byte in value.to_le_bytes() {
        *digest ^= u64::from(byte);
        *digest = digest.wrapping_mul(0x100_0000_01b3);
    }
}
