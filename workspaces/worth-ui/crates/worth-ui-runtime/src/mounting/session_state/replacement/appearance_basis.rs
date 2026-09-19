use super::UiMountedGraphReplacementSuccessor;

impl UiMountedGraphReplacementSuccessor {
    pub(crate) fn appearance_identity_basis(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> Option<crate::mounting::UiMountedIdentityBasis> {
        self.identity
            .projection_instance(instance)
            .map(|view| view.basis().clone())
    }

    pub(crate) fn seal_appearance_receipt_basis(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        incarnation: worth_ui_host_contract::UiMountIncarnation,
        receipts: &crate::mounting::UiMountedNodeReceiptBasis,
    ) -> Result<
        crate::mounting::UiMountedAppearanceReceiptBasis,
        crate::mounting::UiMountedAppearanceReceiptBasisDenial,
    > {
        self.identity
            .seal_appearance_receipt_basis(instance, incarnation, receipts)
    }

    pub(crate) fn validate_appearance_receipt_basis(
        &self,
        basis: crate::mounting::UiMountedAppearanceReceiptBasis,
    ) -> Result<(), crate::mounting::UiMountedAppearanceReceiptBasisDenial> {
        self.identity.validate_appearance_receipt_basis(basis)
    }
}

impl crate::mounting::WorthUiMountedSessionState {
    pub(crate) fn take_replacement_appearance_attempt(
        &mut self,
        attempt: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
    ) -> Option<crate::runtime::appearance::UiAppearanceInspectionAttemptBatch> {
        self.presentation.take_appearance_attempt(attempt)
    }
}
