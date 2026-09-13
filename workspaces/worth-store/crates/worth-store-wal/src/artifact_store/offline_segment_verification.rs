use std::io::Read;
use std::path::Path;

use sha2::{Digest, Sha256};
use worth_store_physical_format::wal_frame::{
    decode_wal_frame_v1_header, WalFrameV1ChecksumCalculator, WAL_FRAME_V1_FOOTER_BYTES,
    WAL_FRAME_V1_HEADER_BYTES,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundedWalSegmentVerificationRequest {
    segment_id: u64,
    generation: u64,
    start_lsn: u64,
    end_exclusive_lsn: u64,
    expected_bytes: u64,
    expected_digest: [u8; 32],
    max_buffer_bytes: usize,
}

impl BoundedWalSegmentVerificationRequest {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        segment_id: u64,
        generation: u64,
        start_lsn: u64,
        end_exclusive_lsn: u64,
        expected_bytes: u64,
        expected_digest: [u8; 32],
        max_buffer_bytes: usize,
    ) -> Option<Self> {
        (segment_id > 0
            && generation > 0
            && start_lsn < end_exclusive_lsn
            && expected_bytes > 0
            && max_buffer_bytes > WAL_FRAME_V1_HEADER_BYTES + WAL_FRAME_V1_FOOTER_BYTES)
            .then_some(Self {
                segment_id,
                generation,
                start_lsn,
                end_exclusive_lsn,
                expected_bytes,
                expected_digest,
                max_buffer_bytes,
            })
    }
}

#[derive(Debug)]
pub enum BoundedWalSegmentDenial {
    Io(std::io::Error),
    AllocationFailed,
    CounterOverflow,
    LengthMismatch { expected: u64, actual: u64 },
    InvalidFrame,
    FrameDigestMismatch,
    PayloadDigestMismatch,
    ArtifactDigestMismatch,
    SegmentBindingMismatch,
    GenerationBindingMismatch,
    CoverageMismatch,
    NonContiguousLsn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundedWalSegmentObservation {
    segment_id: u64,
    generation: u64,
    start_lsn: u64,
    end_exclusive_lsn: u64,
    frame_count: u64,
    bytes_read: u64,
    decoder_allocation_bytes: u64,
    peak_buffer_bytes: u64,
    artifact_digest: [u8; 32],
}

impl BoundedWalSegmentObservation {
    pub const fn segment_id(self) -> u64 {
        self.segment_id
    }

    pub const fn generation(self) -> u64 {
        self.generation
    }

    pub const fn lsn_interval(self) -> (u64, u64) {
        (self.start_lsn, self.end_exclusive_lsn)
    }

