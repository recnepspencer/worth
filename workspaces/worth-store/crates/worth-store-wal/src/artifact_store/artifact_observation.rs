use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

use worth_store_physical_format::wal_frame::{
    decode_wal_frame_v1_header, wal_frame_v1_declared_identity_digest,
    WalFrameV1ChecksumCalculator, WAL_FRAME_V1_FOOTER_BYTES, WAL_FRAME_V1_HEADER_BYTES,
};

use super::WalArtifactStoreDenial;
use crate::{CheckpointPublicationScope, WalFramePublicationScope};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalFrameArtifactObservation {
    scope: WalFramePublicationScope,
    path: PathBuf,
    frame_offset: u64,
    payload_offset: u64,
    payload_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckpointArtifactObservation {
    scope: CheckpointPublicationScope,
    path: PathBuf,
    bytes: u64,
}

pub fn observe_wal_frame_artifact(
    path: &Path,
    frame_offset: u64,
    frame_bytes: u64,
    scope: &WalFramePublicationScope,
) -> Result<WalFrameArtifactObservation, WalArtifactStoreDenial> {
    let path = std::fs::canonicalize(path).map_err(|_| WalArtifactStoreDenial::Io)?;
    let (payload_offset, payload_bytes) =
        validate_persisted_frame(&path, frame_offset, frame_bytes, scope)?;
    Ok(WalFrameArtifactObservation {
        scope: scope.clone(),
        path,
        frame_offset,
        payload_offset,
        payload_bytes,
    })
}

fn validate_persisted_frame(
    path: &Path,
    encoded_offset: u64,
    encoded_bytes: u64,
    scope: &WalFramePublicationScope,
) -> Result<(u64, u64), WalArtifactStoreDenial> {
    let mut file = std::fs::File::open(path).map_err(|_| WalArtifactStoreDenial::Io)?;
    let end = encoded_offset
        .checked_add(encoded_bytes)
        .ok_or(WalArtifactStoreDenial::InvalidFrame)?;
    if file
        .metadata()
        .map_err(|_| WalArtifactStoreDenial::Io)?
        .len()
        < end
        || encoded_bytes < (WAL_FRAME_V1_HEADER_BYTES + WAL_FRAME_V1_FOOTER_BYTES) as u64
    {
        return Err(WalArtifactStoreDenial::InvalidFrame);
    }
    file.seek(SeekFrom::Start(encoded_offset))
        .map_err(|_| WalArtifactStoreDenial::Io)?;
    let mut header_bytes = [0; WAL_FRAME_V1_HEADER_BYTES];
    file.read_exact(&mut header_bytes)
        .map_err(|_| WalArtifactStoreDenial::Io)?;
    let header = decode_wal_frame_v1_header(&header_bytes).map_err(super::map_wal_frame_denial)?;
    let expected_encoded = ((WAL_FRAME_V1_HEADER_BYTES + WAL_FRAME_V1_FOOTER_BYTES) as u64)
        .checked_add(header.payload_bytes())
        .ok_or(WalArtifactStoreDenial::InvalidFrame)?;
    if expected_encoded != encoded_bytes
        || header.segment_id() != scope.segment_id()
        || header.generation() != scope.generation()
        || header.lsn_start() != scope.lsn_start()
        || header.lsn_end() != scope.lsn_end()
        || header.identity_digest()
            != wal_frame_v1_declared_identity_digest(scope.frame_digest().as_bytes())
        || header.payload_bytes() != scope.expected_bytes()
    {
        return Err(WalArtifactStoreDenial::StoreBindingMismatch);
    }
    let mut calculator = WalFrameV1ChecksumCalculator::new(&header_bytes);
    let mut buffer = [0; super::prefix_scan::WAL_SCAN_BUFFER_BYTES];
    let mut remaining = header.payload_bytes();
    while remaining > 0 {
        let take = usize::try_from(remaining.min(buffer.len() as u64))
            .expect("bounded WAL validation chunk fits usize");
        file.read_exact(&mut buffer[..take])
            .map_err(|_| WalArtifactStoreDenial::Io)?;
        calculator
            .update_payload(&buffer[..take])
            .map_err(super::map_wal_frame_denial)?;
        remaining -= take as u64;
    }
    let mut footer = [0; WAL_FRAME_V1_FOOTER_BYTES];
    file.read_exact(&mut footer)
        .map_err(|_| WalArtifactStoreDenial::Io)?;
    calculator
        .finish(header, &footer)
        .map_err(super::map_wal_frame_denial)?;
    Ok((
        encoded_offset + WAL_FRAME_V1_HEADER_BYTES as u64,
        header.payload_bytes(),
    ))
}

pub fn observe_checkpoint_artifact(
    path: &Path,
    scope: &CheckpointPublicationScope,
    expected: &[u8],
) -> Result<CheckpointArtifactObservation, WalArtifactStoreDenial> {
    let path = std::fs::canonicalize(path).map_err(|_| WalArtifactStoreDenial::Io)?;
    let observed = std::fs::read(&path).map_err(|_| WalArtifactStoreDenial::Io)?;
    if observed != expected {
        return Err(WalArtifactStoreDenial::DigestMismatch);
    }
    Ok(CheckpointArtifactObservation {
        scope: scope.clone(),
        path,
        bytes: observed.len() as u64,
    })
}

impl WalFrameArtifactObservation {
    pub const fn scope(&self) -> &WalFramePublicationScope {
        &self.scope
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub const fn frame_offset(&self) -> u64 {
        self.frame_offset
    }

    pub const fn payload_offset(&self) -> u64 {
        self.payload_offset
    }

    pub const fn payload_bytes(&self) -> u64 {
        self.payload_bytes
    }

    pub fn payload_matches(&self, expected: &[u8]) -> bool {
        use std::io::{Read, Seek, SeekFrom};

        if self.payload_bytes != expected.len() as u64 {
            return false;
        }
        let mut observed = vec![0; expected.len()];
        std::fs::File::open(&self.path)
            .and_then(|mut file| {
                file.seek(SeekFrom::Start(self.payload_offset))?;
                file.read_exact(&mut observed)
            })
            .is_ok_and(|()| observed == expected)
    }
}

impl CheckpointArtifactObservation {
    pub const fn scope(&self) -> &CheckpointPublicationScope {
        &self.scope
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub const fn bytes(&self) -> u64 {
        self.bytes
    }
}
