use std::path::Path;

use super::artifact_walk;
use super::conclusion;
use super::observer_evidence_summary::RecoveryObserverEvidence;
use super::report_protocol::RecoveryObserverDecodeDenial;
use super::report_wire;
use super::{RecoveryObserverLimits, RecoveryObserverObservationFailure};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryObserverReport {
    pub(super) artifact_count: u64,
    pub(super) bytes_read: u64,
    pub(super) artifact_set_digest: [u8; 32],
    pub(super) evidence: RecoveryObserverEvidence,
}

pub fn observe_recovery_artifacts(
    store_root: &Path,
    limits: RecoveryObserverLimits,
) -> Result<RecoveryObserverReport, RecoveryObserverObservationFailure> {
    let walk = artifact_walk::walk(store_root, limits)?;
    let counters = walk.counters();
    let conclusion = conclusion::conclude(walk.artifacts()).map_err(|denial| {
        RecoveryObserverObservationFailure::at_path(
            super::RecoveryObserverObservationDenial::WalTopology(denial),
            counters,
            store_root,
        )
    })?;
    Ok(RecoveryObserverReport {
        artifact_count: counters.artifacts_observed(),
        bytes_read: counters.bytes_read(),
        artifact_set_digest: conclusion.artifact_set_digest(),
        evidence: conclusion.evidence(),
    })
}

impl RecoveryObserverReport {
    pub const fn artifact_count(self) -> u64 {
        self.artifact_count
    }

    pub const fn bytes_read(self) -> u64 {
        self.bytes_read
    }

    pub const fn artifact_set_digest(self) -> [u8; 32] {
        self.artifact_set_digest
    }

    pub const fn artifact_identity_count(self) -> u64 {
        self.evidence.artifact_identities().observations()
    }

    pub const fn artifact_identity_digest(self) -> [u8; 32] {
        self.evidence.artifact_identities().digest()
    }

    pub const fn generation_link_count(self) -> u64 {
        self.evidence.generation_links().observations()
    }

    pub const fn generation_link_digest(self) -> [u8; 32] {
        self.evidence.generation_links().digest()
    }

    pub const fn durable_selector_count(self) -> u64 {
        self.evidence.durable_selectors().selector_count()
    }

    pub const fn linked_selector_count(self) -> u64 {
        self.evidence.durable_selectors().linked_selector_count()
    }

    pub const fn unpaired_selector_link_count(self) -> u64 {
        self.evidence.durable_selectors().unpaired_link_count()
    }

    pub const fn selector_store_identity(self) -> Option<[u8; 16]> {
        self.evidence.durable_selectors().store_identity()
    }

    pub const fn current_root_generation(self) -> Option<u64> {
        self.evidence.durable_selectors().current_root_generation()
    }

    pub const fn durable_selector_digest(self) -> [u8; 32] {
        self.evidence.durable_selectors().digest()
    }

    pub const fn checkpoint_count(self) -> u64 {
        self.evidence.checkpoint_coverage().checkpoint_count()
    }

    pub const fn checkpoint_page_count(self) -> u64 {
        self.evidence.checkpoint_coverage().page_count()
    }

    pub const fn checkpoint_covered_lsn_start(self) -> Option<u64> {
        self.evidence.checkpoint_coverage().covered_lsn_start()
    }

    pub const fn checkpoint_covered_lsn_end(self) -> Option<u64> {
        self.evidence.checkpoint_coverage().covered_lsn_end()
    }

    pub const fn checkpoint_redo_lsn(self) -> Option<u64> {
        self.evidence.checkpoint_coverage().redo_lsn()
    }

    pub const fn durable_checkpoint_lsn(self) -> Option<u64> {
        self.evidence.checkpoint_coverage().durable_checkpoint_lsn()
    }

    pub const fn checkpoint_coverage_digest(self) -> [u8; 32] {
        self.evidence.checkpoint_coverage().digest()
    }

    pub const fn wal_segment_count(self) -> u64 {
        self.evidence.valid_wal_prefix().segment_count()
    }

    pub const fn valid_wal_prefix_bytes(self) -> u64 {
        self.evidence.valid_wal_prefix().valid_prefix_bytes()
    }

    pub const fn observed_wal_bytes(self) -> u64 {
        self.evidence.valid_wal_prefix().observed_bytes()
    }

    pub const fn wal_frame_count(self) -> u64 {
        self.evidence.valid_wal_prefix().frame_count()
    }

    pub const fn wal_first_lsn(self) -> Option<u64> {
        self.evidence.valid_wal_prefix().first_lsn()
    }

    pub const fn wal_last_lsn(self) -> Option<u64> {
        self.evidence.valid_wal_prefix().last_lsn()
    }

    pub const fn valid_wal_prefix_digest(self) -> [u8; 32] {
        self.evidence.valid_wal_prefix().digest()
    }

    pub const fn page_lsn_count(self) -> u64 {
        self.evidence.page_lsns().observation_count()
    }

    pub const fn page_lsn_minimum(self) -> Option<u64> {
        self.evidence.page_lsns().minimum()
    }

    pub const fn page_lsn_maximum(self) -> Option<u64> {
        self.evidence.page_lsns().maximum()
    }

    pub const fn page_lsn_digest(self) -> [u8; 32] {
        self.evidence.page_lsns().digest()
    }

    pub const fn manifest_count(self) -> u64 {
        self.evidence.manifest_membership().manifest_count()
    }

    pub const fn manifest_member_count(self) -> u64 {
        self.evidence.manifest_membership().member_count()
    }

    pub const fn manifest_membership_digest(self) -> [u8; 32] {
        self.evidence.manifest_membership().digest()
    }

    pub const fn residue_artifact_count(self) -> u64 {
        self.evidence.residue().artifact_count()
    }

    pub const fn residue_bytes(self) -> u64 {
        self.evidence.residue().bytes()
    }

    pub const fn residue_digest(self) -> [u8; 32] {
        self.evidence.residue().digest()
    }

    pub fn encode(self) -> Vec<u8> {
        report_wire::encode(self)
    }

    pub fn decode(encoded: &[u8]) -> Result<Self, RecoveryObserverDecodeDenial> {
        report_wire::decode(encoded)
    }
}
