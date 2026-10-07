/// Retirement evidence may inspect an old source using the current root's
/// format. That does not make the inspected record selected by that root.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum TraversalOrigin {
    RootManifest,
    RetirementProof,
}

impl TraversalOrigin {
    pub(super) fn selected_blob(self, root: u64, current: Option<u64>) -> bool {
        matches!(self, Self::RootManifest) && Some(root) == current
    }
}

#[cfg(test)]
mod tests {
    use super::TraversalOrigin;

    #[test]
    fn held_old_source_tagged_with_current_generation_is_not_a_selected_claim() {
        assert!(TraversalOrigin::RootManifest.selected_blob(10, Some(10)));
        assert!(!TraversalOrigin::RootManifest.selected_blob(9, Some(10)));
        assert!(!TraversalOrigin::RetirementProof.selected_blob(10, Some(10)));
        assert!(!TraversalOrigin::RootManifest.selected_blob(10, None));
    }
}
