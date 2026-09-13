use worth_store_offline_verifier::RecoveryObserverReport;

use super::{
    checkpoint_evidence::CheckpointEvidence, evidence_digest::DigestBuilder,
    identity_evidence::IdentityEvidence, manifest_evidence::ManifestEvidence,
    page_evidence::PageEvidence, residue_evidence::ResidueEvidence,
    selector_evidence::SelectorEvidence, wal_evidence::WalEvidence,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParentPhysicalEvidence {
    artifact_identity_count: u64,
    artifact_identity_digest: [u8; 32],
    generation_link_count: u64,
    generation_link_digest: [u8; 32],
    selector_count: u64,
    linked_selector_count: u64,
    unpaired_selector_link_count: u64,
    durable_selector_digest: [u8; 32],
    selector_store_identity: Option<[u8; 16]>,
    current_root_generation: Option<u64>,
    checkpoint_count: u64,
    latest_checkpoint_sequence: u64,
    checkpoint_page_count: u64,
    checkpoint_covered_lsn_start: Option<u64>,
    checkpoint_covered_lsn_end: Option<u64>,
    checkpoint_redo_lsn: Option<u64>,
    durable_checkpoint_lsn: Option<u64>,
    checkpoint_coverage_digest: [u8; 32],
    wal_segment_count: u64,
    valid_wal_prefix_bytes: u64,
    observed_wal_bytes: u64,
    wal_frame_count: u64,
    wal_first_lsn: Option<u64>,
    wal_last_lsn: Option<u64>,
    wal_digest: [u8; 32],
    page_lsn_count: u64,
    page_lsn_minimum: Option<u64>,
    page_lsn_maximum: Option<u64>,
    page_lsn_digest: [u8; 32],
    manifest_count: u64,
    manifest_member_count: u64,
    manifest_digest: [u8; 32],
    residue_artifact_count: u64,
    residue_bytes: u64,
    residue_digest: [u8; 32],
}

pub(crate) struct ParentPhysicalEvidenceParts {
    pub(super) artifact_count: u64,
    pub(super) identity: IdentityEvidence,
    pub(super) selectors: SelectorEvidence,
    pub(super) checkpoints: CheckpointEvidence,
    pub(super) wal: WalEvidence,
    pub(super) pages: PageEvidence,
    pub(super) manifests: ManifestEvidence,
    pub(super) residue: ResidueEvidence,
}

impl ParentPhysicalEvidence {
    pub(crate) fn from_parts(parts: ParentPhysicalEvidenceParts) -> Self {
        Self {
            artifact_identity_count: parts.artifact_count,
            artifact_identity_digest: parts.identity.artifact_digest,
            generation_link_count: parts.identity.generation_link_count,
            generation_link_digest: parts.identity.generation_link_digest,
            selector_count: parts.selectors.count,
            linked_selector_count: parts.selectors.linked_count,
            unpaired_selector_link_count: parts.selectors.unpaired_count,
            durable_selector_digest: parts.selectors.digest,
            selector_store_identity: parts.selectors.store_identity,
            current_root_generation: parts.selectors.current_generation,
            checkpoint_count: parts.checkpoints.count,
            latest_checkpoint_sequence: parts.checkpoints.latest_sequence,
            checkpoint_page_count: parts.checkpoints.page_count,
            checkpoint_covered_lsn_start: parts.checkpoints.covered_start,
            checkpoint_covered_lsn_end: parts.checkpoints.covered_end,
            checkpoint_redo_lsn: parts.checkpoints.redo_lsn,
            durable_checkpoint_lsn: parts.checkpoints.durable_lsn,
            checkpoint_coverage_digest: parts.checkpoints.digest,
            wal_segment_count: parts.wal.segment_count,
            valid_wal_prefix_bytes: parts.wal.valid_bytes,
            observed_wal_bytes: parts.wal.observed_bytes,
            wal_frame_count: parts.wal.frame_count,
            wal_first_lsn: parts.wal.first_lsn,
            wal_last_lsn: parts.wal.last_lsn,
            wal_digest: parts.wal.digest,
            page_lsn_count: parts.pages.count,
            page_lsn_minimum: parts.pages.minimum,
            page_lsn_maximum: parts.pages.maximum,
            page_lsn_digest: parts.pages.digest,
            manifest_count: parts.manifests.count,
            manifest_member_count: parts.manifests.member_count,
            manifest_digest: parts.manifests.digest,
            residue_artifact_count: parts.residue.count,
            residue_bytes: parts.residue.bytes,
            residue_digest: parts.residue.digest,
        }
    }

    pub(crate) const fn checkpoint_count(&self) -> u64 {
        self.checkpoint_count
    }

    pub(crate) const fn latest_checkpoint_sequence(&self) -> u64 {
        self.latest_checkpoint_sequence
    }

    pub(crate) const fn wal_segment_count(&self) -> u64 {
        self.wal_segment_count
    }

    pub(crate) const fn current_root_generation(&self) -> Option<u64> {
        self.current_root_generation
    }

    pub(crate) fn matches(&self, report: &RecoveryObserverReport) -> bool {
        self.artifact_identity_count == report.artifact_identity_count()
            && self.artifact_identity_digest == report.artifact_identity_digest()
            && self.generation_link_count == report.generation_link_count()
            && self.generation_link_digest == report.generation_link_digest()
            && self.selector_count == report.durable_selector_count()
            && self.linked_selector_count == report.linked_selector_count()
            && self.unpaired_selector_link_count == report.unpaired_selector_link_count()
            && self.durable_selector_digest == report.durable_selector_digest()
            && self.selector_store_identity == report.selector_store_identity()
            && self.current_root_generation == report.current_root_generation()
            && self.checkpoint_count == report.checkpoint_count()
            && self.checkpoint_page_count == report.checkpoint_page_count()
            && self.checkpoint_covered_lsn_start == report.checkpoint_covered_lsn_start()
            && self.checkpoint_covered_lsn_end == report.checkpoint_covered_lsn_end()
            && self.checkpoint_redo_lsn == report.checkpoint_redo_lsn()
            && self.durable_checkpoint_lsn == report.durable_checkpoint_lsn()
            && self.checkpoint_coverage_digest == report.checkpoint_coverage_digest()
            && self.wal_segment_count == report.wal_segment_count()
            && self.valid_wal_prefix_bytes == report.valid_wal_prefix_bytes()
            && self.observed_wal_bytes == report.observed_wal_bytes()
            && self.wal_frame_count == report.wal_frame_count()
            && self.wal_first_lsn == report.wal_first_lsn()
            && self.wal_last_lsn == report.wal_last_lsn()
            && self.wal_digest == report.valid_wal_prefix_digest()
            && self.page_lsn_count == report.page_lsn_count()
            && self.page_lsn_minimum == report.page_lsn_minimum()
            && self.page_lsn_maximum == report.page_lsn_maximum()
            && self.page_lsn_digest == report.page_lsn_digest()
            && self.manifest_count == report.manifest_count()
            && self.manifest_member_count == report.manifest_member_count()
            && self.manifest_digest == report.manifest_membership_digest()
            && self.residue_artifact_count == report.residue_artifact_count()
            && self.residue_bytes == report.residue_bytes()
            && self.residue_digest == report.residue_digest()
    }

    pub(crate) fn publication_digest(&self, unresolved_payload: bool) -> [u8; 32] {
        let mut digest = DigestBuilder::new(b"worth.store.recovery-observer.publication.v1");
        record_u64(&mut digest, self.generation_link_count);
        digest.record(&self.generation_link_digest);
        record_u64(&mut digest, self.selector_count);
        record_u64(&mut digest, self.linked_selector_count);
        record_u64(&mut digest, self.unpaired_selector_link_count);
        digest.record(&self.durable_selector_digest);
        record_optional_bytes(&mut digest, self.selector_store_identity.as_ref());
        record_optional_u64(&mut digest, self.current_root_generation);
        record_u64(&mut digest, self.checkpoint_count);
        record_u64(&mut digest, self.latest_checkpoint_sequence);
        record_u64(&mut digest, self.checkpoint_page_count);
        record_optional_u64(&mut digest, self.checkpoint_covered_lsn_start);
        record_optional_u64(&mut digest, self.checkpoint_covered_lsn_end);
        record_optional_u64(&mut digest, self.checkpoint_redo_lsn);
        record_optional_u64(&mut digest, self.durable_checkpoint_lsn);
        digest.record(&self.checkpoint_coverage_digest);
        record_u64(&mut digest, self.wal_segment_count);
        record_u64(&mut digest, self.valid_wal_prefix_bytes);
        record_u64(&mut digest, self.observed_wal_bytes);
        record_u64(&mut digest, self.wal_frame_count);
        record_optional_u64(&mut digest, self.wal_first_lsn);
        record_optional_u64(&mut digest, self.wal_last_lsn);
        digest.record(&self.wal_digest);
        record_u64(&mut digest, self.page_lsn_count);
        record_optional_u64(&mut digest, self.page_lsn_minimum);
        record_optional_u64(&mut digest, self.page_lsn_maximum);
        digest.record(&self.page_lsn_digest);
        record_u64(&mut digest, self.manifest_count);
        record_u64(&mut digest, self.manifest_member_count);
        digest.record(&self.manifest_digest);
        digest.record(&[u8::from(unresolved_payload)]);
        digest.finish().digest()
    }
}

fn record_u64(digest: &mut DigestBuilder, value: u64) {
    digest.record(&value.to_le_bytes());
}

fn record_optional_u64(digest: &mut DigestBuilder, value: Option<u64>) {
    digest.record(&[u8::from(value.is_some())]);
    if let Some(value) = value {
        record_u64(digest, value);
    }
}

fn record_optional_bytes(digest: &mut DigestBuilder, value: Option<&[u8; 16]>) {
    digest.record(&[u8::from(value.is_some())]);
    if let Some(value) = value {
        digest.record(value);
    }
}
