pub(super) enum UiAllocationReplanCommitMode<'a> {
    Ordinary(&'a crate::graph::UiAdmittedReplanNeighborhoodSet),
    Viewport(Box<crate::runtime::UiViewportResizeCommitBasis>),
    DurableResize {
        selection: &'a crate::graph::UiAdmittedReplanNeighborhoodSet,
        basis: crate::runtime::UiResizeAllocationPlanningBasis,
    },
}

impl UiAllocationReplanCommitMode<'_> {
    pub(super) fn selection(&self) -> &crate::graph::UiAdmittedReplanNeighborhoodSet {
        match self {
            Self::Ordinary(selection) => selection,
            Self::Viewport(basis) => basis.selection(),
            Self::DurableResize { selection, .. } => selection,
        }
    }

    pub(super) fn durable_resize(
        &self,
    ) -> Option<&crate::runtime::UiResizeAllocationPlanningBasis> {
        match self {
            Self::DurableResize { basis, .. } => Some(basis),
            Self::Ordinary(_) | Self::Viewport(_) => None,
        }
    }

    pub(super) fn admits_query_measurement_successor(
        &self,
        selected: &crate::evidence::UiAllocationNeighborhoodIdentity,
    ) -> bool {
        self.selection()
            .transaction_basis()
            .consequences()
            .query_measurements()
            .iter()
            .any(|consequence| {
                consequence.neighborhood_identity_digest() == selected.identity_digest()
            })
    }

    pub(super) fn admits_host_measurement_successor(
        &self,
        selected: &crate::evidence::UiAllocationNeighborhoodIdentity,
    ) -> bool {
        self.selection()
            .transaction_basis()
            .consequences()
            .host_measurements()
            .iter()
            .any(|consequence| {
                consequence.neighborhood_identity_digest() == selected.identity_digest()
            })
    }
}
