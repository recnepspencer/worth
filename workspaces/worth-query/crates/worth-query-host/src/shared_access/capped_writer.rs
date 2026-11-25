//! Pre-allocation-capped binary encoder and request/row codecs enforcing frame ceilings.

use super::auth::RemoteAuthToken;
use super::binding::RemoteBinaryCodec;
use super::denial::{BudgetRefusalReason, ProtocolRefusalReason, SharedAccessDenial};
use super::framing::{BinaryReader, FrameKind, FRAME_HEADER_LEN, FRAME_MAGIC, FRAME_VERSION};
use super::types::{RemoteWatchId, MAX_SCOPE_KEY_BYTES};

/// Bounded binary wire writer that refuses BEFORE exceeding `max_bytes`.
pub struct RemoteByteWriter {
    buf: Vec<u8>,
    max_bytes: usize,
}

impl RemoteByteWriter {
    pub(super) fn new(max_bytes: u32) -> Self {
        let cap = (max_bytes as usize).min(256);
        Self {
            buf: Vec::with_capacity(cap),
            max_bytes: max_bytes as usize,
        }
    }

    /// Creates one writer capped to `max_frame_bytes` across the full wire frame
    /// (12-byte header + payload), reserving and charging the header up front.
    pub(super) fn new_for_frame(
        kind: FrameKind,
        max_frame_bytes: u32,
    ) -> Result<Self, SharedAccessDenial> {
        let mut writer = Self::new(max_frame_bytes);
        writer.charge(FRAME_HEADER_LEN)?;
        writer.buf.extend_from_slice(&FRAME_MAGIC);
        writer.buf.push(FRAME_VERSION);
        writer.buf.push(kind as u8);
        writer.buf.extend_from_slice(&0_u16.to_be_bytes());
        writer.buf.extend_from_slice(&0_u32.to_be_bytes());
        Ok(writer)
    }

    pub(super) fn finish_frame(mut self) -> Vec<u8> {
        let payload_len = u32::try_from(self.buf.len().saturating_sub(FRAME_HEADER_LEN))
            .expect("frame bounded below u32::MAX");
        self.buf[8..12].copy_from_slice(&payload_len.to_be_bytes());
        self.buf
    }

