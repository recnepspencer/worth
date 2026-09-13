use std::collections::BTreeMap;
use std::sync::Arc;

use worth_store_physical_backend::{
    AdmittedRecoveryFilesystemMedia, PhysicalRecoveryMediaGeneration,
};
use worth_store_physical_format::store_namespace::StableStoreIdentity;
use worth_store_physical_integrity::VerifiedCheckpointStream;
use worth_store_wal::WalSegmentArtifactIdentity;

use crate::physical_runtime::{
    ClosedPhysicalRecoveryCleanup, CompletedPhysicalRecoveryFreshReopen,
    PhysicalRecoveryCoordination,
};

use super::{StoreRecoveryCleanupEligibility, StoreRecoveryCleanupFreshnessFailure};

mod admission;
mod candidates;
mod identity;
mod materialization;
pub(in crate::physical_runtime) use admission::admit;

/// Store-owned, consuming removal plan for one exact bounded candidate set.
///
/// The plan is derived from fresh-reopen and verified checkpoint/WAL facts.
/// Callers may request admission, but cannot add a candidate after admission,
/// substitute its bytes, or mint per-artifact removal eligibility directly.
pub struct StoreRecoveryCleanupPlan {
    identity: [u8; 32],
    descriptive_plan_identity: [u8; 32],
    store: StableStoreIdentity,
    media_generation: PhysicalRecoveryMediaGeneration,
    session: [u8; 16],
    policy_identity: [u8; 32],
    reopen: CompletedPhysicalRecoveryFreshReopen,
    checkpoint: Arc<VerifiedCheckpointStream>,
    candidates: BTreeMap<WalSegmentArtifactIdentity, StoreRecoveryCleanupEligibility>,
    terminal_binding_evaluations: u64,
    media_handle_baseline: worth_store_physical_backend::RecoveryMediaHandleObservation,
}

/// Rejected Store cleanup admission with the fresh-reopen authority already
/// closed for cleanup. Recovery construction may still consume the close, but
/// the same reopen cannot be used to request a second cleanup plan.
pub struct StoreRecoveryCleanupPlanAdmissionFailure {
    closed: ClosedPhysicalRecoveryCleanup,
    failure: StoreRecoveryCleanupFreshnessFailure,
}

impl StoreRecoveryCleanupPlan {
    pub const fn identity(&self) -> [u8; 32] {
        self.identity
    }

    pub const fn descriptive_plan_identity(&self) -> [u8; 32] {
        self.descriptive_plan_identity
    }

    pub const fn terminal_binding_evaluations(&self) -> u64 {
        self.terminal_binding_evaluations
    }

    pub fn live_media_handle_delta(&self, media: &AdmittedRecoveryFilesystemMedia) -> u64 {
        media
            .handle_observation()
            .excess_over(self.media_handle_baseline)
    }

    pub fn close(self, live_media_handle_delta: u64) -> ClosedPhysicalRecoveryCleanup {
        ClosedPhysicalRecoveryCleanup::new(
            self.reopen,
            self.descriptive_plan_identity,
            Some(self.identity),
            live_media_handle_delta,
        )
    }

    pub(super) const fn policy_identity(&self) -> [u8; 32] {
        self.policy_identity
    }

    pub(super) fn bindings_match(
        &self,
        coordination: &PhysicalRecoveryCoordination,
        media: &AdmittedRecoveryFilesystemMedia,
    ) -> bool {
        self.store == media.store_identity()
            && self.media_generation == media.media_generation()
            && self.session == coordination.session_identity()
    }

    pub(super) fn checkpoint(&self) -> Arc<VerifiedCheckpointStream> {
        Arc::clone(&self.checkpoint)
    }

    pub(super) fn take(
        &mut self,
        artifact: WalSegmentArtifactIdentity,
    ) -> Option<StoreRecoveryCleanupEligibility> {
        self.candidates.remove(&artifact)
    }
}

impl StoreRecoveryCleanupPlanAdmissionFailure {
    pub const fn failure(&self) -> &StoreRecoveryCleanupFreshnessFailure {
        &self.failure
    }

    pub fn into_parts(
        self,
    ) -> (
        ClosedPhysicalRecoveryCleanup,
        StoreRecoveryCleanupFreshnessFailure,
    ) {
        (self.closed, self.failure)
    }
}
