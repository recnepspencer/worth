#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiSelectionDelta {
    added: Box<[super::UiSelectionStableKey]>,
    removed: Box<[super::UiSelectionStableKey]>,
    selected_count: usize,
    candidates_visited: u32,
    revision: u64,
    positions: UiSelectionPositionChanges,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct UiSelectionPositions {
    pub(super) anchor: Option<super::UiSelectionStableKey>,
    pub(super) cursor: Option<super::UiSelectionStableKey>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiSelectionPositionChanges {
    previous: UiSelectionPositions,
    current: UiSelectionPositions,
}

impl UiSelectionPositionChanges {
    pub(super) const fn new(previous: UiSelectionPositions, current: UiSelectionPositions) -> Self {
        Self { previous, current }
    }
    pub(crate) const fn previous(self) -> UiSelectionPositions {
        self.previous
    }
    pub(crate) const fn current(self) -> UiSelectionPositions {
        self.current
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiSelectionReconciliationReceipt {
    delta: UiSelectionDelta,
    order_changed: bool,
    missing_keys_preserved_for_partial_catalog: usize,
}

impl UiSelectionDelta {
    pub(super) fn new(
        added: Vec<super::UiSelectionStableKey>,
        removed: Vec<super::UiSelectionStableKey>,
        selected_count: usize,
        candidates_visited: u32,
        revision: u64,
        positions: UiSelectionPositionChanges,
    ) -> Self {
        Self {
            added: added.into_boxed_slice(),
            removed: removed.into_boxed_slice(),
            selected_count,
            candidates_visited,
            revision,
            positions,
        }
    }

    #[cfg(test)]
    pub(crate) fn added(&self) -> &[super::UiSelectionStableKey] {
        &self.added
    }
    pub(crate) fn removed(&self) -> &[super::UiSelectionStableKey] {
        &self.removed
    }
    pub(crate) const fn selected_count(&self) -> usize {
        self.selected_count
    }
    pub(crate) const fn candidates_visited(&self) -> u32 {
        self.candidates_visited
    }
    pub(crate) const fn revision(&self) -> u64 {
        self.revision
    }

    pub(crate) const fn positions(&self) -> UiSelectionPositionChanges {
        self.positions
    }

    pub(in crate::runtime::selection) fn has_same_effect_as(&self, staged: &Self) -> bool {
        self.added == staged.added
            && self.removed == staged.removed
            && self.selected_count == staged.selected_count
            && self.candidates_visited == staged.candidates_visited
            && self.positions == staged.positions
    }
}

impl UiSelectionReconciliationReceipt {
    pub(super) const fn new(
        delta: UiSelectionDelta,
        order_changed: bool,
        missing_keys_preserved_for_partial_catalog: usize,
    ) -> Self {
        Self {
            delta,
            order_changed,
            missing_keys_preserved_for_partial_catalog,
        }
    }

    pub(crate) const fn delta(&self) -> &UiSelectionDelta {
        &self.delta
    }

    pub(in crate::runtime::selection) fn has_same_effect_as(&self, staged: &Self) -> bool {
        self.delta.has_same_effect_as(&staged.delta)
            && self.order_changed == staged.order_changed
            && self.missing_keys_preserved_for_partial_catalog
                == staged.missing_keys_preserved_for_partial_catalog
    }
    #[cfg(test)]
    pub(crate) const fn order_changed(&self) -> bool {
        self.order_changed
    }
    #[cfg(test)]
    pub(crate) const fn missing_keys_preserved_for_partial_catalog(&self) -> usize {
        self.missing_keys_preserved_for_partial_catalog
    }
}
