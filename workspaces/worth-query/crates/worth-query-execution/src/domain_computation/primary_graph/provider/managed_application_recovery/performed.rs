//! Performed native recovery retains its original result and cleanup custody.
use super::*;
use crate::domain_computation::application_aftermath::WorthQueryAdmittedIdempotencyRead;
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
    /// Match actual retained recovery custody to original owner data. No live
    /// receipt or publication witness is constructed by this comparison.
    pub fn matches_original_receipt(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
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

    /// Carries the performed handler's definition into a fresh-authorized read.
    /// Every other fresh component, and any explicit definition, stays intact.
    pub fn bind_performed_definition(
        &self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
        fresh: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryApplicationIdempotencyBinding {
        if fresh.conditional_definition_identity().is_none() {
            fresh.bind_conditional_definition(self.conditional_definition_identity.as_ref())
        } else {
            fresh
        }
    }

    /// Transfers the actual old record's cleanup custody without claiming refusal or completion.
    pub fn take_pending_cleanup(
        &mut self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
    ) -> Option<WorthQueryProductUnpublishedRecovery> {
        self.pending_cleanup.take()
    }

    pub fn take_prior_cleanup_failure(
        &mut self,
        _: &crate::publication_boundary::WorthQueryProgramPublicationAccess,
    ) -> Option<WorthQueryProductUnpublishedRecoveryReleaseFailure> {
        self.prior_cleanup_failure.take()
    }

    pub fn into_parts(
        self,
    ) -> (
        Result<WorthQueryAdmittedIdempotencyRead, WorthQueryApplicationIdempotencyResolutionDenial>,
        Option<WorthQueryProviderSessionFailure>,
        Option<WorthQueryProductUnpublishedRecoveryReleaseFailure>,
        Option<WorthQueryProductUnpublishedRecovery>,
    ) {
        (
            self.read,
            self.publication_failure,
            self.prior_cleanup_failure,
            self.pending_cleanup,
        )
    }
}
