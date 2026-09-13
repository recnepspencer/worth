use std::collections::BTreeMap;
use std::sync::Arc;

use worth_store_physical_backend::AdmittedRecoveryFilesystemMedia;
use worth_store_physical_integrity::VerifiedCheckpointStream;
use worth_store_wal::WalSegmentArtifactIdentity;

use crate::physical_runtime::{
    CompletedPhysicalRecoveryFreshReopen, PhysicalRecoveryCoordinationCapacity,
};

use super::super::{StoreRecoveryCleanupEligibility, StoreRecoveryCleanupRemovalBasis};
use super::candidates::{AdmittedCandidates, PendingCandidate};
use super::identity::{plan_identity, policy_identity};
use super::StoreRecoveryCleanupPlan;

pub(super) struct PlanMaterializationInput {
    pub(super) reopened: CompletedPhysicalRecoveryFreshReopen,
    pub(super) checkpoint: Arc<VerifiedCheckpointStream>,
    pub(super) descriptive_plan_identity: [u8; 32],
    pub(super) admitted: AdmittedCandidates,
    pub(super) capacity: PhysicalRecoveryCoordinationCapacity,
}

pub(super) fn materialize(
    media: &AdmittedRecoveryFilesystemMedia,
    input: PlanMaterializationInput,
) -> StoreRecoveryCleanupPlan {
    let AdmittedCandidates {
        common,
        pending,
        terminal_binding_evaluations,
    } = input.admitted;
    let policy_identity = policy_identity(&common, input.capacity);
    let identity = plan_identity(
        &common,
        input.descriptive_plan_identity,
        policy_identity,
        &pending,
    );
    let candidates = materialize_candidates(&common, pending, identity, input.reopened.format());
    StoreRecoveryCleanupPlan {
        identity,
        descriptive_plan_identity: input.descriptive_plan_identity,
        store: common.store,
        media_generation: common.media_generation,
        session: common.session,
        policy_identity,
        reopen: input.reopened,
        checkpoint: input.checkpoint,
        candidates,
        terminal_binding_evaluations,
        media_handle_baseline: media.handle_observation(),
    }
}

fn materialize_candidates(
    common: &super::admission::CommonBasis,
    pending: BTreeMap<WalSegmentArtifactIdentity, PendingCandidate>,
    identity: [u8; 32],
    format: worth_store_physical_format::PhysicalRecordFormatDeclaration,
) -> BTreeMap<WalSegmentArtifactIdentity, StoreRecoveryCleanupEligibility> {
    pending
        .into_iter()
        .map(|(artifact, pending)| {
            (
                artifact,
                StoreRecoveryCleanupEligibility {
                    wal: pending.wal,
                    removal: StoreRecoveryCleanupRemovalBasis {
                        store: common.store,
                        media_generation: common.media_generation,
                        session: common.session,
                        plan: identity,
                        published_generation: common.published_generation,
                        format,
                        sealed_publication_basis: common.sealed_publication_basis,
                        checkpoint: common.checkpoint,
                        compaction_generation: common.compaction_generation,
                        compaction_digest: common.compaction_digest,
                        retained_boundary: common.retained_boundary,
                        artifact,
                        lsn_range: pending.lsn_range,
                        byte_count: pending.byte_count,
                        root_read: common.root_read.clone(),
                    },
                },
            )
        })
        .collect()
}
