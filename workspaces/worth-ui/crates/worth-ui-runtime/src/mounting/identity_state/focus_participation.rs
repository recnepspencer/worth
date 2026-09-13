use worth_ui_host_contract::UiSemanticSurfaceIdentity;

impl super::UiMountedIdentityState {
    pub(crate) fn focus_participation_snapshot(
        &self,
        surfaces: &[UiSemanticSurfaceIdentity],
    ) -> Option<crate::mounting::UiMountedFocusParticipationSnapshot> {
        Some(self.project_focus_participation(
            self.current_projection.as_ref()?.projection(),
            self.current_receipt_basis.as_ref()?,
            surfaces,
        ))
    }

    pub(in crate::mounting) fn project_focus_participation(
        &self,
        projection: &crate::mounting::UiMountedProjectionFrame,
        receipts: &crate::mounting::UiMountedNodeReceiptBasis,
        surfaces: &[UiSemanticSurfaceIdentity],
    ) -> crate::mounting::UiMountedFocusParticipationSnapshot {
        let retained = self
            .bindings
            .keys()
            .filter(|surface| !surfaces.contains(surface))
            .copied()
            .collect();
        crate::mounting::UiMountedFocusParticipationSnapshot::from_projection(
            projection, receipts, surfaces, retained,
        )
    }
}
