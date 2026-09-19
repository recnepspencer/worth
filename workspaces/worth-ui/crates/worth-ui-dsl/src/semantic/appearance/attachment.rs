#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiAppearanceRoleAttachmentDeclaration {
    role: super::UiAppearanceRoleIdentity,
    revision: super::UiAppearanceRoleRevision,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAppearanceRoleAttachmentDeclarationDenial {
    DuplicateAttachment,
}

impl UiAppearanceRoleAttachmentDeclaration {
    pub fn new(
        role: super::UiAppearanceRoleIdentity,
        revision: super::UiAppearanceRoleRevision,
    ) -> Self {
        Self { role, revision }
    }

    pub fn role(&self) -> &super::UiAppearanceRoleIdentity {
        &self.role
    }

    pub const fn revision(&self) -> super::UiAppearanceRoleRevision {
        self.revision
    }

    pub(crate) fn fold_source_revision(&self, digest: &mut u64) {
        fold_text(digest, self.role.as_str());
        fold_u64(digest, self.revision.value());
    }

    pub(crate) fn semantic_digest(&self) -> u64 {
        let mut digest = 0xcbf2_9ce4_8422_2325;
        self.fold_source_revision(&mut digest);
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
    use super::*;

    fn attachment(role: &str) -> UiAppearanceRoleAttachmentDeclaration {
        UiAppearanceRoleAttachmentDeclaration::new(
            super::super::UiAppearanceRoleIdentity::new(role).unwrap(),
            super::super::UiAppearanceRoleRevision::new(1).unwrap(),
        )
    }

    #[test]
    fn one_semantic_declaration_cannot_accept_last_writer_attachment() {
        let spec = crate::UiDslSemanticArtifactSpec::new(
            crate::UiDslSemanticKey::new("test.node"),
            crate::UiDslSemanticFamily::Control,
            crate::UiDslSourceProvenance::rust_authored("test/attachment", 0),
        )
        .with_appearance_role_attachment(attachment("test.role.one"))
        .unwrap();

        assert_eq!(
            spec.with_appearance_role_attachment(attachment("test.role.two")),
            Err(UiAppearanceRoleAttachmentDeclarationDenial::DuplicateAttachment)
        );
    }
}
