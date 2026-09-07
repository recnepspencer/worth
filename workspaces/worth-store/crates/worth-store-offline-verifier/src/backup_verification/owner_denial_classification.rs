use crate::BoundedPhysicalArtifactDenial;
use worth_store_blob_chunks::BoundedBlobBackupDenial;
use worth_store_layout_indexes::BoundedLayoutIndexDenial;
use worth_store_physical_format::PhysicalHeaderDecodeDenialKind;
use worth_store_wal::artifact_store::BoundedWalSegmentDenial;

use super::checkpoint_backup_verification::BoundedCheckpointBackupDenial;
use super::BackupArtifactSemanticDefectKind;

pub(super) fn classify_physical_denial(
    denial: BoundedPhysicalArtifactDenial,
) -> BackupArtifactSemanticDefectKind {
    match denial {
        BoundedPhysicalArtifactDenial::Io(_) => BackupArtifactSemanticDefectKind::Io,
        BoundedPhysicalArtifactDenial::AllocationFailed => {
            BackupArtifactSemanticDefectKind::VerifierAllocationFailed
        }
        BoundedPhysicalArtifactDenial::BufferTooSmall { .. } => {
            BackupArtifactSemanticDefectKind::BufferBudgetTooSmall
        }
        BoundedPhysicalArtifactDenial::LengthMismatch { .. } => {
            BackupArtifactSemanticDefectKind::LengthMismatch
        }
        BoundedPhysicalArtifactDenial::DigestMismatch => {
            BackupArtifactSemanticDefectKind::DigestMismatch
        }
        BoundedPhysicalArtifactDenial::HeaderDecode(denial)
            if denial.kind() == PhysicalHeaderDecodeDenialKind::OwnerCoordinateMismatch =>
        {
            BackupArtifactSemanticDefectKind::OwnerBindingMismatch
        }
        BoundedPhysicalArtifactDenial::RootDecode(_)
        | BoundedPhysicalArtifactDenial::HeaderDecode(_) => {
            BackupArtifactSemanticDefectKind::MalformedOwnerEncoding
        }
        BoundedPhysicalArtifactDenial::ReferenceMismatch => {
            BackupArtifactSemanticDefectKind::OwnerBindingMismatch
        }
        BoundedPhysicalArtifactDenial::UnpublishedArtifact => {
            BackupArtifactSemanticDefectKind::UnpublishedArtifact
        }
    }
}

pub(super) fn classify_checkpoint_denial(
    denial: BoundedCheckpointBackupDenial,
) -> BackupArtifactSemanticDefectKind {
    match denial {
        BoundedCheckpointBackupDenial::Io(_) => BackupArtifactSemanticDefectKind::Io,
        BoundedCheckpointBackupDenial::AllocationFailed => {
            BackupArtifactSemanticDefectKind::VerifierAllocationFailed
        }
        BoundedCheckpointBackupDenial::BufferTooSmall => {
            BackupArtifactSemanticDefectKind::BufferBudgetTooSmall
        }
        BoundedCheckpointBackupDenial::LengthMismatch { .. } => {
            BackupArtifactSemanticDefectKind::LengthMismatch
        }
        BoundedCheckpointBackupDenial::InvalidHeader
        | BoundedCheckpointBackupDenial::InvalidPageFrontier => {
            BackupArtifactSemanticDefectKind::MalformedOwnerEncoding
        }
        BoundedCheckpointBackupDenial::BindingMismatch => {
            BackupArtifactSemanticDefectKind::OwnerBindingMismatch
        }
        BoundedCheckpointBackupDenial::InternalDigestMismatch => {
            BackupArtifactSemanticDefectKind::OwnerIntegrityMismatch
        }
        BoundedCheckpointBackupDenial::ArtifactDigestMismatch => {
            BackupArtifactSemanticDefectKind::DigestMismatch
        }
    }
}

