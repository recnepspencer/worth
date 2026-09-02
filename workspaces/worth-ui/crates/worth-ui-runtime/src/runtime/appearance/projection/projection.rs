#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiAppearanceProjection {
    target: super::super::state::UiAppearanceTarget,
    role: worth_ui_dsl::UiAppearanceRoleIdentity,
    role_schema: worth_ui_dsl::UiAppearanceRoleSchemaVersion,
    role_revision: worth_ui_dsl::UiAppearanceRoleRevision,
    theme: Box<str>,
    theme_revision: u64,
    catalog_revision: u64,
    state: super::super::state::UiAppearanceStateVector,
    aspects: Box<[super::UiResolvedAppearanceAspect]>,
    semantic_digest: u64,
}

impl UiAppearanceProjection {
    pub(crate) fn seal(
        target: &super::super::state::UiAppearanceTarget,
        role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
        state: super::super::state::UiAppearanceStateVector,
        theme: &super::super::theme::UiThemeResolutionView,
        aspects: Box<[super::UiResolvedAppearanceAspect]>,
    ) -> Self {
        let mut semantic_digest = 0xcbf2_9ce4_8422_2325_u64;
        semantic_digest = fold_text(semantic_digest, role.role().as_str());
        semantic_digest = fold(semantic_digest, u64::from(role.schema().revision()));
        semantic_digest = fold(semantic_digest, role.revision().value());
        semantic_digest = fold(semantic_digest, theme.semantic_digest());
        semantic_digest = fold(semantic_digest, state.semantic_digest());
        semantic_digest = fold(semantic_digest, aspects.len() as u64);
        for aspect in &aspects {
            semantic_digest = fold(semantic_digest, aspect.aspect() as u64 + 1);
            semantic_digest = fold(semantic_digest, aspect.semantic_digest());
        }
        Self {
            target: target.clone(),
            role: role.role().clone(),
            role_schema: role.schema(),
            role_revision: role.revision(),
            theme: theme.definition_identity().into(),
            theme_revision: theme.definition_revision(),
            catalog_revision: theme.catalog_revision(),
            state,
            aspects,
            semantic_digest,
        }
    }

    pub(crate) fn target(&self) -> super::super::state::UiAppearanceTarget {
        self.target.clone()
    }
    pub(crate) fn role(&self) -> &worth_ui_dsl::UiAppearanceRoleIdentity {
        &self.role
    }
    pub(crate) const fn role_schema(&self) -> worth_ui_dsl::UiAppearanceRoleSchemaVersion {
        self.role_schema
    }
    pub(crate) const fn role_revision(&self) -> worth_ui_dsl::UiAppearanceRoleRevision {
        self.role_revision
    }
    pub(crate) fn theme(&self) -> &str {
        &self.theme
    }
    pub(crate) const fn theme_revision(&self) -> u64 {
        self.theme_revision
    }
    pub(crate) const fn catalog_revision(&self) -> u64 {
        self.catalog_revision
    }
    pub(crate) const fn state(&self) -> &super::super::state::UiAppearanceStateVector {
        &self.state
    }
    pub(crate) fn aspects(&self) -> &[super::UiResolvedAppearanceAspect] {
        &self.aspects
    }
    pub(crate) const fn semantic_digest(&self) -> u64 {
        self.semantic_digest
    }
    pub(crate) fn exactly_equivalent(&self, other: &Self) -> bool {
        self == other
    }

    pub(crate) fn physical_output_equivalent(&self, other: &Self) -> bool {
        self.role == other.role
            && self.role_schema == other.role_schema
            && self.role_revision == other.role_revision
            && self.aspects.len() == other.aspects.len()
            && self.aspects.iter().all(|left| {
                other
                    .aspects
                    .iter()
                    .find(|right| right.aspect() == left.aspect())
                    .is_some_and(|right| {
                        right.value() == left.value()
                            && right.support() == left.support()
                            && right.provenance().selected_slot()
                                == left.provenance().selected_slot()
                            && right.provenance().terminal_slot()
                                == left.provenance().terminal_slot()
                    })
            })
    }
}

fn fold(digest: u64, value: u64) -> u64 {
    digest.wrapping_mul(0x0000_0100_0000_01b3) ^ value
}

fn fold_text(mut digest: u64, value: &str) -> u64 {
    digest = fold(digest, value.len() as u64);
    for byte in value.as_bytes() {
        digest = fold(digest, u64::from(*byte));
    }
    digest
}
