//! Exact semantic report comparison and its first typed difference.

use worth_store_offline_verifier::RecoveryObserverReport;

use super::ParentPhysicalEvidence;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ParentSemanticField {
    ArtifactIdentityCount,
    ArtifactIdentityDigest,
    GenerationLinkCount,
    GenerationLinkDigest,
    SelectorCount,
    LinkedSelectorCount,
    UnpairedSelectorLinkCount,
    DurableSelectorDigest,
    SelectorStoreIdentity,
    CurrentRootGeneration,
    CheckpointCount,
    CheckpointPageCount,
    CheckpointCoveredLsnStart,
    CheckpointCoveredLsnEnd,
    CheckpointRedoLsn,
    DurableCheckpointLsn,
    CheckpointCoverageDigest,
    WalSegmentCount,
    ValidWalPrefixBytes,
    ObservedWalBytes,
    WalFrameCount,
    WalFirstLsn,
    WalLastLsn,
    WalDigest,
    PageLsnCount,
    PageLsnMinimum,
    PageLsnMaximum,
    PageLsnDigest,
    ManifestCount,
    ManifestMemberCount,
    ManifestDigest,
    ResidueArtifactCount,
    ResidueBytes,
    ResidueDigest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ParentEvidenceValue {
    Counter(u64),
    OptionalCounter(Option<u64>),
    Digest([u8; 32]),
    StoreIdentity(Option<[u8; 16]>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ParentSemanticMismatch {
    pub(crate) field: ParentSemanticField,
    pub(crate) expected: ParentEvidenceValue,
    pub(crate) observed: ParentEvidenceValue,
}

impl ParentPhysicalEvidence {
    pub(crate) fn compare_report(
        &self,
        report: &RecoveryObserverReport,
    ) -> Result<(), ParentSemanticMismatch> {
        macro_rules! compare_field {
            ($name:ident, $field:ident, $method:ident, $kind:ident) => {
                if self.$field != report.$method() {
                    return Err(ParentSemanticMismatch {
                        field: ParentSemanticField::$name,
                        expected: ParentEvidenceValue::$kind(self.$field),
                        observed: ParentEvidenceValue::$kind(report.$method()),
                    });
                }
            };
        }
        compare_field!(
            ArtifactIdentityCount,
            artifact_identity_count,
            artifact_identity_count,
            Counter
        );
        compare_field!(
            ArtifactIdentityDigest,
            artifact_identity_digest,
            artifact_identity_digest,
            Digest
        );
        compare_field!(
            GenerationLinkCount,
            generation_link_count,
            generation_link_count,
            Counter
        );
        compare_field!(
            GenerationLinkDigest,
            generation_link_digest,
            generation_link_digest,
            Digest
        );
        compare_field!(
            SelectorCount,
            selector_count,
            durable_selector_count,
            Counter
        );
        compare_field!(
            LinkedSelectorCount,
            linked_selector_count,
            linked_selector_count,
            Counter
        );
        compare_field!(
            UnpairedSelectorLinkCount,
            unpaired_selector_link_count,
            unpaired_selector_link_count,
            Counter
        );
        compare_field!(
            DurableSelectorDigest,
            durable_selector_digest,
            durable_selector_digest,
            Digest
        );
        compare_field!(
            SelectorStoreIdentity,
            selector_store_identity,
            selector_store_identity,
            StoreIdentity
        );
        compare_field!(
            CurrentRootGeneration,
            current_root_generation,
            current_root_generation,
            OptionalCounter
        );
        compare_field!(CheckpointCount, checkpoint_count, checkpoint_count, Counter);
        compare_field!(
            CheckpointPageCount,
            checkpoint_page_count,
            checkpoint_page_count,
            Counter
        );
        compare_field!(
            CheckpointCoveredLsnStart,
            checkpoint_covered_lsn_start,
            checkpoint_covered_lsn_start,
            OptionalCounter
        );
        compare_field!(
            CheckpointCoveredLsnEnd,
            checkpoint_covered_lsn_end,
            checkpoint_covered_lsn_end,
            OptionalCounter
        );
        compare_field!(
            CheckpointRedoLsn,
            checkpoint_redo_lsn,
            checkpoint_redo_lsn,
            OptionalCounter
        );
        compare_field!(
            DurableCheckpointLsn,
            durable_checkpoint_lsn,
            durable_checkpoint_lsn,
            OptionalCounter
        );
        compare_field!(
            CheckpointCoverageDigest,
            checkpoint_coverage_digest,
            checkpoint_coverage_digest,
            Digest
        );
        compare_field!(
            WalSegmentCount,
            wal_segment_count,
            wal_segment_count,
            Counter
        );
        compare_field!(
            ValidWalPrefixBytes,
            valid_wal_prefix_bytes,
            valid_wal_prefix_bytes,
            Counter
        );
        compare_field!(
            ObservedWalBytes,
            observed_wal_bytes,
            observed_wal_bytes,
            Counter
        );
        compare_field!(WalFrameCount, wal_frame_count, wal_frame_count, Counter);
        compare_field!(WalFirstLsn, wal_first_lsn, wal_first_lsn, OptionalCounter);
        compare_field!(WalLastLsn, wal_last_lsn, wal_last_lsn, OptionalCounter);
        compare_field!(WalDigest, wal_digest, valid_wal_prefix_digest, Digest);
        compare_field!(PageLsnCount, page_lsn_count, page_lsn_count, Counter);
        compare_field!(
            PageLsnMinimum,
            page_lsn_minimum,
            page_lsn_minimum,
            OptionalCounter
        );
        compare_field!(
            PageLsnMaximum,
            page_lsn_maximum,
            page_lsn_maximum,
            OptionalCounter
        );
        compare_field!(PageLsnDigest, page_lsn_digest, page_lsn_digest, Digest);
        compare_field!(ManifestCount, manifest_count, manifest_count, Counter);
        compare_field!(
            ManifestMemberCount,
            manifest_member_count,
            manifest_member_count,
            Counter
        );
        compare_field!(
            ManifestDigest,
            manifest_digest,
            manifest_membership_digest,
            Digest
        );
        compare_field!(
            ResidueArtifactCount,
            residue_artifact_count,
            residue_artifact_count,
            Counter
        );
        compare_field!(ResidueBytes, residue_bytes, residue_bytes, Counter);
        compare_field!(ResidueDigest, residue_digest, residue_digest, Digest);
        Ok(())
    }
}
