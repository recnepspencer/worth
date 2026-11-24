//! Deterministic length-prefixed binary wire framing with pre-allocation bounds.

use std::io::{Read, Write};

use super::auth::RemoteAuthToken;
use super::denial::{ProtocolRefusalReason, SharedAccessDenial};
use super::types::{RemoteWatchId, MAX_SCOPE_KEY_BYTES};

pub(super) const FRAME_MAGIC: [u8; 4] = *b"WQS1";
pub(super) const FRAME_VERSION: u8 = 1;
pub(super) const FRAME_HEADER_LEN: usize = 12;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub(super) enum FrameKind {
    QueryRequest = 0x01,
    WatchOpenRequest = 0x02,
    WatchPollRequest = 0x03,
    WatchCloseRequest = 0x04,
    QueryResponse = 0x81,
    WatchOpenResponse = 0x82,
    WatchPollResponse = 0x83,
    WatchCloseResponse = 0x84,
    RefusalResponse = 0xFF,
}

impl FrameKind {
    const fn from_wire(byte: u8) -> Option<Self> {
        match byte {
            0x01 => Some(Self::QueryRequest),
            0x02 => Some(Self::WatchOpenRequest),
            0x03 => Some(Self::WatchPollRequest),
            0x04 => Some(Self::WatchCloseRequest),
            0x81 => Some(Self::QueryResponse),
            0x82 => Some(Self::WatchOpenResponse),
            0x83 => Some(Self::WatchPollResponse),
            0x84 => Some(Self::WatchCloseResponse),
            0xFF => Some(Self::RefusalResponse),
            _ => None,
        }
    }
}

/// Validates a 12-byte header on the stack BEFORE any heap allocation occurs.
pub(super) fn parse_header(
    header: &[u8; FRAME_HEADER_LEN],
    max_frame_bytes: u32,
) -> Result<(FrameKind, u32), SharedAccessDenial> {
    if header[0..4] != FRAME_MAGIC {
        return Err(SharedAccessDenial::ProtocolRefused(
            ProtocolRefusalReason::InvalidMagic,
        ));
    }
    if header[4] != FRAME_VERSION {
        return Err(SharedAccessDenial::ProtocolRefused(
            ProtocolRefusalReason::UnsupportedVersion,
        ));
    }
    let kind = FrameKind::from_wire(header[5]).ok_or(SharedAccessDenial::ProtocolRefused(
        ProtocolRefusalReason::UnknownMessageKind,
    ))?;
    let flags = u16::from_be_bytes([header[6], header[7]]);
    if flags != 0 {
        return Err(SharedAccessDenial::ProtocolRefused(
            ProtocolRefusalReason::InvalidFlags,
        ));
    }
    let payload_len = u32::from_be_bytes([header[8], header[9], header[10], header[11]]);
    if payload_len > max_frame_bytes {
        return Err(SharedAccessDenial::ProtocolRefused(
            ProtocolRefusalReason::FrameTooLarge {
                requested: payload_len,
                maximum: max_frame_bytes,
            },
        ));
    }
    Ok((kind, payload_len))
}

pub(super) fn encode_frame(
    kind: FrameKind,
    payload: &[u8],
    max_frame_bytes: u32,
) -> Result<Vec<u8>, SharedAccessDenial> {
    let payload_len = u32::try_from(payload.len()).map_err(|_| {
        SharedAccessDenial::ProtocolRefused(ProtocolRefusalReason::FrameTooLarge {
            requested: u32::MAX,
            maximum: max_frame_bytes,
        })
    })?;
    if payload_len > max_frame_bytes {
        return Err(SharedAccessDenial::ProtocolRefused(
            ProtocolRefusalReason::FrameTooLarge {
                requested: payload_len,
                maximum: max_frame_bytes,
            },
        ));
    }
    let mut out = Vec::with_capacity(FRAME_HEADER_LEN + payload.len());
    out.extend_from_slice(&FRAME_MAGIC);
    out.push(FRAME_VERSION);
    out.push(kind as u8);
    out.extend_from_slice(&0_u16.to_be_bytes());
    out.extend_from_slice(&payload_len.to_be_bytes());
    out.extend_from_slice(payload);
    Ok(out)
}

