use std::io::Read;

use super::checkpoint_backup_verification::{
    verify_bounded_checkpoint_backup_artifact_from_reader,
    BoundedCheckpointBackupVerificationRequest,
};
use crate::{
    verify_bounded_extent_artifact_from_reader, verify_bounded_page_artifact_from_reader,
    verify_bounded_root_manifest_artifact_from_reader,
};
use worth_store_blob_chunks::{
    verify_bounded_blob_backup_artifact_from_reader, BoundedBlobBackupVerificationRequest,
};
use worth_store_layout_indexes::{
    verify_bounded_layout_index_artifact_from_reader, BoundedLayoutIndexVerificationRequest,
    LayoutIndexBackupFormat,
};
use worth_store_physical_format::{
    BackupBundleArtifactCoverage, BackupBundleArtifactFormat, BackupBundleArtifactManifestRow,
    PhysicalGenerationAuthority, RootPublicationCell,
};
use worth_store_wal::artifact_store::{
    verify_bounded_wal_segment_from_reader, BoundedWalSegmentVerificationRequest,
};

use super::owner_denial_classification::{
    classify_blob_denial, classify_checkpoint_denial, classify_index_denial,
    classify_physical_denial, classify_wal_denial,
};
use super::BackupArtifactSemanticDefectKind;
use crate::truth_composition::RecoveryCandidateObservation;

#[derive(Debug, Clone, Copy)]
pub(super) struct OwnerObservation {
    pub(super) bytes_read: u64,
    pub(super) decoder_allocation_bytes: u64,
    pub(super) peak_buffer_bytes: u64,
}

pub(super) struct VerifiedOwnerArtifact {
    observation: OwnerObservation,
    recovery_candidate: Option<RecoveryCandidateObservation>,
    root_publication: Option<RootPublicationCell>,
}

impl VerifiedOwnerArtifact {
    fn non_candidate(observation: OwnerObservation) -> Self {
        Self {
            observation,
            recovery_candidate: None,
            root_publication: None,
        }
    }

    pub(super) const fn observation(&self) -> OwnerObservation {
        self.observation
    }

    pub(super) fn into_recovery_candidate(self) -> Option<RecoveryCandidateObservation> {
        self.recovery_candidate
    }

    pub(super) const fn root_publication(&self) -> Option<RootPublicationCell> {
        self.root_publication
    }
}