    pub const fn frame_count(self) -> u64 {
        self.frame_count
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
}

pub fn verify_bounded_wal_segment(
    path: &Path,
    request: BoundedWalSegmentVerificationRequest,
) -> Result<BoundedWalSegmentObservation, BoundedWalSegmentDenial> {
    let mut file = std::fs::File::open(path).map_err(BoundedWalSegmentDenial::Io)?;
    let actual = file.metadata().map_err(BoundedWalSegmentDenial::Io)?.len();
    verify_bounded_wal_segment_from_reader(&mut file, actual, request)
}

pub fn verify_bounded_wal_segment_from_reader(
    reader: &mut impl Read,
    actual: u64,
    request: BoundedWalSegmentVerificationRequest,
) -> Result<BoundedWalSegmentObservation, BoundedWalSegmentDenial> {
    if actual != request.expected_bytes {
        return Err(BoundedWalSegmentDenial::LengthMismatch {
            expected: request.expected_bytes,
            actual,
        });
    }

    let chunk_bytes =
        (request.max_buffer_bytes - WAL_FRAME_V1_HEADER_BYTES - WAL_FRAME_V1_FOOTER_BYTES)
            .min(64 * 1024);
    let mut buffer = Vec::new();
    buffer
        .try_reserve_exact(chunk_bytes)
        .map_err(|_| BoundedWalSegmentDenial::AllocationFailed)?;
    buffer.resize(chunk_bytes, 0);
    let mut artifact_digest = Sha256::new();
    let mut bytes_read = 0_u64;
    let mut frame_count = 0_u64;
    let mut prior_lsn_end = None;

    while bytes_read < actual {
        if actual - bytes_read < (WAL_FRAME_V1_HEADER_BYTES + WAL_FRAME_V1_FOOTER_BYTES) as u64 {
            return Err(BoundedWalSegmentDenial::InvalidFrame);
        }
        let mut header = [0_u8; WAL_FRAME_V1_HEADER_BYTES];
        reader
            .read_exact(&mut header)
            .map_err(BoundedWalSegmentDenial::Io)?;
        artifact_digest.update(header);
        let fields = decode_wal_frame_v1_header(&header)
            .map_err(|_| BoundedWalSegmentDenial::InvalidFrame)?;
        if fields.segment_id() != request.segment_id {
            return Err(BoundedWalSegmentDenial::SegmentBindingMismatch);
        }
        if fields.generation() != request.generation {
            return Err(BoundedWalSegmentDenial::GenerationBindingMismatch);
        }
        if prior_lsn_end.is_some_and(|prior| prior != fields.lsn_start()) {
            return Err(BoundedWalSegmentDenial::NonContiguousLsn);
        }
        if frame_count == 0 && fields.lsn_start() != request.start_lsn {
            return Err(BoundedWalSegmentDenial::CoverageMismatch);
        }
        let encoded_bytes =
            (WAL_FRAME_V1_HEADER_BYTES + WAL_FRAME_V1_FOOTER_BYTES) as u64 + fields.payload_bytes();
        if encoded_bytes > actual - bytes_read {
            return Err(BoundedWalSegmentDenial::InvalidFrame);
        }

        let mut checksums = WalFrameV1ChecksumCalculator::new(&header);
        let mut payload_remaining = fields.payload_bytes();
        while payload_remaining > 0 {
            let take = usize::try_from(payload_remaining.min(chunk_bytes as u64))
                .expect("bounded WAL chunk fits usize");
            reader
                .read_exact(&mut buffer[..take])
                .map_err(BoundedWalSegmentDenial::Io)?;
            checksums
                .update_payload(&buffer[..take])
                .map_err(|_| BoundedWalSegmentDenial::InvalidFrame)?;
            artifact_digest.update(&buffer[..take]);
            payload_remaining -= take as u64;
        }
        let mut footer = [0_u8; WAL_FRAME_V1_FOOTER_BYTES];
        reader
            .read_exact(&mut footer)
            .map_err(BoundedWalSegmentDenial::Io)?;
        artifact_digest.update(footer);
        let calculated = checksums
            .finish_calculation(fields)
            .map_err(|_| BoundedWalSegmentDenial::InvalidFrame)?;
        if calculated.payload() != fields.payload_digest() {
            return Err(BoundedWalSegmentDenial::PayloadDigestMismatch);
        }
        if calculated.frame() != footer {
            return Err(BoundedWalSegmentDenial::FrameDigestMismatch);
        }

        prior_lsn_end = Some(fields.lsn_end());
        frame_count = frame_count
            .checked_add(1)
            .ok_or(BoundedWalSegmentDenial::CounterOverflow)?;
        bytes_read = bytes_read
            .checked_add(encoded_bytes)
            .ok_or(BoundedWalSegmentDenial::CounterOverflow)?;
    }

    if frame_count == 0 || prior_lsn_end != Some(request.end_exclusive_lsn) {
        return Err(BoundedWalSegmentDenial::CoverageMismatch);
    }
    if <[u8; 32]>::from(artifact_digest.finalize()) != request.expected_digest {
        return Err(BoundedWalSegmentDenial::ArtifactDigestMismatch);
    }
    Ok(BoundedWalSegmentObservation {
        segment_id: request.segment_id,
        generation: request.generation,
        start_lsn: request.start_lsn,
        end_exclusive_lsn: request.end_exclusive_lsn,
        frame_count,
        bytes_read,
        decoder_allocation_bytes: chunk_bytes as u64,
        peak_buffer_bytes: (WAL_FRAME_V1_HEADER_BYTES + WAL_FRAME_V1_FOOTER_BYTES + chunk_bytes)
            as u64,
        artifact_digest: request.expected_digest,
    })
}
