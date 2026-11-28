//! Bounded binary frame encoding and zero-copy borrowed decoding for
//! historical chunk images and continuation pages.
//!
//! Every serialized count is validated against the remaining frame byte length
//! BEFORE calling `Vec::with_capacity`. Streaming reads use borrowed views
//! (`BorrowedChunkRow`, `BorrowedContinuationPage`) so skipped, filtered, and
//! oversized rows allocate ZERO owned payload or history buffers.

use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::storage::RelationalStorageSubject;
use worth_runtime_world::facade::{CanonicalKeyDomain, HistoricalReadDenial};

use super::chunk_format::{
    BorrowedChunkRow, BorrowedContinuationPage, ChunkMemberId, ContinuationPageImage,
    PhysicalChunkImage, PhysicalChunkRow,
};

const CHUNK_MAGIC: &[u8; 5] = b"WHBC3";
const CONT_MAGIC: &[u8; 5] = b"WHBP1";

// Minimum serialized byte sizes used to bound counts BEFORE allocation.
const MIN_LAYER_BYTES: usize = 4;
const MIN_PHYSICAL_ROW_BYTES: usize = 4 + 1 + 4 + 8 + 32 + 4 + 4 + 4 + 4;
const MIN_BYTES_FIELD: usize = 4;
const MIN_CONT_ID_BYTES: usize = 32 + 8;

fn corrupt() -> HistoricalReadDenial {
    HistoricalReadDenial::CorruptStorage {
        subject: RelationalStorageSubject::Region(PartitionId(0)),
    }
}

pub(crate) fn encode_chunk_image(image: &PhysicalChunkImage) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(CHUNK_MAGIC);
    out.extend_from_slice(&(image.layers.len() as u32).to_le_bytes());
    for layer in &image.layers {
        out.extend_from_slice(&(layer.len() as u32).to_le_bytes());
        for row in layer {
            out.extend_from_slice(&row.partition_id.to_le_bytes());
            out.push(row.domain.tag());
            out.extend_from_slice(&row.kind_id.to_le_bytes());
            out.extend_from_slice(&row.total_history_bytes.to_le_bytes());
            out.extend_from_slice(&row.history_versions_sha256);
            write_bytes(&mut out, &row.logical_key);
            write_bytes(&mut out, &row.payload);
            out.extend_from_slice(&(row.inline_history.len() as u32).to_le_bytes());
            for version in &row.inline_history {
                write_bytes(&mut out, version);
            }
            out.extend_from_slice(&(row.continuation_pages.len() as u32).to_le_bytes());
            for page in &row.continuation_pages {
                out.extend_from_slice(&page.sha256);
                out.extend_from_slice(&page.length.to_le_bytes());
            }
        }
    }
    out
}

/// Zero-copy borrowed scan of all physical rows in a chunk frame, allocating
/// ZERO owned payload, logical-key, or history `Vec<u8>` buffers.
pub(crate) fn decode_borrowed_chunk_rows(
    bytes: &[u8],
) -> Result<Vec<BorrowedChunkRow<'_>>, HistoricalReadDenial> {
    let mut r = Cursor { bytes, at: 0 };
    if r.take(5)? != CHUNK_MAGIC {
        return Err(corrupt());
    }
    let layer_count = r.bounded_count(MIN_LAYER_BYTES)?;
    let mut rows = Vec::new();
    for _ in 0..layer_count {
        let row_count = r.bounded_count(MIN_PHYSICAL_ROW_BYTES)?;
        rows.reserve(row_count);
        for _ in 0..row_count {
            let partition_id = r.u32()?;
            let domain = CanonicalKeyDomain::from_tag(r.u8()?).ok_or_else(corrupt)?;
            let kind_id = r.u32()?;
            let total_history_bytes = r.u64()?;
            let history_versions_sha256 = r.array_32()?;
            let logical_key = r.bytes_slice()?;
            let payload = r.bytes_slice()?;
            let inline_history_count = r.bounded_count(MIN_BYTES_FIELD)?;
            let inline_start = r.at;
            let mut inline_history_bytes = 0u64;
            for _ in 0..inline_history_count {
                let v = r.bytes_slice()?;
                inline_history_bytes = inline_history_bytes.saturating_add(v.len() as u64);
            }
            let inline_history_raw = &bytes[inline_start..r.at];
            let continuation_page_count = r.bounded_count(MIN_CONT_ID_BYTES)?;
            let cont_len = continuation_page_count
                .checked_mul(MIN_CONT_ID_BYTES)
                .ok_or_else(corrupt)?;
            let continuation_pages_raw = r.take(cont_len)?;
            rows.push(BorrowedChunkRow {
                partition_id,
                domain,
                kind_id,
                total_history_bytes,
                history_versions_sha256,
                logical_key,
                payload,
                inline_history_raw,
                inline_history_count,
                inline_history_bytes,
                continuation_pages_raw,
                continuation_page_count,
            });
        }
    }
    if r.at != bytes.len() {
        return Err(corrupt());
    }
    Ok(rows)
}

