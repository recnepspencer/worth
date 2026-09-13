use std::io::Read;

use sha2::Digest;
use worth_store_physical_format::{
    decode_checkpoint_backup_artifact_from_reader, CheckpointBackupArtifactDecodeDenial,
    CheckpointBackupArtifactDecodeRequest, PageGenerationCell, RootPublicationCell,
};

#[derive(Debug, Clone, Copy)]
pub struct BoundedCheckpointBackupVerificationRequest<'a> {
    pub checkpoint_identity: &'a str,
    pub manifest_generation: u64,
    pub durable_checkpoint_lsn: u64,
    pub expected_root: RootPublicationCell,
    pub expected_authority_fingerprint: [u8; 32],
    pub expected_frontier_digest: [u8; 32],
    pub expected_bytes: u64,
    pub expected_digest: [u8; 32],
    pub max_buffer_bytes: usize,
}

#[derive(Debug)]
pub enum BoundedCheckpointBackupDenial {
    Io(std::io::Error),
    BufferTooSmall,
    AllocationFailed,
    LengthMismatch { expected: u64, actual: u64 },
    InvalidHeader,
    InvalidPageFrontier,
    BindingMismatch,
    InternalDigestMismatch,
    ArtifactDigestMismatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundedCheckpointBackupObservation {
    checkpoint_identity_digest: [u8; 32],
    manifest_generation: u64,
    durable_checkpoint_lsn: u64,
    root_reference: u64,
    root_generation: u64,
    covered_lsn_start: u64,
    covered_lsn_end_exclusive: u64,
    redo_lsn: u64,
    page_count: u64,
    bytes_read: u64,
    decoder_allocation_bytes: u64,
    peak_buffer_bytes: u64,
    artifact_digest: [u8; 32],
    frontier_digest: [u8; 32],
}

impl BoundedCheckpointBackupObservation {
    pub const fn checkpoint_identity_digest(self) -> [u8; 32] {
        self.checkpoint_identity_digest
    }

    pub const fn manifest_generation(self) -> u64 {
        self.manifest_generation
    }

    pub const fn durable_checkpoint_lsn(self) -> u64 {
        self.durable_checkpoint_lsn
    }

    pub const fn root_reference(self) -> u64 {
        self.root_reference
    }

    pub const fn root_generation(self) -> u64 {
        self.root_generation
    }

    pub const fn covered_lsn(self) -> (u64, u64) {
        (self.covered_lsn_start, self.covered_lsn_end_exclusive)
    }

    pub const fn redo_lsn(self) -> u64 {
        self.redo_lsn
    }

    pub const fn page_count(self) -> u64 {
        self.page_count
    }

    pub const fn bytes_read(self) -> u64 {
        self.bytes_read
    }

    pub const fn decoder_allocation_bytes(self) -> u64 {
        self.decoder_allocation_bytes
    }

    pub const fn peak_buffer_bytes(self) -> u64 {
        self.peak_buffer_bytes
    }

    pub const fn artifact_digest(self) -> [u8; 32] {
        self.artifact_digest
    }

