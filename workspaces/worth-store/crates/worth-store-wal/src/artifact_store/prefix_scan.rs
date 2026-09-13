use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

use worth_store_physical_format::wal_frame::{
    decode_wal_frame_v1_header, WalFrameV1ChecksumCalculator, WAL_FRAME_V1_FOOTER_BYTES,
    WAL_FRAME_V1_HEADER_BYTES,
};

use super::segment_inventory::wal_segment_relative_path;
use super::WalArtifactStoreDenial;

pub(super) const WAL_SCAN_BUFFER_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct WalPrefixScan {
    pub valid_prefix_bytes: u64,
    pub observed_file_bytes: u64,
    pub last_lsn_end: Option<u64>,
    pub bytes_scanned: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct WalFrameObservation {
    pub payload_offset: u64,
    pub payload_bytes: u64,
    pub payload_digest: [u8; 32],
    pub lsn_end: u64,
    pub encoded_end: u64,
}

pub(super) fn scan_segment_path(
    root: &Path,
    segment_id: u64,
    generation: u64,
) -> Result<WalPrefixScan, WalArtifactStoreDenial> {
    let mut buffer = vec![0; WAL_SCAN_BUFFER_BYTES];
    scan_segment_path_with_buffer(root, segment_id, generation, &mut buffer)
}

pub(super) fn scan_segment_path_with_buffer(
    root: &Path,
    segment_id: u64,
    generation: u64,
    buffer: &mut [u8],
) -> Result<WalPrefixScan, WalArtifactStoreDenial> {
    let path = root.join(wal_segment_relative_path(segment_id, generation)?);
    let mut file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(empty_prefix()),
        Err(_) => return Err(WalArtifactStoreDenial::Io),
    };
    scan_segment_reader(&mut file, 0, None, segment_id, generation, buffer, |_| {})
}

#[cfg(feature = "certification-authority")]
pub(super) fn scan_segment_suffix_with_buffer(
    root: &Path,
    segment_id: u64,
    generation: u64,
    valid_prefix_bytes: u64,
    last_lsn_end: Option<u64>,
    buffer: &mut [u8],
) -> Result<WalPrefixScan, WalArtifactStoreDenial> {
    let path = root.join(wal_segment_relative_path(segment_id, generation)?);
    let mut file = match std::fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound && valid_prefix_bytes == 0 => {
            return Ok(empty_prefix());
        }
        Err(_) => return Err(WalArtifactStoreDenial::Io),
    };
    if file
        .metadata()
        .map_err(|_| WalArtifactStoreDenial::Io)?
        .len()
        < valid_prefix_bytes
    {
        return scan_segment_reader(&mut file, 0, None, segment_id, generation, buffer, |_| {});
    }
    scan_segment_reader(
        &mut file,
        valid_prefix_bytes,
        last_lsn_end,
        segment_id,
        generation,
        buffer,
        |_| {},
    )
}

pub(super) fn scan_segment_reader(
    file: &mut std::fs::File,
    initial_offset: u64,
    initial_lsn_end: Option<u64>,
    segment_id: u64,
    generation: u64,
    chunk: &mut [u8],
    mut observe: impl FnMut(WalFrameObservation),
) -> Result<WalPrefixScan, WalArtifactStoreDenial> {
    if chunk.is_empty() {
        return Err(WalArtifactStoreDenial::InvalidFrame);
    }
    let observed_file_bytes = file
        .metadata()
        .map_err(|_| WalArtifactStoreDenial::Io)?
        .len();
    file.seek(SeekFrom::Start(initial_offset))
        .map_err(|_| WalArtifactStoreDenial::Io)?;
    let mut offset = initial_offset;
    let mut last_lsn_end = initial_lsn_end;
    let mut bytes_scanned = 0u64;
    while offset < observed_file_bytes {
        if observed_file_bytes - offset < WAL_FRAME_V1_HEADER_BYTES as u64 {
            break;
        }
        let mut header = [0u8; WAL_FRAME_V1_HEADER_BYTES];
        file.read_exact(&mut header)
            .map_err(|_| WalArtifactStoreDenial::Io)?;
        let fields = decode_wal_frame_v1_header(&header).map_err(super::map_wal_frame_denial)?;
        if fields.segment_id() != segment_id || fields.generation() != generation {
            return Err(WalArtifactStoreDenial::StoreBindingMismatch);
        }
        if last_lsn_end.is_some_and(|last| last != fields.lsn_start()) {
            return Err(WalArtifactStoreDenial::NonContiguousLsn);
        }
        let encoded_bytes = ((WAL_FRAME_V1_HEADER_BYTES + WAL_FRAME_V1_FOOTER_BYTES) as u64)
            .checked_add(fields.payload_bytes())
            .ok_or(WalArtifactStoreDenial::InvalidFrame)?;
        if encoded_bytes > observed_file_bytes - offset {
            break;
        }
        verify_frame(file, chunk, &header, fields)?;
        observe(WalFrameObservation {
            payload_offset: offset + WAL_FRAME_V1_HEADER_BYTES as u64,
            payload_bytes: fields.payload_bytes(),
            payload_digest: fields.payload_digest(),
            lsn_end: fields.lsn_end(),
            encoded_end: offset + encoded_bytes,
        });
        offset = offset
            .checked_add(encoded_bytes)
            .ok_or(WalArtifactStoreDenial::InvalidFrame)?;
        bytes_scanned = bytes_scanned
            .checked_add(encoded_bytes)
            .ok_or(WalArtifactStoreDenial::InvalidFrame)?;
        last_lsn_end = Some(fields.lsn_end());
    }
    Ok(WalPrefixScan {
        valid_prefix_bytes: offset,
        observed_file_bytes,
        last_lsn_end,
        bytes_scanned,
    })
}

fn verify_frame(
    file: &mut std::fs::File,
    chunk: &mut [u8],
    header_bytes: &[u8; WAL_FRAME_V1_HEADER_BYTES],
    header: worth_store_physical_format::wal_frame::WalFrameV1Header,
) -> Result<(), WalArtifactStoreDenial> {
    let mut checksums = WalFrameV1ChecksumCalculator::new(header_bytes);
    let mut remaining = header.payload_bytes();
    while remaining > 0 {
        let take = usize::try_from(remaining.min(chunk.len() as u64))
            .expect("bounded WAL scan chunk fits usize");
        file.read_exact(&mut chunk[..take])
            .map_err(|_| WalArtifactStoreDenial::Io)?;
        checksums
            .update_payload(&chunk[..take])
            .map_err(super::map_wal_frame_denial)?;
        remaining -= take as u64;
    }
    let mut footer = [0u8; WAL_FRAME_V1_FOOTER_BYTES];
    file.read_exact(&mut footer)
        .map_err(|_| WalArtifactStoreDenial::Io)?;
    checksums
        .finish(header, &footer)
        .map_err(super::map_wal_frame_denial)?;
    Ok(())
}

const fn empty_prefix() -> WalPrefixScan {
    WalPrefixScan {
        valid_prefix_bytes: 0,
        observed_file_bytes: 0,
        last_lsn_end: None,
        bytes_scanned: 0,
    }
}
