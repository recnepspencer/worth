use crate::source::WorthUiRustAuthoredArtifactInputModule;

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct WorthUiRustAuthoredArtifactInput {
    modules: Vec<WorthUiRustAuthoredArtifactInputModule>,
}

impl WorthUiRustAuthoredArtifactInput {
    pub fn from_modules(
        modules: impl IntoIterator<Item = WorthUiRustAuthoredArtifactInputModule>,
    ) -> Self {
        Self {
            modules: modules.into_iter().collect(),
        }
    }

    pub(crate) fn modules(&self) -> &[WorthUiRustAuthoredArtifactInputModule] {
        &self.modules
    }

    pub fn source_revision_digest(&self) -> u64 {
        let mut digest = 0xcbf2_9ce4_8422_2325u64;
        fold_text(&mut digest, "worth-ui:rust-authored-input:v1");
        fold_u64(&mut digest, self.modules.len() as u64);
        for module in &self.modules {
            fold_u64(&mut digest, module.source_revision_digest());
        }
        digest
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

#[cfg(test)]
mod tests {
    use crate::source::{WorthUiArtifactInputBodyAtom, WorthUiRustAuthoredArtifactInputModule};

    use super::WorthUiRustAuthoredArtifactInput;

    #[test]
    fn source_revision_digest_tracks_explicit_rust_composition_structure() {
        let baseline = WorthUiRustAuthoredArtifactInput::from_modules([
            WorthUiRustAuthoredArtifactInputModule::new("app/main.wui")
                .with_component_body_atoms_and_authored_identity(
                    "workspace.component.main",
                    "component-main",
                    [WorthUiArtifactInputBodyAtom::Identifier(
                        "workspace.token.background".to_owned(),
                    )],
                ),
        ]);
        let equivalent = baseline.clone();
        let changed_identity = WorthUiRustAuthoredArtifactInput::from_modules([
            WorthUiRustAuthoredArtifactInputModule::new("app/main.wui")
                .with_component_body_atoms_and_authored_identity(
                    "workspace.component.main",
                    "component-main-v2",
                    [WorthUiArtifactInputBodyAtom::Identifier(
                        "workspace.token.background".to_owned(),
                    )],
                ),
        ]);
        let changed_atom = WorthUiRustAuthoredArtifactInput::from_modules([
            WorthUiRustAuthoredArtifactInputModule::new("app/main.wui")
                .with_component_body_atoms_and_authored_identity(
                    "workspace.component.main",
                    "component-main",
                    [WorthUiArtifactInputBodyAtom::StringLiteral(
                        "workspace.token.background".to_owned(),
                    )],
                ),
        ]);

        assert_eq!(
            baseline.source_revision_digest(),
            equivalent.source_revision_digest()
        );
        assert_ne!(
            baseline.source_revision_digest(),
            changed_identity.source_revision_digest()
        );
        assert_ne!(
            baseline.source_revision_digest(),
            changed_atom.source_revision_digest()
        );
    }

    #[test]
    fn source_revision_digest_tracks_appearance_role_attachment() {
        let declaration = |role: &str| {
            crate::WorthUiSemanticArtifactDeclaration::new(
                crate::UiDslSemanticKey::new("test.node"),
                crate::UiDslSemanticFamily::Control,
            )
            .with_appearance_role_attachment(crate::UiAppearanceRoleAttachmentDeclaration::new(
                crate::UiAppearanceRoleIdentity::new(role).unwrap(),
                crate::UiAppearanceRoleRevision::new(1).unwrap(),
            ))
            .unwrap()
            .with_component_reference(
                crate::UiDslComponentReference::new("test.component").unwrap(),
            )
            .unwrap()
        };
        let input = |role| {
            WorthUiRustAuthoredArtifactInput::from_modules([
                WorthUiRustAuthoredArtifactInputModule::new("app/main.wui")
                    .with_semantic_declaration(declaration(role)),
            ])
        };

        assert_ne!(
            input("test.role.one").source_revision_digest(),
            input("test.role.two").source_revision_digest()
        );
    }
}
