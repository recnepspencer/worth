use std::collections::BTreeSet;

use crate::runtime::expression::UiExpressionSlot;

/// The expressions one settle changed, reported where the owner counts each
/// published change. A caller that consumes results passes it on to the
/// consumers of those slots; nothing is retained for a later drain.
#[must_use = "a settlement names the consumers that must re-observe"]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct UiExpressionSettlement {
    changed: BTreeSet<UiExpressionSlot>,
}

impl UiExpressionSettlement {
    pub(super) const fn new(changed: BTreeSet<UiExpressionSlot>) -> Self {
        Self { changed }
    }

    pub(crate) fn changed_slots(&self) -> impl Iterator<Item = UiExpressionSlot> + '_ {
        self.changed.iter().copied()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.changed.is_empty()
    }
}
