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
    pub(super) conditional_definition_identity: Option<[u8; 32]>,
}

impl WorthQueryManagedApplicationRecoveryPerformed {
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
