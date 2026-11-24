//! Deterministic length-prefixed binary wire framing with pre-allocation bounds.

use std::io::{Read, Write};

use super::denial::{ProtocolRefusalReason, SharedAccessDenial};

pub(super) const FRAME_MAGIC: [u8; 4] = *b"WQS1";
pub(super) const FRAME_VERSION: u8 = 1;
pub(super) const FRAME_HEADER_LEN: usize = 12;
pub(super) const REFUSAL_PAYLOAD_LEN: usize = 10;
pub(super) const REFUSAL_FRAME_LEN: usize = FRAME_HEADER_LEN + REFUSAL_PAYLOAD_LEN;

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

/// Stack-encodes a refusal frame with zero heap allocation and zero panic paths.
pub(super) fn encode_refusal_frame(denial: &SharedAccessDenial) -> [u8; REFUSAL_FRAME_LEN] {
    let (code, a, b) = denial.to_wire_triple();
    let mut frame = [0_u8; REFUSAL_FRAME_LEN];
    frame[0..4].copy_from_slice(&FRAME_MAGIC);
    frame[4] = FRAME_VERSION;
    frame[5] = FrameKind::RefusalResponse as u8;
    frame[6..8].copy_from_slice(&0_u16.to_be_bytes());
    frame[8..12].copy_from_slice(&(REFUSAL_PAYLOAD_LEN as u32).to_be_bytes());
    frame[12..14].copy_from_slice(&code.to_be_bytes());
    frame[14..18].copy_from_slice(&a.to_be_bytes());
    frame[18..22].copy_from_slice(&b.to_be_bytes());
    frame
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

    pub(super) fn read_str_ref_bounded(
        &mut self,
        max_len: usize,
    ) -> Result<&'a str, SharedAccessDenial> {
        let slice = self.read_bytes_bounded(max_len)?;
        std::str::from_utf8(slice).map_err(|_| {
            SharedAccessDenial::ProtocolRefused(ProtocolRefusalReason::MalformedPayload)
        })
    }

    pub(super) fn read_str_bounded(
        &mut self,
        max_len: usize,
    ) -> Result<String, SharedAccessDenial> {
        self.read_str_ref_bounded(max_len).map(str::to_owned)
    }
}
