use std::sync::Arc;

use super::{
    WorthQueryArtifactDenial, WorthQueryArtifactDisposition, WorthQueryArtifactHandleCore,
    WorthQueryArtifactHandleGuard, WorthQueryArtifactOwnerSnapshot,
    WorthQueryArtifactSemanticProjection, WorthQueryArtifactTraceMeaning,
    WorthQueryArtifactTransferAdmission, WorthQueryBorrowedArtifactView,
    WorthQueryDisposedArtifact, WorthQueryRetainedArtifactLease,
    WorthQueryTransferredArtifactHandle,
};

pub struct WorthQueryMoveOnlyArtifactHandle {
    pub(super) core: WorthQueryArtifactHandleCore,
}

impl WorthQueryMoveOnlyArtifactHandle {
    pub fn identity(&self) -> &str {
        &self.core.handle_identity
    }

    pub fn occurrence_identity(&self) -> &str {
        &self.core.owner.binding().occurrence_identity
    }

    pub fn semantic_projection(&self) -> &WorthQueryArtifactSemanticProjection {
        self.core.owner.semantic_projection()
    }

    pub fn retained_bytes(&self) -> usize {
        self.core.owner.retained_bytes()
    }

    pub fn owner_snapshot(&self) -> WorthQueryArtifactOwnerSnapshot {
        self.core.owner.snapshot()
    }

    pub fn borrow(
        &self,
        purpose: impl Into<String>,
    ) -> Result<WorthQueryBorrowedArtifactView<'_>, WorthQueryArtifactDenial> {
        WorthQueryBorrowedArtifactView::admit(&self.core.owner, self.core.guard, purpose)
    }

    pub fn retain(
        &self,
        lease_role: impl Into<String>,
    ) -> Result<WorthQueryRetainedArtifactLease, WorthQueryArtifactDenial> {
        WorthQueryRetainedArtifactLease::admit(
            &self.core.owner,
            self.core.owner_generation(),
            lease_role,
        )
    }

    pub fn dispose(mut self) -> Result<WorthQueryDisposedArtifact, WorthQueryArtifactDenial> {
        self.core
            .dispose(WorthQueryArtifactDisposition::Disposed, true)
    }

    pub fn cancel(mut self) -> WorthQueryDisposedArtifact {
        self.core
            .dispose(WorthQueryArtifactDisposition::Cancelled, false)
            .expect("cancelling an owned workflow artifact cannot retain an active borrow")
    }

    pub fn retire_for_trace(mut self) -> WorthQueryDisposedArtifact {
        self.core
            .dispose(WorthQueryArtifactDisposition::Released, false)
            .expect("completed artifact cannot retain an active borrow")
    }

    pub fn transfer(
        mut self,
        admission: &WorthQueryArtifactTransferAdmission,
    ) -> Result<WorthQueryTransferredArtifactHandle, WorthQueryArtifactDenial> {
        self.core.validate_transfer(admission)?;
        let generation = self
            .core
            .owner
            .admit_transfer(self.core.owner_generation())?;
        self.core.active = false;
        Ok(WorthQueryTransferredArtifactHandle {
            core: WorthQueryArtifactHandleCore::new(
                Arc::clone(&self.core.owner),
                admission.consumer_stage.clone(),
                WorthQueryArtifactDisposition::Transferred,
                WorthQueryArtifactHandleGuard::Owner(generation),
            ),
        })
    }

    pub fn lease_for_transfer(
        &self,
        admission: &WorthQueryArtifactTransferAdmission,
        lease_role: impl Into<String>,
    ) -> Result<WorthQueryTransferredArtifactHandle, WorthQueryArtifactDenial> {
        self.core.validate_transfer(admission)?;
        let lease = self.retain(lease_role)?;
        Ok(WorthQueryTransferredArtifactHandle::from_lease(
            lease,
            &admission.consumer_stage,
        ))
    }

    pub fn trace_meaning(&self) -> WorthQueryArtifactTraceMeaning {
        self.core.trace_meaning()
    }

    pub fn contract_matches(
        &self,
        reference: &worth_query_installation::facade::WorthQueryArtifactContractReference,
    ) -> bool {
        self.core.contract_matches(reference)
    }

    pub fn validate_output(
        &self,
        admission: &WorthQueryArtifactTransferAdmission,
    ) -> Result<(), WorthQueryArtifactDenial> {
        self.core.validate_output(admission)?;
        self.core.owner.validate_guard(self.core.guard)
    }

    pub fn validate_replacement(
        &self,
        admission: &super::WorthQueryArtifactProductionAdmission,
    ) -> Result<(), WorthQueryArtifactDenial> {
        self.core.validate_replacement_binding(admission)?;
        self.core.owner.validate_guard(self.core.guard)
    }

    pub fn retire_as_replaced(
        &mut self,
    ) -> Result<WorthQueryDisposedArtifact, WorthQueryArtifactDenial> {
        self.core
            .dispose(WorthQueryArtifactDisposition::Replaced, false)
    }
}

impl std::fmt::Debug for WorthQueryMoveOnlyArtifactHandle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryMoveOnlyArtifactHandle")
            .field("identity", &self.identity())
            .field("occurrence_identity", &self.occurrence_identity())
            .field("retained_bytes", &self.retained_bytes())
            .finish_non_exhaustive()
    }
}