    pub const fn frontier_digest(self) -> [u8; 32] {
        self.frontier_digest
    }
}

pub fn verify_bounded_checkpoint_backup_artifact_from_reader(
    reader: &mut impl Read,
    actual_bytes: u64,
    request: BoundedCheckpointBackupVerificationRequest<'_>,
) -> Result<BoundedCheckpointBackupObservation, BoundedCheckpointBackupDenial> {
    let decoded = decode_checkpoint_backup_artifact_from_reader(
        reader,
        actual_bytes,
        CheckpointBackupArtifactDecodeRequest::new(
            request.expected_bytes,
            request.expected_digest,
            request.max_buffer_bytes,
        ),
    )
    .map_err(map_decode_denial)?;
    let artifact = decoded.artifact();
    if artifact.checkpoint_identity() != request.checkpoint_identity
        || artifact.manifest_generation() != request.manifest_generation
        || artifact.durable_checkpoint_lsn() != request.durable_checkpoint_lsn
        || artifact.root_reference() != request.expected_root.root_reference()
        || artifact.root_generation() != request.expected_root.generation().get()
    {
        return Err(BoundedCheckpointBackupDenial::BindingMismatch);
    }
    let format_observation = decoded.observation();
    let covered_lsn = artifact.covered_lsn();
    let frontier_digest = checkpoint_backup_frontier_digest(
        request.expected_authority_fingerprint,
        artifact.checkpoint_identity(),
        artifact.manifest_generation(),
        artifact.durable_checkpoint_lsn(),
        request.expected_root,
        covered_lsn,
        artifact.redo_lsn(),
        artifact.pages(),
    );
    if frontier_digest != request.expected_frontier_digest {
        return Err(BoundedCheckpointBackupDenial::BindingMismatch);
    }
    Ok(BoundedCheckpointBackupObservation {
        checkpoint_identity_digest: sha2::Sha256::digest(request.checkpoint_identity.as_bytes())
            .into(),
        manifest_generation: artifact.manifest_generation(),
        durable_checkpoint_lsn: artifact.durable_checkpoint_lsn(),
        root_reference: artifact.root_reference().get(),
        root_generation: artifact.root_generation(),
        covered_lsn_start: covered_lsn.0,
        covered_lsn_end_exclusive: covered_lsn.1,
        redo_lsn: artifact.redo_lsn(),
        page_count: artifact.pages().len() as u64,
        bytes_read: format_observation.bytes_read(),
        decoder_allocation_bytes: format_observation.decoder_allocation_bytes(),
        peak_buffer_bytes: format_observation.peak_buffer_bytes(),
        artifact_digest: request.expected_digest,
        frontier_digest,
    })
}

pub fn checkpoint_backup_frontier_digest(
    authority_fingerprint: [u8; 32],
    checkpoint_identity: &str,
    manifest_generation: u64,
    durable_checkpoint_lsn: u64,
    root: RootPublicationCell,
    covered_lsn: (u64, u64),
    redo_lsn: u64,
    pages: &[(PageGenerationCell, u64)],
) -> [u8; 32] {
    let mut digest = sha2::Sha256::new();
    digest.update(b"worth-store:checkpoint-backup-frontier:v1\0");
    digest.update(authority_fingerprint);
    digest.update((checkpoint_identity.len() as u64).to_le_bytes());
    digest.update(checkpoint_identity.as_bytes());
    digest.update(manifest_generation.to_le_bytes());
    digest.update(durable_checkpoint_lsn.to_le_bytes());
    digest.update(root.root_reference().get().to_le_bytes());
    digest.update(root.generation().get().to_le_bytes());
    digest.update(covered_lsn.0.to_le_bytes());
    digest.update(covered_lsn.1.to_le_bytes());
    digest.update(redo_lsn.to_le_bytes());
    digest.update((pages.len() as u64).to_le_bytes());
    for (page, page_lsn) in pages {
        digest.update(page.segment_id().get().to_le_bytes());
        digest.update(page.page_id().get().to_le_bytes());
        digest.update(page.generation().get().to_le_bytes());
        digest.update(page_lsn.to_le_bytes());
    }
    digest.finalize().into()
}

fn map_decode_denial(
    denial: CheckpointBackupArtifactDecodeDenial,
) -> BoundedCheckpointBackupDenial {
    match denial {
        CheckpointBackupArtifactDecodeDenial::Io(denial) => {
            BoundedCheckpointBackupDenial::Io(denial)
        }
        CheckpointBackupArtifactDecodeDenial::BufferTooSmall => {
            BoundedCheckpointBackupDenial::BufferTooSmall
        }
        CheckpointBackupArtifactDecodeDenial::AllocationFailed => {
            BoundedCheckpointBackupDenial::AllocationFailed
        }
        CheckpointBackupArtifactDecodeDenial::LengthMismatch { expected, actual } => {
            BoundedCheckpointBackupDenial::LengthMismatch { expected, actual }
        }
        CheckpointBackupArtifactDecodeDenial::InvalidHeader
        | CheckpointBackupArtifactDecodeDenial::InvalidIdentity => {
            BoundedCheckpointBackupDenial::InvalidHeader
        }
        CheckpointBackupArtifactDecodeDenial::InvalidPageFrontier => {
            BoundedCheckpointBackupDenial::InvalidPageFrontier
        }
        CheckpointBackupArtifactDecodeDenial::InternalDigestMismatch => {
            BoundedCheckpointBackupDenial::InternalDigestMismatch
        }
        CheckpointBackupArtifactDecodeDenial::ArtifactDigestMismatch => {
            BoundedCheckpointBackupDenial::ArtifactDigestMismatch
        }
    }
}