pub(crate) fn decode_chunk_image(bytes: &[u8]) -> Result<PhysicalChunkImage, HistoricalReadDenial> {
    let mut r = Cursor { bytes, at: 0 };
    if r.take(5)? != CHUNK_MAGIC {
        return Err(corrupt());
    }
    let layer_count = r.bounded_count(MIN_LAYER_BYTES)?;
    let mut layers = Vec::with_capacity(layer_count);
    for _ in 0..layer_count {
        let row_count = r.bounded_count(MIN_PHYSICAL_ROW_BYTES)?;
        let mut rows = Vec::with_capacity(row_count);
        for _ in 0..row_count {
            let partition_id = r.u32()?;
            let domain = CanonicalKeyDomain::from_tag(r.u8()?).ok_or_else(corrupt)?;
            let kind_id = r.u32()?;
            let total_history_bytes = r.u64()?;
            let history_versions_sha256 = r.array_32()?;
            let logical_key = r.bytes_slice()?.to_vec();
            let payload = r.bytes_slice()?.to_vec();
            let hist_count = r.bounded_count(MIN_BYTES_FIELD)?;
            let mut inline_history = Vec::with_capacity(hist_count);
            for _ in 0..hist_count {
                inline_history.push(r.bytes_slice()?.to_vec());
            }
            let cont_count = r.bounded_count(MIN_CONT_ID_BYTES)?;
            let mut continuation_pages = Vec::with_capacity(cont_count);
            for _ in 0..cont_count {
                let sha256 = r.array_32()?;
                let length = r.u64()?;
                continuation_pages.push(ChunkMemberId { sha256, length });
            }
            rows.push(PhysicalChunkRow {
                partition_id,
                domain,
                kind_id,
                total_history_bytes,
                history_versions_sha256,
                logical_key,
                payload,
                inline_history,
                continuation_pages,
            });
        }
        layers.push(rows);
    }
    if r.at != bytes.len() {
        return Err(corrupt());
    }
    Ok(PhysicalChunkImage { layers })
}

pub(crate) fn encode_continuation_page(page: &ContinuationPageImage) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(CONT_MAGIC);
    out.extend_from_slice(&page.partition_id.to_le_bytes());
    out.push(page.domain.tag());
    write_bytes(&mut out, &page.logical_key);
    out.extend_from_slice(&page.sequence.to_le_bytes());
    out.extend_from_slice(&(page.versions.len() as u32).to_le_bytes());
    for version in &page.versions {
        write_bytes(&mut out, version);
    }
    out
}

/// Zero-copy borrowed decoding of one continuation page, allocating ZERO owned
/// version buffers before `streaming_window_bytes` checks pass.
pub(crate) fn decode_borrowed_continuation_page(
    bytes: &[u8],
) -> Result<BorrowedContinuationPage<'_>, HistoricalReadDenial> {
    let mut r = Cursor { bytes, at: 0 };
    if r.take(5)? != CONT_MAGIC {
        return Err(corrupt());
    }
    let partition_id = r.u32()?;
    let domain = CanonicalKeyDomain::from_tag(r.u8()?).ok_or_else(corrupt)?;
    let logical_key = r.bytes_slice()?;
    let sequence = r.u32()?;
    let version_count = r.bounded_count(MIN_BYTES_FIELD)?;
    let versions_start = r.at;
    for _ in 0..version_count {
        let _ = r.bytes_slice()?;
    }
    if r.at != bytes.len() {
        return Err(corrupt());
    }
    Ok(BorrowedContinuationPage {
        partition_id,
        domain,
        logical_key,
        sequence,
        versions_raw: &bytes[versions_start..r.at],
        version_count,
    })
}

/// Iterates zero-copy length-prefixed byte slices encoded in `raw`.
pub(crate) fn visit_length_prefixed_slices<'a>(
    raw: &'a [u8],
    count: usize,
    mut f: impl FnMut(&'a [u8]) -> Result<(), HistoricalReadDenial>,
) -> Result<(), HistoricalReadDenial> {
    let mut r = Cursor { bytes: raw, at: 0 };
    for _ in 0..count {
        f(r.bytes_slice()?)?;
    }
    if r.at != raw.len() {
        return Err(corrupt());
    }
    Ok(())
}

/// Iterates zero-copy `ChunkMemberId` entries encoded in `raw`.
pub(crate) fn visit_continuation_ids(
    raw: &[u8],
    count: usize,
    mut f: impl FnMut(usize, ChunkMemberId) -> Result<(), HistoricalReadDenial>,
) -> Result<(), HistoricalReadDenial> {
    let mut r = Cursor { bytes: raw, at: 0 };
    for idx in 0..count {
        let sha256 = r.array_32()?;
        let length = r.u64()?;
        f(idx, ChunkMemberId { sha256, length })?;
    }
    if r.at != raw.len() {
        return Err(corrupt());
    }
    Ok(())
}

fn write_bytes(out: &mut Vec<u8>, bytes: &[u8]) {
    out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(bytes);
}

struct Cursor<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Cursor<'a> {
    fn remaining(&self) -> usize {
        self.bytes.len().saturating_sub(self.at)
    }

    fn bounded_count(&mut self, min_elem_bytes: usize) -> Result<usize, HistoricalReadDenial> {
        let count = self.u32()? as usize;
        if min_elem_bytes > 0 && count > self.remaining() / min_elem_bytes {
            return Err(corrupt());
        }
        Ok(count)
    }

    fn take(&mut self, len: usize) -> Result<&'a [u8], HistoricalReadDenial> {
        let end = self.at.checked_add(len).ok_or_else(corrupt)?;
        let slice = self.bytes.get(self.at..end).ok_or_else(corrupt)?;
        self.at = end;
        Ok(slice)
    }

    fn u8(&mut self) -> Result<u8, HistoricalReadDenial> {
        Ok(self.take(1)?[0])
    }

    fn u32(&mut self) -> Result<u32, HistoricalReadDenial> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().expect("4")))
    }

    fn u64(&mut self) -> Result<u64, HistoricalReadDenial> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().expect("8")))
    }

    fn array_32(&mut self) -> Result<[u8; 32], HistoricalReadDenial> {
        Ok(self.take(32)?.try_into().expect("32"))
    }

    fn bytes_slice(&mut self) -> Result<&'a [u8], HistoricalReadDenial> {
        let len = self.u32()? as usize;
        self.take(len)
    }
}
