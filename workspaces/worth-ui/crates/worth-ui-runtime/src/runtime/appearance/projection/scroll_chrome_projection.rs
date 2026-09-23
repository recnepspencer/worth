//! The resolved appearance of one scroll-chrome part.
//!
//! Chrome is painted from declared appearance roles exactly as nodes are, but a
//! track or a thumb is not a mounted node: it has no graph node, no receipt and
//! no sealed state vector. What it has is a role and the two interaction
//! classes the pointer lane derived for it, so its projection is the role's
//! aspects resolved at those classes under the surface's active theme, sealed
//! with the same digest discipline a node projection carries.

use super::appearance_projection::{fold, fold_text};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiScrollChromeAppearanceProjection {
    role_digest: u64,
    aspects: Box<[super::UiResolvedAppearanceAspect]>,
    semantic_digest: u64,
}

impl UiScrollChromeAppearanceProjection {
    pub(super) fn seal(
        role: &worth_ui_dsl::UiAppearanceRoleDeclaration,
        classes: &[(
            worth_ui_dsl::UiAppearanceStateAxis,
            worth_ui_dsl::UiAppearanceAxisClass,
        )],
        theme: &super::super::theme::UiThemeResolutionView,
        aspects: Box<[super::UiResolvedAppearanceAspect]>,
    ) -> Self {
        let mut role_digest = 0xcbf2_9ce4_8422_2325_u64;
        role_digest = fold_text(role_digest, role.role().as_str());
        role_digest = fold(role_digest, u64::from(role.schema().revision()));
        role_digest = fold(role_digest, role.revision().value());
        let mut semantic_digest = fold(role_digest, theme.semantic_digest());
        semantic_digest = fold(semantic_digest, classes.len() as u64);
        for (axis, class) in classes {
            semantic_digest = fold(semantic_digest, *axis as u64 + 1);
            semantic_digest = fold(semantic_digest, *class as u64 + 1);
        }
        semantic_digest = fold(semantic_digest, aspects.len() as u64);
        for aspect in &aspects {
            semantic_digest = fold(semantic_digest, aspect.aspect() as u64 + 1);
            semantic_digest = fold(semantic_digest, aspect.semantic_digest());
        }
        Self {
            role_digest,
            aspects,
            semantic_digest,
        }
    }

    /// The declared role and revision this part was painted from, folded to
    /// the transport width the host attribution carries.
    pub(crate) const fn role_digest(&self) -> u64 {
        self.role_digest
    }

    pub(crate) fn aspects(&self) -> &[super::UiResolvedAppearanceAspect] {
        &self.aspects
    }

    /// Changes whenever the role, the theme, the interaction classes or any
    /// resolved aspect value changes, and only then.
    pub(crate) const fn semantic_digest(&self) -> u64 {
        self.semantic_digest
    }
}
