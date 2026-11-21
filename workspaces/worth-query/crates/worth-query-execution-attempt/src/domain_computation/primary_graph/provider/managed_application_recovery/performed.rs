//! Performed native recovery retains its original result and cleanup custody.


use crate::domain_computation::primary_graph::WorthQueryAdmittedIdempotencyRead;
use super::*;
use crate::domain_computation::WorthQueryProviderSessionFailure;
/// A performed World successor has been handed to the normal Query pending
/// publication owner before this result is returned. A failed reply or pending
/// projection remains visible in the admitted idempotency read's posture.
pub struct WorthQueryManagedApplicationRecoveryPerformed {
    pub(super) read:
        Result<WorthQueryAdmittedIdempotencyRead, WorthQueryApplicationIdempotencyResolutionDenial>,
    pub(super) publication_failure: Option<WorthQueryProviderSessionFailure>,
    pub(super) prior_cleanup_failure: Option<WorthQueryProductUnpublishedRecoveryReleaseFailure>,
    pub(super) pending_cleanup: Option<WorthQueryProductUnpublishedRecovery>,
    pub(super) original_owner: worth_runtime_world::facade::RuntimeWorldOwnerIdentity,
    pub(super) original_world_commit: u64,
    pub(super) original_native_commit: u64,
    pub(super) original_native_version: u64,
    pub(super) original_key: [u8; 32],
    pub(super) conditional_definition_identity: Option<[u8; 32]>,
}

impl WorthQueryManagedApplicationRecoveryPerformed {
    pub fn matches_original_receipt(
        &self,
        _: &dyn worth_query_execution_contracts::facade::WorthQueryProgramPublicationAccessPort,
        receipt: &super::super::WorthQueryRecoveredApplicationCommitReceipt,
    ) -> bool {
        use worth_foundational::facade::RecordedProductBranchReference;
        let same_world = match receipt.new_product_head() {
            RecordedProductBranchReference::Home(head) => {
                self.original_owner.durable() == Some(head.branch().owner())
                    && head.selected_commit().ordinal() == self.original_world_commit
            }
            RecordedProductBranchReference::Memory(head) => {
                self.original_owner.durable().is_none()
                    && head.owner() == self.original_owner.get()
                    && head.selected_commit() == self.original_world_commit
            }
        };
        same_world
            && receipt.idempotency_key_identity() == &self.original_key
            && receipt.commit_reference().commit_id.0 == self.original_native_commit
            && receipt.commit_reference().version_id.as_u64() == self.original_native_version
    }

    pub fn bind_performed_definition(
        &self,
        _: &dyn worth_query_execution_contracts::facade::WorthQueryProgramPublicationAccessPort,
        fresh: crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding,
    ) -> crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding {
        self.bind_original_definition(fresh)
    }

    pub fn bind_original_definition(
        &self,
        fresh: crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding,
    ) -> crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding {
        if fresh.conditional_definition_identity().is_none() {
            fresh.bind_conditional_definition(self.conditional_definition_identity.as_ref())
        } else {
            fresh
        }
    }

    pub fn take_pending_cleanup(
        &mut self,
        _: &dyn worth_query_execution_contracts::facade::WorthQueryProgramPublicationAccessPort,
    ) -> Option<WorthQueryProductUnpublishedRecovery> {
        self.take_original_cleanup()
    }

    pub fn take_original_cleanup(&mut self) -> Option<WorthQueryProductUnpublishedRecovery> {
        self.pending_cleanup.take()
    }

    pub fn take_prior_cleanup_failure(
        &mut self,
        _: &dyn worth_query_execution_contracts::facade::WorthQueryProgramPublicationAccessPort,
    ) -> Option<WorthQueryProductUnpublishedRecoveryReleaseFailure> {
        self.take_original_cleanup_failure()
    }

    pub fn take_original_cleanup_failure(
        &mut self,
    ) -> Option<WorthQueryProductUnpublishedRecoveryReleaseFailure> {
        self.prior_cleanup_failure.take()
    }

    pub fn into_parts(
        self,
    ) -> (
        Result<WorthQueryAdmittedIdempotencyRead, WorthQueryApplicationIdempotencyResolutionDenial>,
        Option<WorthQueryProviderSessionFailure>,
    ) {
        (self.read, self.publication_failure)
    }
}


#[doc(hidden)]
pub struct WorthQueryManagedApplicationRecoveryPerformedAttemptParts {
    pub read: Result<WorthQueryAdmittedIdempotencyRead, WorthQueryApplicationIdempotencyResolutionDenial>,
    pub publication_failure: Option<WorthQueryProviderSessionFailure>,
    pub prior_cleanup_failure: Option<WorthQueryProductUnpublishedRecoveryReleaseFailure>,
    pub pending_cleanup: Option<WorthQueryProductUnpublishedRecovery>,
    pub original_owner: worth_runtime_world::facade::RuntimeWorldOwnerIdentity,
    pub original_world_commit: u64,
    pub original_native_commit: u64,
    pub original_native_version: u64,
    pub original_key: [u8; 32],
    pub conditional_definition_identity: Option<[u8; 32]>,
}

impl WorthQueryManagedApplicationRecoveryPerformed {
    #[doc(hidden)]
    pub fn from_attempt_parts(
        _seal: crate::ports::WorthQueryAttemptCrateSeal,
        parts: WorthQueryManagedApplicationRecoveryPerformedAttemptParts,
    ) -> Self {
        Self {
            read: parts.read,
            publication_failure: parts.publication_failure,
            prior_cleanup_failure: parts.prior_cleanup_failure,
            pending_cleanup: parts.pending_cleanup,
            original_owner: parts.original_owner,
            original_world_commit: parts.original_world_commit,
            original_native_commit: parts.original_native_commit,
            original_native_version: parts.original_native_version,
            original_key: parts.original_key,
            conditional_definition_identity: parts.conditional_definition_identity,
        }
    }
}
