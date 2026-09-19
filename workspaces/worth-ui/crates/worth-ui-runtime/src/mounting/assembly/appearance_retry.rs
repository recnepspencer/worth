use super::UiAssembledMountedFrame;

impl UiAssembledMountedFrame {
    pub(in crate::mounting) fn reserve_appearance_retry_basis(&mut self) {
        assert!(self.appearance_retry_basis.is_none());
        // Lowering consumes pending appearance inputs and advances sidecars.
        // Until physical acceptance, a rejected attempt must retain those inputs
        // and its accepted predecessor for a newly admitted presentation attempt.
        self.appearance_retry_basis = Some(self.candidate.owner.clone());
    }

    pub(in crate::mounting) fn restore_rejected_appearance(&mut self) {
        if let Some(basis) = self.appearance_retry_basis.take() {
            self.candidate.owner = basis;
        }
    }
}