pub(super) fn verify_owner_artifact(
    reader: &mut impl Read,
    actual_bytes: u64,
    expected_root: Option<RootPublicationCell>,
    row: &BackupBundleArtifactManifestRow,
    max_buffer_bytes: usize,
) -> Result<VerifiedOwnerArtifact, BackupArtifactSemanticDefectKind> {
    let owner = row
        .reclaim_owner()
        .generation_owner()
        .ok_or(BackupArtifactSemanticDefectKind::OwnerReferenceInvalid)?;
    let generations = PhysicalGenerationAuthority::for_canonical_physical_format();
    match row.format() {
        BackupBundleArtifactFormat::PhysicalRootManifestV1
        | BackupBundleArtifactFormat::PhysicalSecondaryRootManifestV1 => {
            let cell = generations
                .root_publication_cell(
                    owner
                        .root_reference()
                        .ok_or(BackupArtifactSemanticDefectKind::OwnerReferenceInvalid)?,
                )
                .with_root_publication_generation(owner.generation());
            verify_bounded_root_manifest_artifact_from_reader(
                reader,
                actual_bytes,
                cell,
                row.bytes(),
                row.content_digest(),
                max_buffer_bytes,
            )
            .map(|verified| {
                let observed = verified.observation();
                VerifiedOwnerArtifact {
                    observation: OwnerObservation {
                        bytes_read: observed.bytes_read(),
                        decoder_allocation_bytes: observed.decoder_allocation_bytes(),
                        peak_buffer_bytes: observed.peak_buffer_bytes(),
                    },
                    recovery_candidate: Some(
                        RecoveryCandidateObservation::from_verified_root_manifest(verified),
                    ),
                    root_publication: Some(cell),
                }
            })
            .map_err(classify_physical_denial)
        }
        BackupBundleArtifactFormat::PhysicalDataPageV1 => {
            let cell = generations
                .page_cell(
                    owner
                        .segment_id()
                        .ok_or(BackupArtifactSemanticDefectKind::OwnerReferenceInvalid)?,
                    owner
                        .page_id()
                        .ok_or(BackupArtifactSemanticDefectKind::OwnerReferenceInvalid)?,
                )
                .with_page_generation(owner.generation());
            verify_bounded_page_artifact_from_reader(
                reader,
                actual_bytes,
                cell,
                row.bytes(),
                row.content_digest(),
                max_buffer_bytes,
            )
            .map(|observed| {
                VerifiedOwnerArtifact::non_candidate(OwnerObservation {
                    bytes_read: observed.bytes_read(),
                    decoder_allocation_bytes: observed.decoder_allocation_bytes(),
                    peak_buffer_bytes: observed.peak_buffer_bytes(),
                })
            })
            .map_err(classify_physical_denial)
        }
        BackupBundleArtifactFormat::PhysicalExtentRecordV1 => {
            let cell = generations
                .extent_cell(
                    owner
                        .segment_id()
                        .ok_or(BackupArtifactSemanticDefectKind::OwnerReferenceInvalid)?,
                    owner
                        .extent_id()
                        .ok_or(BackupArtifactSemanticDefectKind::OwnerReferenceInvalid)?,
                )
                .with_extent_generation(owner.generation());
            verify_bounded_extent_artifact_from_reader(
                reader,
                actual_bytes,
                cell,
                row.bytes(),
                row.content_digest(),
                max_buffer_bytes,
            )
            .map(|observed| {
                VerifiedOwnerArtifact::non_candidate(OwnerObservation {
                    bytes_read: observed.bytes_read(),
                    decoder_allocation_bytes: observed.decoder_allocation_bytes(),
                    peak_buffer_bytes: observed.peak_buffer_bytes(),
                })
            })
            .map_err(classify_physical_denial)
        }
        BackupBundleArtifactFormat::RecoveryCheckpointManifestV1 => {
            let BackupBundleArtifactCoverage::CheckpointManifest {
                checkpoint_identity,
                manifest_generation,
                durable_checkpoint_lsn,
                authority_fingerprint,
                frontier_digest,
            } = row.coverage()
            else {
                return Err(BackupArtifactSemanticDefectKind::CoverageMismatch);
            };
            // This read-only owner verifier rechecks the persisted row-to-bytes
            // commitment. Live authority equality is admitted by the physical
            // cut owner before materialization; this layer must not mint or
            // infer current Store authority from a bundle row.
            verify_bounded_checkpoint_backup_artifact_from_reader(
                reader,
                actual_bytes,
                BoundedCheckpointBackupVerificationRequest {
                    checkpoint_identity,
                    manifest_generation: *manifest_generation,
                    durable_checkpoint_lsn: *durable_checkpoint_lsn,
                    expected_authority_fingerprint: *authority_fingerprint,
                    expected_frontier_digest: *frontier_digest,
                    expected_root: expected_root
                        .ok_or(BackupArtifactSemanticDefectKind::OwnerBindingMismatch)?,
                    expected_bytes: row.bytes(),
                    expected_digest: row.content_digest(),
                    max_buffer_bytes,
                },
            )
            .map(|observed| VerifiedOwnerArtifact {
                observation: OwnerObservation {
                    bytes_read: observed.bytes_read(),
                    decoder_allocation_bytes: observed.decoder_allocation_bytes(),
                    peak_buffer_bytes: observed.peak_buffer_bytes(),
                },
                recovery_candidate: Some(RecoveryCandidateObservation::from_verified_checkpoint(
                    observed,
                )),
                root_publication: None,
            })
            .map_err(classify_checkpoint_denial)
        }
        BackupBundleArtifactFormat::WalSegmentV1 => {
            let BackupBundleArtifactCoverage::WalSegment {
                start_lsn,
                end_exclusive_lsn,
            } = row.coverage()
            else {
                return Err(BackupArtifactSemanticDefectKind::CoverageMismatch);
            };
            let request = BoundedWalSegmentVerificationRequest::new(
                owner
                    .segment_id()
                    .ok_or(BackupArtifactSemanticDefectKind::OwnerReferenceInvalid)?
                    .get(),
                owner.generation().get(),
                *start_lsn,
                *end_exclusive_lsn,
                row.bytes(),
                row.content_digest(),
                max_buffer_bytes,
            )
            .ok_or(BackupArtifactSemanticDefectKind::BufferBudgetTooSmall)?;
            verify_bounded_wal_segment_from_reader(reader, actual_bytes, request)
                .map(|observed| VerifiedOwnerArtifact {
                    observation: OwnerObservation {
                        bytes_read: observed.bytes_read(),
                        decoder_allocation_bytes: observed.decoder_allocation_bytes(),
                        peak_buffer_bytes: observed.peak_buffer_bytes(),
                    },
                    recovery_candidate: Some(
                        RecoveryCandidateObservation::from_verified_wal_segment(observed),
                    ),
                    root_publication: None,
                })
                .map_err(classify_wal_denial)
        }
        BackupBundleArtifactFormat::LayoutBTreeLeafV1
        | BackupBundleArtifactFormat::LayoutBTreeRootV1 => {
            let format = match row.format() {
                BackupBundleArtifactFormat::LayoutBTreeLeafV1 => {
                    LayoutIndexBackupFormat::BaselineBTreeLeafV1
                }
                BackupBundleArtifactFormat::LayoutBTreeRootV1 => {
                    LayoutIndexBackupFormat::BaselineBTreeRootV1
                }
                _ => unreachable!("matched layout formats"),
            };
            verify_bounded_layout_index_artifact_from_reader(
                reader,
                actual_bytes,
                BoundedLayoutIndexVerificationRequest::new(
                    format,
                    row.identity(),
                    row.bytes(),
                    row.content_digest(),
                    max_buffer_bytes,
                ),
            )
            .map(|observed| {
                VerifiedOwnerArtifact::non_candidate(OwnerObservation {
                    bytes_read: observed.bytes_read(),
                    decoder_allocation_bytes: 0,
                    peak_buffer_bytes: observed.peak_buffer_bytes(),
                })
            })
            .map_err(classify_index_denial)
        }
        BackupBundleArtifactFormat::BlobChunkV1 => verify_bounded_blob_backup_artifact_from_reader(
            reader,
            actual_bytes,
            BoundedBlobBackupVerificationRequest {
                expected_identity: row.identity(),
                expected_bytes: row.bytes(),
                expected_digest: row.content_digest(),
                max_buffer_bytes,
            },
        )
        .map(|observed| {
            VerifiedOwnerArtifact::non_candidate(OwnerObservation {
                bytes_read: observed.bytes_read(),
                decoder_allocation_bytes: observed.decoder_allocation_bytes(),
                peak_buffer_bytes: observed.peak_buffer_bytes(),
            })
        })
        .map_err(classify_blob_denial),
    }
}