pub(super) fn classify_wal_denial(
    denial: BoundedWalSegmentDenial,
) -> BackupArtifactSemanticDefectKind {
    match denial {
        BoundedWalSegmentDenial::Io(_) => BackupArtifactSemanticDefectKind::Io,
        BoundedWalSegmentDenial::AllocationFailed => {
            BackupArtifactSemanticDefectKind::VerifierAllocationFailed
        }
        BoundedWalSegmentDenial::CounterOverflow => {
            BackupArtifactSemanticDefectKind::MalformedOwnerEncoding
        }
        BoundedWalSegmentDenial::LengthMismatch { .. } => {
            BackupArtifactSemanticDefectKind::LengthMismatch
        }
        BoundedWalSegmentDenial::InvalidFrame => {
            BackupArtifactSemanticDefectKind::MalformedOwnerEncoding
        }
        BoundedWalSegmentDenial::FrameDigestMismatch
        | BoundedWalSegmentDenial::PayloadDigestMismatch => {
            BackupArtifactSemanticDefectKind::OwnerIntegrityMismatch
        }
        BoundedWalSegmentDenial::ArtifactDigestMismatch => {
            BackupArtifactSemanticDefectKind::DigestMismatch
        }
        BoundedWalSegmentDenial::SegmentBindingMismatch
        | BoundedWalSegmentDenial::GenerationBindingMismatch => {
            BackupArtifactSemanticDefectKind::OwnerBindingMismatch
        }
        BoundedWalSegmentDenial::CoverageMismatch | BoundedWalSegmentDenial::NonContiguousLsn => {
            BackupArtifactSemanticDefectKind::CoverageMismatch
        }
    }
}

pub(super) fn classify_index_denial(
    denial: BoundedLayoutIndexDenial,
) -> BackupArtifactSemanticDefectKind {
    match denial {
        BoundedLayoutIndexDenial::Io(_) => BackupArtifactSemanticDefectKind::Io,
        BoundedLayoutIndexDenial::BufferTooSmall { .. } => {
            BackupArtifactSemanticDefectKind::BufferBudgetTooSmall
        }
        BoundedLayoutIndexDenial::LengthMismatch { .. } => {
            BackupArtifactSemanticDefectKind::LengthMismatch
        }
        BoundedLayoutIndexDenial::DigestMismatch => {
            BackupArtifactSemanticDefectKind::DigestMismatch
        }
        BoundedLayoutIndexDenial::IdentityMismatch => {
            BackupArtifactSemanticDefectKind::OwnerBindingMismatch
        }
        BoundedLayoutIndexDenial::MalformedIndex => {
            BackupArtifactSemanticDefectKind::MalformedOwnerEncoding
        }
    }
}

pub(super) fn classify_blob_denial(
    denial: BoundedBlobBackupDenial,
) -> BackupArtifactSemanticDefectKind {
    match denial {
        BoundedBlobBackupDenial::Io(_) => BackupArtifactSemanticDefectKind::Io,
        BoundedBlobBackupDenial::AllocationFailed => {
            BackupArtifactSemanticDefectKind::VerifierAllocationFailed
        }
        BoundedBlobBackupDenial::BufferTooSmall => {
            BackupArtifactSemanticDefectKind::BufferBudgetTooSmall
        }
        BoundedBlobBackupDenial::LengthMismatch { .. } => {
            BackupArtifactSemanticDefectKind::LengthMismatch
        }
        BoundedBlobBackupDenial::InvalidHeader | BoundedBlobBackupDenial::InvalidMetadata => {
            BackupArtifactSemanticDefectKind::MalformedOwnerEncoding
        }
        BoundedBlobBackupDenial::IdentityMismatch => {
            BackupArtifactSemanticDefectKind::OwnerBindingMismatch
        }
        BoundedBlobBackupDenial::ChecksumMismatch
        | BoundedBlobBackupDenial::ContentDigestMismatch
        | BoundedBlobBackupDenial::StoredDigestMismatch
        | BoundedBlobBackupDenial::InternalDigestMismatch => {
            BackupArtifactSemanticDefectKind::OwnerIntegrityMismatch
        }
        BoundedBlobBackupDenial::ArtifactDigestMismatch => {
            BackupArtifactSemanticDefectKind::DigestMismatch
        }
    }
}
