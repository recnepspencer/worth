use std::sync::Arc;

use super::{
    WorthQueryArtifactDenial, WorthQueryArtifactDisposition, WorthQueryArtifactHandleCore,
    WorthQueryArtifactHandleGuard, WorthQueryArtifactOwnerSnapshot,
    WorthQueryArtifactSemanticProjection, WorthQueryArtifactTraceMeaning,
    WorthQueryArtifactTransferAdmission, WorthQueryBorrowedArtifactView,
    WorthQueryMoveOnlyArtifactHandle, WorthQueryRetainedArtifactLease,
};

pub struct WorthQueryTransferredArtifactHandle {
    pub(super) core: WorthQueryArtifactHandleCore,
}

impl WorthQueryTransferredArtifactHandle {
    pub(super) fn from_lease(
        mut lease: WorthQueryRetainedArtifactLease,
        holder_stage: &str,
    ) -> Self {
        lease.active = false;
        Self {
            core: WorthQueryArtifactHandleCore::new(
                Arc::clone(&lease.owner),
                holder_stage.to_owned(),
                WorthQueryArtifactDisposition::Leased,
                WorthQueryArtifactHandleGuard::Lease(lease.lease_generation),
            ),
        }
    }

    pub fn identity(&self) -> &str {
        &self.core.handle_identity
    }

    pub fn occurrence_identity(&self) -> &str {
        &self.core.owner.binding().occurrence_identity
    }

    pub fn semantic_projection(&self) -> &WorthQueryArtifactSemanticProjection {
        self.core.owner.semantic_projection()
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

    pub fn into_owned_output(mut self) -> Result<WorthQueryMoveOnlyArtifactHandle, Self> {
        if !matches!(self.core.guard, WorthQueryArtifactHandleGuard::Owner(_)) {
            return Err(self);
        }
        self.core.active = false;
        Ok(WorthQueryMoveOnlyArtifactHandle {
            core: WorthQueryArtifactHandleCore::new(
                Arc::clone(&self.core.owner),
                self.core.holder_stage.clone(),
                self.core.disposition,
                self.core.guard,
            ),
        })
    }

    pub fn trace_meaning(&self) -> WorthQueryArtifactTraceMeaning {
        self.core.trace_meaning()
    }

    pub fn validate_input(
        &self,
        admission: &WorthQueryArtifactTransferAdmission,
    ) -> Result<(), WorthQueryArtifactDenial> {
        self.core.validate_transfer(admission)?;
        self.core.owner.validate_guard(self.core.guard)
    }

    pub fn contract_matches(
        &self,
        reference: &worth_query_installation::facade::WorthQueryArtifactContractReference,
    ) -> bool {
        self.core.contract_matches(reference)
    }
}

impl std::fmt::Debug for WorthQueryTransferredArtifactHandle {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryTransferredArtifactHandle")
            .field("identity", &self.identity())
            .field("occurrence_identity", &self.occurrence_identity())
            .finish_non_exhaustive()
    }
}
