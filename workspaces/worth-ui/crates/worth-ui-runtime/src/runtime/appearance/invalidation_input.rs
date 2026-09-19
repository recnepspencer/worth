use super::UiAppearanceInvalidationBatch;

/// Ordinary mounting receives the canonical consumer relation even when no
/// semantic change is pending. Preview and replacement use their own lifecycle.
pub(crate) struct UiAppearanceInvalidationInput<'a> {
    pub(crate) index: &'a crate::graph::UiGraphConsumedFactIndex,
    pub(crate) pending: Option<UiAppearanceInvalidationBatch>,
}