pub(super) fn decode_frame_slice(
    bytes: &[u8],
    max_frame_bytes: u32,
) -> Result<(FrameKind, &[u8]), SharedAccessDenial> {
    let header: [u8; FRAME_HEADER_LEN] = bytes
        .get(..FRAME_HEADER_LEN)
        .and_then(|slice| slice.try_into().ok())
        .ok_or(SharedAccessDenial::ProtocolRefused(
            ProtocolRefusalReason::MalformedPayload,
        ))?;
    let (kind, payload_len) = parse_header(&header, max_frame_bytes)?;
    let expected_total = FRAME_HEADER_LEN.checked_add(payload_len as usize).ok_or(
        SharedAccessDenial::ProtocolRefused(ProtocolRefusalReason::MalformedPayload),
    )?;
    if bytes.len() < expected_total {
        return Err(SharedAccessDenial::ProtocolRefused(
            ProtocolRefusalReason::MalformedPayload,
        ));
    }
    if bytes.len() > expected_total {
        return Err(SharedAccessDenial::ProtocolRefused(
            ProtocolRefusalReason::TrailingBytes,
        ));
    }
    Ok((kind, &bytes[FRAME_HEADER_LEN..expected_total]))
}

pub(super) fn read_bounded_frame(
    reader: &mut impl Read,
    max_frame_bytes: u32,
) -> Result<(FrameKind, Vec<u8>), SharedAccessDenial> {
    let mut header = [0_u8; FRAME_HEADER_LEN];
    reader
        .read_exact(&mut header)
        .map_err(|err| SharedAccessDenial::TransportIo(err.kind().to_string()))?;
    let (kind, payload_len) = parse_header(&header, max_frame_bytes)?;
    // Allocation happens strictly AFTER header and length-bound validation.
    let mut payload = vec![0_u8; payload_len as usize];
    reader
        .read_exact(&mut payload)
        .map_err(|err| SharedAccessDenial::TransportIo(err.kind().to_string()))?;
    Ok((kind, payload))
}

pub(super) fn write_bounded_frame(
    writer: &mut impl Write,
    kind: FrameKind,
    payload: &[u8],
    max_frame_bytes: u32,
) -> Result<(), SharedAccessDenial> {
    let frame = encode_frame(kind, payload, max_frame_bytes)?;
    writer
        .write_all(&frame)
        .map_err(|err| SharedAccessDenial::TransportIo(err.kind().to_string()))?;
    writer
        .flush()
        .map_err(|err| SharedAccessDenial::TransportIo(err.kind().to_string()))
}

pub(super) struct BinaryWriter {
    buf: Vec<u8>,
}

impl BinaryWriter {
    pub(super) fn with_capacity(cap: usize) -> Self {
        Self {
            buf: Vec::with_capacity(cap),
        }
    }

    pub(super) fn write_u8(&mut self, v: u8) {
        self.buf.push(v);
    }

    pub(super) fn write_u16(&mut self, v: u16) {
        self.buf.extend_from_slice(&v.to_be_bytes());
    }

    pub(super) fn write_u32(&mut self, v: u32) {
        self.buf.extend_from_slice(&v.to_be_bytes());
    }

    pub(super) fn write_u64(&mut self, v: u64) {
        self.buf.extend_from_slice(&v.to_be_bytes());
    }

    pub(super) fn write_fixed(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
    }

    pub(super) fn write_bytes(&mut self, bytes: &[u8]) {
        let len = u32::try_from(bytes.len()).expect("frame payload bounded below u32::MAX");
        self.write_u32(len);
        self.buf.extend_from_slice(bytes);
    }

    pub(super) fn write_str(&mut self, text: &str) {
        self.write_bytes(text.as_bytes());
    }

    pub(super) fn into_bytes(self) -> Vec<u8> {
        self.buf
    }
}

