use super::{CheckpointReadInterlockCounters, ReadDuringCheckpointVerdict};
use crate::{CheckpointPublicationIdentity, ManifestEpoch, RootEpoch};
use worth_store_physical_format::CheckpointWalSourceRange;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckpointInterlockFoundationalEvidence {
    origin: CheckpointInterlockEvidenceOrigin,
    counters: CheckpointReadInterlockCounters,
    materialized_after_store_decision: bool,
    no_mixed_root: bool,
    old_reader_retained_old_root: bool,
    post_publication_reader_observed_new_epoch: bool,
    checkpoint_wal_bound_to_cutover: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckpointInterlockEvidenceOrigin {
    checkpoint_identity: CheckpointPublicationIdentity,
    old_root_epoch: RootEpoch,
    published_root_epoch: RootEpoch,
    old_manifest_epoch: ManifestEpoch,
    published_manifest_epoch: ManifestEpoch,
    checkpoint_wal_range: CheckpointWalSourceRange,
}

impl CheckpointInterlockFoundationalEvidence {
    pub fn after_executed_interlock(verdict: &ReadDuringCheckpointVerdict) -> Self {
        let proof = verdict.proof();
        let counters = proof.plan().transition().counters();
        Self {
            origin: CheckpointInterlockEvidenceOrigin::from_verdict(verdict),
            counters,
            materialized_after_store_decision: true,
            no_mixed_root: proof.no_mixed_root(),
            old_reader_retained_old_root: verdict.old_reader_retained_old_root(),
            post_publication_reader_observed_new_epoch: verdict
                .post_publication_reader_observed_new_epoch(),
            checkpoint_wal_bound_to_cutover: proof
                .plan()
                .transition()
                .checkpoint_wal_bound_to_cutover(),
        }
    }

    pub fn copied_report_attempt_from_store_evidence(evidence: &Self) -> Self {
        Self {
            origin: evidence.origin.clone(),
            counters: evidence.counters,
            materialized_after_store_decision: false,
            no_mixed_root: evidence.no_mixed_root,
            old_reader_retained_old_root: evidence.old_reader_retained_old_root,
            post_publication_reader_observed_new_epoch: evidence
                .post_publication_reader_observed_new_epoch,
            checkpoint_wal_bound_to_cutover: evidence.checkpoint_wal_bound_to_cutover,
        }
    }

    pub const fn origin(&self) -> &CheckpointInterlockEvidenceOrigin {
        &self.origin
    }

    pub const fn counters(&self) -> CheckpointReadInterlockCounters {
        self.counters
    }

    pub const fn materialized_after_store_decision(&self) -> bool {
        self.materialized_after_store_decision
    }

    pub const fn no_mixed_root(&self) -> bool {
        self.no_mixed_root
    }

    pub const fn old_reader_retained_old_root(&self) -> bool {
        self.old_reader_retained_old_root
    }

    pub const fn post_publication_reader_observed_new_epoch(&self) -> bool {
        self.post_publication_reader_observed_new_epoch
    }

    pub const fn checkpoint_wal_bound_to_cutover(&self) -> bool {
        self.checkpoint_wal_bound_to_cutover
    }
}

impl CheckpointInterlockEvidenceOrigin {
    fn from_verdict(verdict: &ReadDuringCheckpointVerdict) -> Self {
        let transition = verdict.proof().plan().transition();
        let old_root = transition.old_current_root();
        let published_root = transition.published_current_root();
        Self {
            checkpoint_identity: transition.checkpoint_root().checkpoint_identity().clone(),
            old_root_epoch: old_root.epoch(),
            published_root_epoch: published_root.epoch(),
            old_manifest_epoch: old_root.manifest_epoch(),
            published_manifest_epoch: published_root.manifest_epoch(),
            checkpoint_wal_range: transition.checkpoint_wal_range(),
        }
    }

    pub const fn checkpoint_identity(&self) -> &CheckpointPublicationIdentity {
        &self.checkpoint_identity
    }

    pub const fn old_root_epoch(&self) -> RootEpoch {
        self.old_root_epoch
    }

    pub const fn published_root_epoch(&self) -> RootEpoch {
        self.published_root_epoch
    }

    pub const fn old_manifest_epoch(&self) -> ManifestEpoch {
        self.old_manifest_epoch
    }

    pub const fn published_manifest_epoch(&self) -> ManifestEpoch {
        self.published_manifest_epoch
    }

    pub const fn checkpoint_wal_range(&self) -> CheckpointWalSourceRange {
        self.checkpoint_wal_range
    }
}
