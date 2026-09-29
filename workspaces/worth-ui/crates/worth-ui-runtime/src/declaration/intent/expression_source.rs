//! An expression one intent use site reads: an operability axis or a
//! payload field.

use worth_ui_dsl::WorthUiExpressionRole;

use crate::runtime::expression::{UiExpressionCatalog, UiExpressionSlot};

/// An expression an intent use site reads. The slot addresses the prepared
/// expression catalog; meaning is the authored identity, so two catalogs that
/// number the same expression differently still compare equal.
#[derive(Clone, Debug)]
pub(crate) struct UiResolvedIntentExpressionSource {
    identity: Box<str>,
    slot: UiExpressionSlot,
}

impl UiResolvedIntentExpressionSource {
    /// The expression named `identity` in the prepared catalog, with its
    /// role, or `None` when the catalog installs no such expression.
    pub(crate) fn resolve(
        identity: &str,
        expressions: &UiExpressionCatalog,
    ) -> Option<(Self, WorthUiExpressionRole)> {
        let slot = expressions.slot_of(identity)?;
        let role = expressions.expression(slot)?.role();
        Some((
            Self {
                identity: identity.into(),
                slot,
            },
            role,
        ))
    }

    pub(crate) fn identity(&self) -> &str {
        &self.identity
    }

    pub(crate) const fn slot(&self) -> UiExpressionSlot {
        self.slot
    }
}

impl PartialEq for UiResolvedIntentExpressionSource {
    fn eq(&self, other: &Self) -> bool {
        self.identity == other.identity
    }
}

impl Eq for UiResolvedIntentExpressionSource {}
