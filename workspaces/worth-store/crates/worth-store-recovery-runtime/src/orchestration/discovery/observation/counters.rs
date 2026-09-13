use worth_store_recovery_physics::{
    PhysicalRecoveryResidue, PhysicalRecoveryResidueKind, PhysicalRootSlotObservation,
};

use crate::progression::PhysicalRecoveryDiscoveryCounters;

use super::super::{CheckpointDiscovery, WalDiscovery};

pub(super) fn record_root_counters(
    counters: &mut PhysicalRecoveryDiscoveryCounters,
    current: &PhysicalRootSlotObservation,
    previous: &PhysicalRootSlotObservation,
) {
    counters.root_candidates = admitted_root_count(current) + admitted_root_count(previous);
    counters.current_root_admitted = root_posture(current, RootPosture::Admitted);
    counters.current_root_rejected = root_posture(current, RootPosture::Rejected);
    counters.current_root_absent = root_posture(current, RootPosture::Absent);
    counters.previous_root_admitted = root_posture(previous, RootPosture::Admitted);
    counters.previous_root_rejected = root_posture(previous, RootPosture::Rejected);
    counters.previous_root_absent = root_posture(previous, RootPosture::Absent);
}

pub(super) fn record_checkpoint_counters(
    counters: &mut PhysicalRecoveryDiscoveryCounters,
    checkpoint: &CheckpointDiscovery,
) {
    counters.checkpoint_candidates =
        u64::from(matches!(checkpoint, CheckpointDiscovery::Admitted { .. }));
    counters.checkpoints_admitted = counters.checkpoint_candidates;
    counters.checkpoints_rejected =
        u64::from(matches!(checkpoint, CheckpointDiscovery::Rejected(_)));
    counters.checkpoints_absent = u64::from(matches!(checkpoint, CheckpointDiscovery::Absent));
}

pub(super) fn record_wal_counters(
    counters: &mut PhysicalRecoveryDiscoveryCounters,
    wal: &WalDiscovery,
    residue: &[PhysicalRecoveryResidue],
    wal_entries: u64,
) {
    counters.wal_entries = wal_entries;
    counters.wal_integrity_attempts = wal.integrity_ingress.attempted;
    counters.wal_integrity_admissions = wal.integrity_ingress.admitted;
    counters.wal_integrity_rejections = wal
        .integrity_ingress
        .attempted
        .saturating_sub(wal.integrity_ingress.admitted);
    counters.wal_owner_projections = wal.integrity_ingress.owner_projection_entries;
    counters.wal_owner_decoder_entries = wal.integrity_ingress.owner_decoder_entries;
    counters.wal_segments = wal.scanned_segments;
    counters.wal_segments_scanned = wal.scanned_segments;
    counters.valid_wal_segments = wal.valid_segments;
    counters.wal_frames = wal.scanned_frames;
    counters.wal_bytes = wal.observed_bytes;
    counters.valid_wal_frames = wal.valid_frames;
    counters.valid_wal_bytes = wal.valid_bytes;
    counters.torn_suffix_frames = wal.torn_suffix_frames;
    counters.torn_suffix_bytes = wal.torn_suffix_bytes;
    counters.wal_corruption_denials = wal.corruption_denials;
    counters.residue = residue.len() as u64;
    counters.noncanonical_wal_residue = residue_count(
        residue,
        PhysicalRecoveryResidueKind::NonCanonicalWalArtifact,
    );
    counters.nonregular_wal_residue =
        residue_count(residue, PhysicalRecoveryResidueKind::NonRegularWalEntry);
    counters.trailing_empty_wal_residue = residue_count(
        residue,
        PhysicalRecoveryResidueKind::TrailingEmptyWalSegment,
    );
    counters.interrupted_wal_start_residue = residue_count(
        residue,
        PhysicalRecoveryResidueKind::InterruptedWalSegmentStart,
    );
    counters.unreferenced_compaction_residue = residue_count(
        residue,
        PhysicalRecoveryResidueKind::UnreferencedCompactionProduct,
    );
}

fn admitted_root_count(observation: &PhysicalRootSlotObservation) -> u64 {
    u64::from(matches!(
        observation,
        PhysicalRootSlotObservation::Candidate(_)
    ))
}

enum RootPosture {
    Admitted,
    Rejected,
    Absent,
}

fn root_posture(observation: &PhysicalRootSlotObservation, posture: RootPosture) -> u64 {
    u64::from(matches!(
        (observation, posture),
        (
            PhysicalRootSlotObservation::Candidate(_),
            RootPosture::Admitted
        ) | (
            PhysicalRootSlotObservation::SelectorRejected(_)
                | PhysicalRootSlotObservation::RootRejected { .. },
            RootPosture::Rejected
        ) | (PhysicalRootSlotObservation::Absent, RootPosture::Absent)
    ))
}

fn residue_count(residue: &[PhysicalRecoveryResidue], kind: PhysicalRecoveryResidueKind) -> u64 {
    residue.iter().filter(|item| item.kind() == kind).count() as u64
}
