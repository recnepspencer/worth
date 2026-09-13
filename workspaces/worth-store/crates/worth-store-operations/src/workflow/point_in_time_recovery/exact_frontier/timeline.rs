use worth_store_authority::StoreCurrentAuthorityIdentity;

use super::super::candidate_selection::PitrCandidatePosture;
use super::frontier::{frontier_identity, ExactRecoveryFrontier};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryTimelineAdmission {
    pub observed_time: i64,
    pub uncertainty_before: u64,
    pub uncertainty_after: u64,
    pub checkpoint_durability: u64,
    pub wal_structural: u64,
    pub local_durable_commit: u64,
    pub client_acknowledged: u64,
    pub replication_acknowledged: u64,
    pub authority_identity: StoreCurrentAuthorityIdentity,
    pub source_lineage: [u8; 32],
    pub source_identity: [u8; 32],
    pub posture: PitrCandidatePosture,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryTimelineObservation {
    pub(crate) observed_time: i64,
    pub(crate) uncertainty_before: u64,
    pub(crate) uncertainty_after: u64,
    pub(crate) frontier: ExactRecoveryFrontier,
    pub(crate) source_identity: [u8; 32],
    pub(crate) posture: PitrCandidatePosture,
}

#[derive(Debug, Clone, Copy)]
pub struct RecoveryTimelineOwner;

impl RecoveryTimelineOwner {
    pub fn admit_observation(
        admission: RecoveryTimelineAdmission,
    ) -> Option<RecoveryTimelineObservation> {
        if admission.source_lineage == [0; 32]
            || admission.source_identity == [0; 32]
            || admission.checkpoint_durability > admission.wal_structural
            || admission.wal_structural > admission.local_durable_commit
            || admission.client_acknowledged > admission.local_durable_commit
            || admission.replication_acknowledged > admission.local_durable_commit
        {
            return None;
        }
        let identity = frontier_identity(
            admission.checkpoint_durability,
            admission.wal_structural,
            admission.local_durable_commit,
            admission.client_acknowledged,
            admission.replication_acknowledged,
            admission.authority_identity,
            admission.source_lineage,
        );
        Some(RecoveryTimelineObservation {
            observed_time: admission.observed_time,
            uncertainty_before: admission.uncertainty_before,
            uncertainty_after: admission.uncertainty_after,
            frontier: ExactRecoveryFrontier::from_parts(
                admission.checkpoint_durability,
                admission.wal_structural,
                admission.local_durable_commit,
                admission.client_acknowledged,
                admission.replication_acknowledged,
                admission.authority_identity,
                admission.source_lineage,
                identity,
            ),
            source_identity: admission.source_identity,
            posture: admission.posture,
        })
    }
}