    pub const fn max_bytes(&self) -> usize {
        self.max_bytes
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    pub fn remaining(&self) -> usize {
        self.max_bytes.saturating_sub(self.buf.len())
    }

    /// Charges `additional` bytes against the frame ceiling BEFORE reserving memory.
    pub fn charge(&self, additional: usize) -> Result<(), SharedAccessDenial> {
        let total =
            self.buf
                .len()
                .checked_add(additional)
                .ok_or(SharedAccessDenial::BudgetRefused(
                    BudgetRefusalReason::ResponseTooLarge {
                        bytes: u32::MAX,
                        maximum: u32::try_from(self.max_bytes).unwrap_or(u32::MAX),
                    },
                ))?;
        if total > self.max_bytes {
            return Err(SharedAccessDenial::BudgetRefused(
                BudgetRefusalReason::ResponseTooLarge {
                    bytes: u32::try_from(total).unwrap_or(u32::MAX),
                    maximum: u32::try_from(self.max_bytes).unwrap_or(u32::MAX),
                },
            ));
        }
        Ok(())
    }

    pub fn write_u8(&mut self, value: u8) -> Result<(), SharedAccessDenial> {
        self.charge(1)?;
        self.buf.push(value);
        Ok(())
    }

    pub fn write_u32(&mut self, value: u32) -> Result<(), SharedAccessDenial> {
        self.charge(4)?;
        self.buf.extend_from_slice(&value.to_be_bytes());
        Ok(())
    }

    pub fn write_u64(&mut self, value: u64) -> Result<(), SharedAccessDenial> {
        self.charge(8)?;
        self.buf.extend_from_slice(&value.to_be_bytes());
        Ok(())
    }

    pub fn write_fixed(&mut self, bytes: &[u8]) -> Result<(), SharedAccessDenial> {
        self.charge(bytes.len())?;
        self.buf.extend_from_slice(bytes);
        Ok(())
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), SharedAccessDenial> {
        let needed = 4_usize
            .checked_add(bytes.len())
            .ok_or(SharedAccessDenial::BudgetRefused(
                BudgetRefusalReason::ResponseTooLarge {
                    bytes: u32::MAX,
                    maximum: u32::try_from(self.max_bytes).unwrap_or(u32::MAX),
                },
            ))?;
        self.charge(needed)?;
        let len = u32::try_from(bytes.len()).map_err(|_| {
            SharedAccessDenial::BudgetRefused(BudgetRefusalReason::ResponseTooLarge {
                bytes: u32::MAX,
                maximum: u32::try_from(self.max_bytes).unwrap_or(u32::MAX),
            })
        })?;
        self.buf.extend_from_slice(&len.to_be_bytes());
        self.buf.extend_from_slice(bytes);
        Ok(())
    }

    pub fn write_str(&mut self, text: &str) -> Result<(), SharedAccessDenial> {
        self.write_bytes(text.as_bytes())
    }

    /// Reserves a 4-byte length prefix, encodes a nested segment under the same
    /// ceiling, and backpatches its byte length without any intermediate buffer.
    pub(super) fn write_length_prefixed(
        &mut self,
        encode: impl FnOnce(&mut Self) -> Result<(), SharedAccessDenial>,
    ) -> Result<(), SharedAccessDenial> {
        self.charge(4)?;
        let len_pos = self.buf.len();
        self.buf.extend_from_slice(&[0_u8; 4]);
        let start = self.buf.len();
        encode(self)?;
        let written = self.buf.len() - start;
        let len_u32 = u32::try_from(written).map_err(|_| {
            SharedAccessDenial::BudgetRefused(BudgetRefusalReason::ResponseTooLarge {
                bytes: u32::MAX,
                maximum: u32::try_from(self.max_bytes).unwrap_or(u32::MAX),
            })
        })?;
        self.buf[len_pos..len_pos + 4].copy_from_slice(&len_u32.to_be_bytes());
        Ok(())
    }

    pub(super) fn into_bytes(self) -> Vec<u8> {
        self.buf
    }
}

fn map_request_budget(denial: SharedAccessDenial) -> SharedAccessDenial {
    match denial {
        SharedAccessDenial::BudgetRefused(BudgetRefusalReason::ResponseTooLarge { .. }) => {
            SharedAccessDenial::BudgetRefused(BudgetRefusalReason::RequestLimitExceeded)
        }
        other => other,
    }
}

/// Encodes the full scope request envelope (12-byte frame header, token, scope key,
/// length prefixes, and query body) through ONE capped `RemoteByteWriter`.
pub(super) fn encode_scope_request_frame<Query: RemoteBinaryCodec>(
    header: (FrameKind, u32),
    token: &RemoteAuthToken,
    scope_key: &str,
    query: &Query,
) -> Result<Vec<u8>, SharedAccessDenial> {
    let (kind, max_frame_bytes) = header;
    let mut writer =
        RemoteByteWriter::new_for_frame(kind, max_frame_bytes).map_err(map_request_budget)?;
    token
        .encode_into_capped(&mut writer)
        .map_err(map_request_budget)?;
    writer.write_str(scope_key).map_err(map_request_budget)?;
    writer
        .write_length_prefixed(|inner| query.encode_remote_bounded(inner))
        .map_err(map_request_budget)?;
    Ok(writer.finish_frame())
}

/// Borrows `scope_key` and `query_bytes` directly from `payload` with zero payload copy.
pub(super) fn decode_scope_request(
    payload: &[u8],
    max_frame_bytes: u32,
) -> Result<(RemoteAuthToken, &str, &[u8]), SharedAccessDenial> {
    let mut r = BinaryReader::new(payload);
    let token = RemoteAuthToken::decode_from(&mut r)?;
    let scope_key = r.read_str_ref_bounded(MAX_SCOPE_KEY_BYTES)?;
    let query_bytes = r.read_bytes_bounded(max_frame_bytes as usize)?;
    r.finish()?;
    Ok((token, scope_key, query_bytes))
}

/// Encodes a watch-id request frame (12-byte header, token, watch id) through ONE capped writer.
pub(super) fn encode_watch_id_request_frame(
    kind: FrameKind,
    token: &RemoteAuthToken,
    watch_id: RemoteWatchId,
    max_frame_bytes: u32,
) -> Result<Vec<u8>, SharedAccessDenial> {
    let mut writer =
        RemoteByteWriter::new_for_frame(kind, max_frame_bytes).map_err(map_request_budget)?;
    token
        .encode_into_capped(&mut writer)
        .map_err(map_request_budget)?;
    writer
        .write_u64(watch_id.as_u64())
        .map_err(map_request_budget)?;
    Ok(writer.finish_frame())
}

pub(super) fn decode_watch_id_request(
    payload: &[u8],
) -> Result<(RemoteAuthToken, RemoteWatchId), SharedAccessDenial> {
    let mut r = BinaryReader::new(payload);
    let token = RemoteAuthToken::decode_from(&mut r)?;
    let watch_id = RemoteWatchId::new(r.read_u64()?);
    r.finish()?;
    Ok((token, watch_id))
}

/// Streams typed rows directly into `writer`, failing closed before exceeding the frame ceiling.
pub(super) fn write_encoded_rows<Row: RemoteBinaryCodec>(
    writer: &mut RemoteByteWriter,
    row_count: usize,
    rows: impl IntoIterator<Item = Row>,
) -> Result<(), SharedAccessDenial> {
    let count_u32 = u32::try_from(row_count).map_err(|_| {
        SharedAccessDenial::BudgetRefused(BudgetRefusalReason::ResponseTooLarge {
            bytes: u32::MAX,
            maximum: u32::try_from(writer.max_bytes()).unwrap_or(u32::MAX),
        })
    })?;
    let min_needed = 4_usize.saturating_add(row_count.saturating_mul(4));
    writer.charge(min_needed)?;
    writer.write_u32(count_u32)?;
    for row in rows {
        writer.write_length_prefixed(|inner| row.encode_remote_bounded(inner))?;
    }
    Ok(())
}

pub(super) fn decode_rows_payload(
    reader: &mut BinaryReader<'_>,
    max_frame_bytes: u32,
) -> Result<Vec<Vec<u8>>, SharedAccessDenial> {
    let count = reader.read_u32()? as usize;
    if count.saturating_mul(4) > reader.remaining() {
        return Err(SharedAccessDenial::ProtocolRefused(
            ProtocolRefusalReason::MalformedPayload,
        ));
    }
    let mut rows = Vec::with_capacity(count);
    for _ in 0..count {
        let row = reader
            .read_bytes_bounded(max_frame_bytes as usize)?
            .to_vec();
        rows.push(row);
    }
    Ok(rows)
}

pub(super) fn decode_refusal_payload(
    payload: &[u8],
) -> Result<SharedAccessDenial, SharedAccessDenial> {
    let mut r = BinaryReader::new(payload);
    let code = r.read_u16()?;
    let a = r.read_u32()?;
    let b = r.read_u32()?;
    r.finish()?;
    Ok(SharedAccessDenial::from_wire_triple(code, a, b))
}
