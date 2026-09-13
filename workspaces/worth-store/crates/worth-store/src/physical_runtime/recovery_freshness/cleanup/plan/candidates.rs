use std::collections::BTreeMap;

use worth_store_physical_backend::AdmittedRecoveryFilesystemMedia;
use worth_store_physical_integrity::VerifiedCheckpointStream;
use worth_store_wal::{WalLsnRange, WalSegmentArtifactIdentity};

use crate::physical_runtime::{CompletedPhysicalRecoveryFreshReopen, PhysicalRecoveryCoordination};

use super::super::{StoreRecoveryCleanupFreshnessDenial, StoreRecoveryCleanupFreshnessFailure};
use super::admission::{common_basis, invalid, CommonBasis};

pub(super) struct CandidateAdmissionContext<'a> {
    pub(super) coordination: &'a PhysicalRecoveryCoordination,
    pub(super) media: &'a AdmittedRecoveryFilesystemMedia,
    pub(super) reopened: &'a CompletedPhysicalRecoveryFreshReopen,
    pub(super) checkpoint: &'a VerifiedCheckpointStream,
    pub(super) descriptive_plan_identity: [u8; 32],
}

pub(super) struct PendingCandidate {
    pub(super) wal: crate::physical_runtime::IntegrityAdmittedRecoveryWalSegment,
    pub(super) artifact: WalSegmentArtifactIdentity,
    pub(super) lsn_range: WalLsnRange,
    pub(super) byte_count: u64,
    pub(super) artifact_digest: [u8; 32],
}

pub(super) struct AdmittedCandidates {
    pub(super) common: CommonBasis,
    pub(super) pending: BTreeMap<WalSegmentArtifactIdentity, PendingCandidate>,
    pub(super) terminal_binding_evaluations: u64,
}

pub(super) fn admit(
    context: CandidateAdmissionContext<'_>,
    wal: impl IntoIterator<Item = crate::physical_runtime::IntegrityAdmittedRecoveryWalSegment>,
) -> Result<AdmittedCandidates, StoreRecoveryCleanupFreshnessFailure> {
    if context.descriptive_plan_identity == [0; 32] {
        return Err(invalid());
    }
    let common = common_basis(
        context.coordination,
        context.media,
        context.reopened,
        context.checkpoint,
    )?;
    let capacity = context.coordination.cleanup_capacity();
    let mut pending = BTreeMap::new();
    let mut admitted_bytes = 0_u64;
    for wal in wal {
        if pending.len() == capacity.cleanup_candidates() {
            return Err(invalid());
        }
        let inspection = wal.inspection();
        let next_bytes = admitted_bytes
            .checked_add(inspection.byte_count())
            .filter(|bytes| *bytes <= capacity.cleanup_bytes())
            .ok_or_else(invalid)?;
        if inspection.byte_count() == 0
            || inspection.lsn_range().end_exclusive() > common.retained_boundary
        {
            return Err(invalid());
        }
        let candidate = PendingCandidate {
            artifact: inspection.identity(),
            lsn_range: inspection.lsn_range(),
            byte_count: inspection.byte_count(),
            artifact_digest: inspection.artifact_digest(),
            wal,
        };
        if pending.insert(candidate.artifact, candidate).is_some() {
            return Err(invalid());
        }
        admitted_bytes = next_bytes;
    }
    let terminal_binding_evaluations = admit_terminal_bindings(&context, &pending, admitted_bytes)?;
    Ok(AdmittedCandidates {
        common,
        pending,
        terminal_binding_evaluations,
    })
}

fn admit_terminal_bindings(
    context: &CandidateAdmissionContext<'_>,
    pending: &BTreeMap<WalSegmentArtifactIdentity, PendingCandidate>,
    admitted_bytes: u64,
) -> Result<u64, StoreRecoveryCleanupFreshnessFailure> {
    let wal_frames = pending
        .values()
        .flat_map(|candidate| candidate.wal.frames());
    let frame_count = pending.values().try_fold(0_u64, |count, candidate| {
        count.checked_add(candidate.wal.frames().len() as u64)
    });
    let maximum_operations = frame_count
        .and_then(|count| {
            context
                .checkpoint
                .footer()
                .binding_record_count()
                .checked_add(count)
        })
        .ok_or_else(invalid)?;
    let before = context.coordination.freshness().binding_samples();
    let sampled = super::super::super::binding::sample_binding(
        context.coordination.freshness(),
        context.coordination.checkpoint_binding_basis(),
        context.media,
        context.checkpoint,
        wal_frames,
        maximum_operations,
        admitted_bytes,
    );
    let terminal_binding_evaluations = context
        .coordination
        .freshness()
        .binding_samples()
        .saturating_sub(before);
    let binding = sampled.map_err(|binding| StoreRecoveryCleanupFreshnessFailure {
        denial: StoreRecoveryCleanupFreshnessDenial::InvalidCleanupEligibility,
        sample: None,
        read: None,
        binding: Some(binding),
        terminal_binding_evaluations,
    })?;
    super::super::wal_members_are_terminal(&binding)
        .then_some(terminal_binding_evaluations)
        .ok_or(StoreRecoveryCleanupFreshnessFailure {
            denial: StoreRecoveryCleanupFreshnessDenial::InvalidCleanupEligibility,
            sample: None,
            read: None,
            binding: None,
            terminal_binding_evaluations,
        })
}