pub(super) struct BinaryReader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> BinaryReader<'a> {
    pub(super) const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }

    pub(super) const fn remaining(&self) -> usize {
        self.bytes.len().saturating_sub(self.pos)
    }

    pub(super) fn finish(self) -> Result<(), SharedAccessDenial> {
        if self.remaining() == 0 {
            Ok(())
        } else {
            Err(SharedAccessDenial::ProtocolRefused(
                ProtocolRefusalReason::TrailingBytes,
            ))
        }
    }

    pub(super) fn read_u8(&mut self) -> Result<u8, SharedAccessDenial> {
        let b = *self
            .bytes
            .get(self.pos)
            .ok_or(SharedAccessDenial::ProtocolRefused(
                ProtocolRefusalReason::MalformedPayload,
            ))?;
        self.pos += 1;
        Ok(b)
    }

    pub(super) fn read_u16(&mut self) -> Result<u16, SharedAccessDenial> {
        Ok(u16::from_be_bytes(self.read_fixed::<2>()?))
    }

    pub(super) fn read_u32(&mut self) -> Result<u32, SharedAccessDenial> {
        Ok(u32::from_be_bytes(self.read_fixed::<4>()?))
    }

    pub(super) fn read_u64(&mut self) -> Result<u64, SharedAccessDenial> {
        Ok(u64::from_be_bytes(self.read_fixed::<8>()?))
    }

    pub(super) fn read_fixed<const N: usize>(&mut self) -> Result<[u8; N], SharedAccessDenial> {
        let end = self
            .pos
            .checked_add(N)
            .ok_or(SharedAccessDenial::ProtocolRefused(
                ProtocolRefusalReason::MalformedPayload,
            ))?;
        let slice = self
            .bytes
            .get(self.pos..end)
            .ok_or(SharedAccessDenial::ProtocolRefused(
                ProtocolRefusalReason::MalformedPayload,
            ))?;
        self.pos = end;
        let mut out = [0_u8; N];
        out.copy_from_slice(slice);
        Ok(out)
    }

    pub(super) fn read_bytes_bounded(
        &mut self,
        max_len: usize,
    ) -> Result<&'a [u8], SharedAccessDenial> {
        let len = self.read_u32()? as usize;
        if len > max_len || len > self.remaining() {
            return Err(SharedAccessDenial::ProtocolRefused(
                ProtocolRefusalReason::MalformedPayload,
            ));
        }
        let end = self.pos + len;
        let slice = &self.bytes[self.pos..end];
        self.pos = end;
        Ok(slice)
    }

    pub(super) fn read_str_bounded(
        &mut self,
        max_len: usize,
    ) -> Result<String, SharedAccessDenial> {
        let slice = self.read_bytes_bounded(max_len)?;
        std::str::from_utf8(slice).map(str::to_owned).map_err(|_| {
            SharedAccessDenial::ProtocolRefused(ProtocolRefusalReason::MalformedPayload)
        })
    }
}

pub(super) fn encode_scope_request(
    token: &RemoteAuthToken,
    scope_key: &str,
    query_bytes: &[u8],
) -> Vec<u8> {
    let mut w = BinaryWriter::with_capacity(256 + scope_key.len() + query_bytes.len());
    token.encode_into(&mut w);
    w.write_str(scope_key);
    w.write_bytes(query_bytes);
    w.into_bytes()
}

pub(super) fn decode_scope_request(
    payload: &[u8],
    max_frame_bytes: u32,
) -> Result<(RemoteAuthToken, String, Vec<u8>), SharedAccessDenial> {
    let mut r = BinaryReader::new(payload);
    let token = RemoteAuthToken::decode_from(&mut r)?;
    let scope_key = r.read_str_bounded(MAX_SCOPE_KEY_BYTES)?;
    let query_bytes = r.read_bytes_bounded(max_frame_bytes as usize)?.to_vec();
    r.finish()?;
    Ok((token, scope_key, query_bytes))
}

pub(super) fn encode_watch_id_request(token: &RemoteAuthToken, watch_id: RemoteWatchId) -> Vec<u8> {
    let mut w = BinaryWriter::with_capacity(256);
    token.encode_into(&mut w);
    w.write_u64(watch_id.as_u64());
    w.into_bytes()
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

pub(super) fn encode_rows_payload(rows: &[Vec<u8>]) -> Vec<u8> {
    let approx = rows
        .iter()
        .fold(4_usize, |acc, r| acc.saturating_add(4 + r.len()));
    let mut w = BinaryWriter::with_capacity(approx);
    let count = u32::try_from(rows.len()).expect("row count bounded below u32::MAX");
    w.write_u32(count);
    for row in rows {
        w.write_bytes(row);
    }
    w.into_bytes()
}

pub(super) fn decode_rows_payload(
    reader: &mut BinaryReader<'_>,
    max_frame_bytes: u32,
) -> Result<Vec<Vec<u8>>, SharedAccessDenial> {
    let count = reader.read_u32()? as usize;
    // Preflight minimum possible byte footprint (4 bytes per row length prefix) before allocating.
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

pub(super) fn encode_refusal_payload(denial: &SharedAccessDenial) -> Vec<u8> {
    let (code, a, b) = denial.to_wire_triple();
    let mut w = BinaryWriter::with_capacity(10);
    w.write_u16(code);
    w.write_u32(a);
    w.write_u32(b);
    w.into_bytes()
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
