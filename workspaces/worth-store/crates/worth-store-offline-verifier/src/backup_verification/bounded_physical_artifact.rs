use std::io::Read;

use sha2::{Digest, Sha256};

use worth_store_physical_format::{
    ExtentGenerationCell, OfflineManifestCodec, OfflineVerifierDenial, PageGenerationCell,
    PhysicalBinaryEncodingWitness, PhysicalByteOrder, PhysicalFrameKind, PhysicalHeaderAuthority,
    PhysicalHeaderDecodeDenial, PhysicalPageKind, PhysicalPublicationState,
    PhysicalReferenceAuthority, RootPublicationCell, PHYSICAL_HEADER_LENGTH,
};

const ROOT_MANIFEST_BYTES: usize = 20;

#[derive(Debug)]
pub enum BoundedPhysicalArtifactDenial {
    Io(std::io::Error),
    BufferTooSmall { required: usize, actual: usize },
    AllocationFailed,
    LengthMismatch { expected: u64, actual: u64 },
    DigestMismatch,
    RootDecode(Box<OfflineVerifierDenial>),
    HeaderDecode(PhysicalHeaderDecodeDenial),
    ReferenceMismatch,
    UnpublishedArtifact,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundedPhysicalArtifactObservation {
    bytes_read: u64,
    decoder_allocation_bytes: u64,
    peak_buffer_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VerifiedRootManifestArtifact {
    observation: BoundedPhysicalArtifactObservation,
    root: RootPublicationCell,
    content_digest: [u8; 32],
}

impl VerifiedRootManifestArtifact {
    pub const fn observation(self) -> BoundedPhysicalArtifactObservation {
        self.observation
    }

    pub const fn root(self) -> RootPublicationCell {
        self.root
    }

    pub const fn content_digest(self) -> [u8; 32] {
        self.content_digest
    }
}

impl BoundedPhysicalArtifactObservation {
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

pub fn verify_bounded_root_manifest_artifact_from_reader(
    reader: &mut impl Read,
    actual_bytes: u64,
    expected: RootPublicationCell,
    expected_bytes: u64,
    expected_digest: [u8; 32],
    max_buffer_bytes: usize,
) -> Result<VerifiedRootManifestArtifact, BoundedPhysicalArtifactDenial> {
    require_buffer(max_buffer_bytes, ROOT_MANIFEST_BYTES)?;
    require_length(expected_bytes, actual_bytes)?;
    let mut bytes = [0_u8; ROOT_MANIFEST_BYTES];
    reader
        .read_exact(&mut bytes)
        .map_err(BoundedPhysicalArtifactDenial::Io)?;
    verify_digest(&bytes, expected_digest)?;
    let decoded =
        OfflineManifestCodec::decode_root_manifest(PhysicalByteOrder::LittleEndian, &bytes)
            .map_err(|denial| BoundedPhysicalArtifactDenial::RootDecode(Box::new(denial)))?;
    if decoded != expected {
        return Err(BoundedPhysicalArtifactDenial::ReferenceMismatch);
    }
    Ok(VerifiedRootManifestArtifact {
        observation: BoundedPhysicalArtifactObservation {
            bytes_read: expected_bytes,
            decoder_allocation_bytes: 0,
            peak_buffer_bytes: ROOT_MANIFEST_BYTES as u64,
        },
        root: expected,
        content_digest: expected_digest,
    })
}

pub fn verify_bounded_page_artifact_from_reader(
    reader: &mut impl Read,
    actual_bytes: u64,
    expected: PageGenerationCell,
    expected_bytes: u64,
    expected_digest: [u8; 32],
    max_buffer_bytes: usize,
) -> Result<BoundedPhysicalArtifactObservation, BoundedPhysicalArtifactDenial> {
    verify_header_framed_artifact(
        reader,
        actual_bytes,
        expected_bytes,
        expected_digest,
        max_buffer_bytes,
        |header| {
            canonical_headers()
                .decode_page_header_prefix(expected, header, PhysicalPageKind::DataPage)
                .map(|report| report.witness())
                .map_err(BoundedPhysicalArtifactDenial::HeaderDecode)
        },
    )
}

pub fn verify_bounded_extent_artifact_from_reader(
    reader: &mut impl Read,
    actual_bytes: u64,
    expected: ExtentGenerationCell,
    expected_bytes: u64,
    expected_digest: [u8; 32],
    max_buffer_bytes: usize,
) -> Result<BoundedPhysicalArtifactObservation, BoundedPhysicalArtifactDenial> {
    let references = PhysicalReferenceAuthority::for_canonical_physical_format();
    let validation = references
        .validate_extent(references.admit_extent(expected), expected)
        .map_err(|_| BoundedPhysicalArtifactDenial::ReferenceMismatch)?;
    verify_header_framed_artifact(
        reader,
        actual_bytes,
        expected_bytes,
        expected_digest,
        max_buffer_bytes,
        |header| {
            canonical_headers()
                .decode_frame_header_prefix(
                    validation,
                    header,
                    PhysicalFrameKind::ExtentRecordFrame,
                )
                .map(|report| report.witness())
                .map_err(BoundedPhysicalArtifactDenial::HeaderDecode)
        },
    )
}

fn verify_header_framed_artifact(
    reader: &mut impl Read,
    actual_bytes: u64,
    expected_bytes: u64,
    expected_digest: [u8; 32],
    max_buffer_bytes: usize,
    decode: impl FnOnce(
        &[u8],
    ) -> Result<
        worth_store_physical_format::PhysicalHeaderDecodeWitness,
        BoundedPhysicalArtifactDenial,
    >,
) -> Result<BoundedPhysicalArtifactObservation, BoundedPhysicalArtifactDenial> {
    let header_bytes = PHYSICAL_HEADER_LENGTH as usize;
    require_buffer(max_buffer_bytes, header_bytes + 1)?;
    require_length(expected_bytes, actual_bytes)?;
    let mut header = [0_u8; PHYSICAL_HEADER_LENGTH as usize];
    reader
        .read_exact(&mut header)
        .map_err(BoundedPhysicalArtifactDenial::Io)?;
    let witness = decode(&header)?;
    if witness.publication() != PhysicalPublicationState::Published {
        return Err(BoundedPhysicalArtifactDenial::UnpublishedArtifact);
    }
    let framed_bytes = header_bytes as u64 + u64::from(witness.payload_length());
    if framed_bytes != expected_bytes {
        return Err(BoundedPhysicalArtifactDenial::LengthMismatch {
            expected: framed_bytes,
            actual: expected_bytes,
        });
    }
    let chunk_bytes = (max_buffer_bytes - header_bytes).min(64 * 1024);
    let mut buffer = Vec::new();
    buffer
        .try_reserve_exact(chunk_bytes)
        .map_err(|_| BoundedPhysicalArtifactDenial::AllocationFailed)?;
    buffer.resize(chunk_bytes, 0);
    let mut digest = Sha256::new();
    digest.update(header);
    let mut remaining = u64::from(witness.payload_length());
    while remaining > 0 {
        let take = usize::try_from(remaining.min(chunk_bytes as u64))
            .expect("bounded chunk length fits usize");
        reader
            .read_exact(&mut buffer[..take])
            .map_err(BoundedPhysicalArtifactDenial::Io)?;
        digest.update(&buffer[..take]);
        remaining -= take as u64;
    }
    if <[u8; 32]>::from(digest.finalize()) != expected_digest {
        return Err(BoundedPhysicalArtifactDenial::DigestMismatch);
    }
    Ok(BoundedPhysicalArtifactObservation {
        bytes_read: expected_bytes,
        decoder_allocation_bytes: chunk_bytes as u64,
        peak_buffer_bytes: (header_bytes + chunk_bytes) as u64,
    })
}

fn require_length(expected: u64, actual: u64) -> Result<(), BoundedPhysicalArtifactDenial> {
    if expected == actual {
        Ok(())
    } else {
        Err(BoundedPhysicalArtifactDenial::LengthMismatch { expected, actual })
    }
}

fn require_buffer(actual: usize, required: usize) -> Result<(), BoundedPhysicalArtifactDenial> {
    if actual < required {
        Err(BoundedPhysicalArtifactDenial::BufferTooSmall { required, actual })
    } else {
        Ok(())
    }
}

fn verify_digest(bytes: &[u8], expected: [u8; 32]) -> Result<(), BoundedPhysicalArtifactDenial> {
    if <[u8; 32]>::from(Sha256::digest(bytes)) != expected {
        Err(BoundedPhysicalArtifactDenial::DigestMismatch)
    } else {
        Ok(())
    }
}

fn canonical_headers() -> PhysicalHeaderAuthority {
    PhysicalHeaderAuthority::for_canonical_physical_format(
        PhysicalBinaryEncodingWitness::physical_format_canonical()
            .expect("canonical physical binary declaration is valid"),
    )
}
