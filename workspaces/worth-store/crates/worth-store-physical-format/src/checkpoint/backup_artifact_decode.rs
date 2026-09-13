use std::io::Read;

use sha2::{Digest, Sha256};

use crate::{
    PageGenerationCell, PhysicalGeneration, PhysicalGenerationAuthority, PhysicalPageId,
    PhysicalReference, PhysicalReferenceAuthority, PhysicalRootReference, PhysicalSegmentId,
};

use super::backup_artifact::{
    CheckpointBackupArtifact, CheckpointBackupArtifactInput, CHECKPOINT_BACKUP_FOOTER_BYTES,
    CHECKPOINT_BACKUP_HEADER_BYTES, CHECKPOINT_BACKUP_MAGIC, CHECKPOINT_BACKUP_PAGE_ROW_BYTES,
    CHECKPOINT_BACKUP_VERSION,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckpointBackupArtifactDecodeRequest {
    expected_bytes: u64,
    expected_digest: [u8; 32],
    max_buffer_bytes: usize,
}

impl CheckpointBackupArtifactDecodeRequest {
    pub const fn new(
        expected_bytes: u64,
        expected_digest: [u8; 32],
        max_buffer_bytes: usize,
    ) -> Self {
        Self {
            expected_bytes,
            expected_digest,
            max_buffer_bytes,
        }
    }
}

#[derive(Debug)]
pub enum CheckpointBackupArtifactDecodeDenial {
    Io(std::io::Error),
    BufferTooSmall,
    AllocationFailed,
    LengthMismatch { expected: u64, actual: u64 },
    InvalidHeader,
    InvalidIdentity,
    InvalidPageFrontier,
    InternalDigestMismatch,
    ArtifactDigestMismatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CheckpointBackupArtifactDecodeObservation {
    bytes_read: u64,
    decoder_allocation_bytes: u64,
    peak_buffer_bytes: u64,
}

impl CheckpointBackupArtifactDecodeObservation {
    pub const fn bytes_read(self) -> u64 {
        self.bytes_read
    }

    pub const fn decoder_allocation_bytes(self) -> u64 {
        self.decoder_allocation_bytes
    }

    pub const fn peak_buffer_bytes(self) -> u64 {
        self.peak_buffer_bytes
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedCheckpointBackupArtifact {
    artifact: CheckpointBackupArtifact,
    observation: CheckpointBackupArtifactDecodeObservation,
}

impl DecodedCheckpointBackupArtifact {
    pub const fn artifact(&self) -> &CheckpointBackupArtifact {
        &self.artifact
    }

    pub const fn observation(&self) -> CheckpointBackupArtifactDecodeObservation {
        self.observation
    }
}

pub fn decode_checkpoint_backup_artifact_from_reader(
    reader: &mut impl Read,
    actual_bytes: u64,
    request: CheckpointBackupArtifactDecodeRequest,
) -> Result<DecodedCheckpointBackupArtifact, CheckpointBackupArtifactDecodeDenial> {
    if request.max_buffer_bytes <= CHECKPOINT_BACKUP_HEADER_BYTES + CHECKPOINT_BACKUP_FOOTER_BYTES {
        return Err(CheckpointBackupArtifactDecodeDenial::BufferTooSmall);
    }
    if actual_bytes != request.expected_bytes {
        return Err(CheckpointBackupArtifactDecodeDenial::LengthMismatch {
            expected: request.expected_bytes,
            actual: actual_bytes,
        });
    }
    let mut header = [0_u8; CHECKPOINT_BACKUP_HEADER_BYTES];
    reader
        .read_exact(&mut header)
        .map_err(CheckpointBackupArtifactDecodeDenial::Io)?;
    let fields = decode_header(&header)?;
    let encoded_bytes = fields
        .page_count
        .checked_mul(CHECKPOINT_BACKUP_PAGE_ROW_BYTES as u64)
        .and_then(|bytes| bytes.checked_add(CHECKPOINT_BACKUP_HEADER_BYTES as u64))
        .and_then(|bytes| bytes.checked_add(fields.identity_bytes))
        .and_then(|bytes| bytes.checked_add(CHECKPOINT_BACKUP_FOOTER_BYTES as u64))
        .ok_or(CheckpointBackupArtifactDecodeDenial::InvalidHeader)?;
    if encoded_bytes != actual_bytes {
        return Err(CheckpointBackupArtifactDecodeDenial::InvalidHeader);
    }
    let row_count = usize::try_from(fields.page_count)
        .map_err(|_| CheckpointBackupArtifactDecodeDenial::AllocationFailed)?;
    let identity_len = usize::try_from(fields.identity_bytes)
        .map_err(|_| CheckpointBackupArtifactDecodeDenial::InvalidIdentity)?;
    let page_storage_bytes = u64::try_from(row_count)
        .ok()
        .and_then(|count| {
            count.checked_mul(std::mem::size_of::<(PageGenerationCell, u64)>() as u64)
        })
        .ok_or(CheckpointBackupArtifactDecodeDenial::AllocationFailed)?;
    let decoder_allocation_bytes = page_storage_bytes
        .checked_add(identity_len as u64)
        .ok_or(CheckpointBackupArtifactDecodeDenial::AllocationFailed)?;
    let peak_buffer_bytes = decoder_allocation_bytes
        .checked_add(
            (CHECKPOINT_BACKUP_HEADER_BYTES
                + CHECKPOINT_BACKUP_FOOTER_BYTES
                + CHECKPOINT_BACKUP_PAGE_ROW_BYTES) as u64,
        )
        .ok_or(CheckpointBackupArtifactDecodeDenial::AllocationFailed)?;
    if peak_buffer_bytes > request.max_buffer_bytes as u64 {
        return Err(CheckpointBackupArtifactDecodeDenial::BufferTooSmall);
    }

    let mut internal_digest = Sha256::new();
    let mut artifact_digest = Sha256::new();
    internal_digest.update(header);
    artifact_digest.update(header);
    let mut pages = Vec::new();
    pages
        .try_reserve_exact(row_count)
        .map_err(|_| CheckpointBackupArtifactDecodeDenial::AllocationFailed)?;
    let mut previous_page: Option<(PageGenerationCell, u64)> = None;
    for _ in 0..fields.page_count {
        let mut row = [0_u8; CHECKPOINT_BACKUP_PAGE_ROW_BYTES];
        reader
            .read_exact(&mut row)
            .map_err(CheckpointBackupArtifactDecodeDenial::Io)?;
        internal_digest.update(row);
        artifact_digest.update(row);
        let page = decode_page_row(&row, fields.redo_lsn)?;
        if previous_page.is_some_and(|previous| previous.0 >= page.0) {
            return Err(CheckpointBackupArtifactDecodeDenial::InvalidPageFrontier);
        }
        previous_page = Some(page);
        pages.push(page);
    }

    let mut identity = Vec::new();
    identity
        .try_reserve_exact(identity_len)
        .map_err(|_| CheckpointBackupArtifactDecodeDenial::AllocationFailed)?;
    identity.resize(identity_len, 0);
    reader
        .read_exact(&mut identity)
        .map_err(CheckpointBackupArtifactDecodeDenial::Io)?;
    internal_digest.update(&identity);
    artifact_digest.update(&identity);
    let identity = String::from_utf8(identity)
        .map_err(|_| CheckpointBackupArtifactDecodeDenial::InvalidIdentity)?;

    let mut footer = [0_u8; CHECKPOINT_BACKUP_FOOTER_BYTES];
    reader
        .read_exact(&mut footer)
        .map_err(CheckpointBackupArtifactDecodeDenial::Io)?;
    if internal_digest.finalize()[..] != footer {
        return Err(CheckpointBackupArtifactDecodeDenial::InternalDigestMismatch);
    }
    artifact_digest.update(footer);
    if <[u8; 32]>::from(artifact_digest.finalize()) != request.expected_digest {
        return Err(CheckpointBackupArtifactDecodeDenial::ArtifactDigestMismatch);
    }

    let root = decode_root_reference(fields.root_reference, fields.root_generation)?;
    let artifact = CheckpointBackupArtifact::from_input(CheckpointBackupArtifactInput {
        checkpoint_identity: identity,
        manifest_generation: fields.manifest_generation,
        durable_checkpoint_lsn: fields.durable_lsn,
        root,
        covered_lsn: (fields.covered_start, fields.covered_end),
        redo_lsn: fields.redo_lsn,
        pages,
    })
    .ok_or(CheckpointBackupArtifactDecodeDenial::InvalidHeader)?;
    Ok(DecodedCheckpointBackupArtifact {
        artifact,
        observation: CheckpointBackupArtifactDecodeObservation {
            bytes_read: actual_bytes,
            decoder_allocation_bytes,
            peak_buffer_bytes,
        },
    })
}

#[derive(Debug, Clone, Copy)]
struct HeaderFields {
    manifest_generation: u64,
    durable_lsn: u64,
    root_reference: u64,
    root_generation: u64,
    covered_start: u64,
    covered_end: u64,
    redo_lsn: u64,
    page_count: u64,
    identity_bytes: u64,
}

fn decode_header(
    header: &[u8; CHECKPOINT_BACKUP_HEADER_BYTES],
) -> Result<HeaderFields, CheckpointBackupArtifactDecodeDenial> {
    if &header[0..8] != CHECKPOINT_BACKUP_MAGIC || read_u16(header, 8) != CHECKPOINT_BACKUP_VERSION
    {
        return Err(CheckpointBackupArtifactDecodeDenial::InvalidHeader);
    }
    let fields = HeaderFields {
        manifest_generation: read_u64(header, 10),
        durable_lsn: read_u64(header, 18),
        root_reference: read_u64(header, 26),
        root_generation: read_u64(header, 34),
        covered_start: read_u64(header, 42),
        covered_end: read_u64(header, 50),
        redo_lsn: read_u64(header, 58),
        page_count: read_u64(header, 66),
        identity_bytes: u64::from(read_u32(header, 74)),
    };
    if fields.manifest_generation == 0
        || fields.root_reference == 0
        || fields.root_generation == 0
        || fields.covered_start >= fields.covered_end
        || fields.redo_lsn < fields.covered_start
        || fields.redo_lsn >= fields.covered_end
        || fields.durable_lsn < fields.redo_lsn
        || fields.durable_lsn > fields.covered_end
        || fields.page_count == 0
        || fields.identity_bytes == 0
    {
        return Err(CheckpointBackupArtifactDecodeDenial::InvalidHeader);
    }
    Ok(fields)
}

fn decode_root_reference(
    root_reference: u64,
    root_generation: u64,
) -> Result<PhysicalReference, CheckpointBackupArtifactDecodeDenial> {
    let root_reference = PhysicalRootReference::from_raw(root_reference)
        .map_err(|_| CheckpointBackupArtifactDecodeDenial::InvalidHeader)?;
    let root_generation = PhysicalGeneration::from_raw(root_generation)
        .map_err(|_| CheckpointBackupArtifactDecodeDenial::InvalidHeader)?;
    let cell = PhysicalGenerationAuthority::for_canonical_physical_format()
        .root_publication_cell(root_reference)
        .with_root_publication_generation(root_generation);
    Ok(PhysicalReferenceAuthority::for_canonical_physical_format()
        .admit_root_publication(cell)
        .reference())
}

fn decode_page_row(
    row: &[u8; CHECKPOINT_BACKUP_PAGE_ROW_BYTES],
    redo_lsn: u64,
) -> Result<(PageGenerationCell, u64), CheckpointBackupArtifactDecodeDenial> {
    let segment_id = PhysicalSegmentId::from_raw(read_u64(row, 0))
        .map_err(|_| CheckpointBackupArtifactDecodeDenial::InvalidPageFrontier)?;
    let page_id = PhysicalPageId::from_raw(read_u64(row, 8))
        .map_err(|_| CheckpointBackupArtifactDecodeDenial::InvalidPageFrontier)?;
    let generation = PhysicalGeneration::from_raw(read_u64(row, 16))
        .map_err(|_| CheckpointBackupArtifactDecodeDenial::InvalidPageFrontier)?;
    let page_lsn = read_u64(row, 24);
    if page_lsn < redo_lsn {
        return Err(CheckpointBackupArtifactDecodeDenial::InvalidPageFrontier);
    }
    let page = PhysicalGenerationAuthority::for_canonical_physical_format()
        .page_cell(segment_id, page_id)
        .with_page_generation(generation);
    Ok((page, page_lsn))
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(bytes[offset..offset + 2].try_into().expect("fixed field"))
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().expect("fixed field"))
}

fn read_u64(bytes: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(bytes[offset..offset + 8].try_into().expect("fixed field"))
}
