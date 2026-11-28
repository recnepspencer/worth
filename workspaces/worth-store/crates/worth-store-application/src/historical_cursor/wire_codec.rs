//! Low-level canonical base64url, constant-time HMAC-SHA256, and bounded byte
//! reader for [`super::RetainedBasisToken`].

use sha2::{Digest, Sha256};
use worth_runtime_world::facade::RetainedBasisTokenDecodeError;

pub(super) const MAX_WIRE_DECODED_BYTES: usize = 8192;
/// Maximum legal unpadded base64url length for [`MAX_WIRE_DECODED_BYTES`].
pub(super) const MAX_WIRE_BASE64_LEN: usize = MAX_WIRE_DECODED_BYTES.div_ceil(3) * 4;

pub(super) struct ByteReader<'a> {
    pub(super) bytes: &'a [u8],
    pub(super) at: usize,
}

impl<'a> ByteReader<'a> {
    pub(super) fn take(&mut self, count: usize) -> Result<&'a [u8], RetainedBasisTokenDecodeError> {
        let end = self
            .at
            .checked_add(count)
            .ok_or(RetainedBasisTokenDecodeError::Truncated)?;
        let slice = self
            .bytes
            .get(self.at..end)
            .ok_or(RetainedBasisTokenDecodeError::Truncated)?;
        self.at = end;
        Ok(slice)
    }

    pub(super) fn u8(&mut self) -> Result<u8, RetainedBasisTokenDecodeError> {
        Ok(self.take(1)?[0])
    }

    pub(super) fn u16(&mut self) -> Result<u16, RetainedBasisTokenDecodeError> {
        let b = self.take(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }

    pub(super) fn u32(&mut self) -> Result<u32, RetainedBasisTokenDecodeError> {
        let b = self.take(4)?;
        Ok(u32::from_le_bytes(b.try_into().expect("4 bytes")))
    }

    pub(super) fn u64(&mut self) -> Result<u64, RetainedBasisTokenDecodeError> {
        let b = self.take(8)?;
        Ok(u64::from_le_bytes(b.try_into().expect("8 bytes")))
    }

    pub(super) fn array_16(&mut self) -> Result<[u8; 16], RetainedBasisTokenDecodeError> {
        Ok(self.take(16)?.try_into().expect("16 bytes"))
    }

    pub(super) fn array_32(&mut self) -> Result<[u8; 32], RetainedBasisTokenDecodeError> {
        Ok(self.take(32)?.try_into().expect("32 bytes"))
    }

    pub(super) fn utf8(&mut self, max: usize) -> Result<String, RetainedBasisTokenDecodeError> {
        let len = usize::from(self.u16()?);
        if len == 0 || len > max {
            return Err(RetainedBasisTokenDecodeError::MalformedField);
        }
        let raw = self.take(len)?;
        std::str::from_utf8(raw)
            .map(str::to_owned)
            .map_err(|_| RetainedBasisTokenDecodeError::MalformedField)
    }
}

/// RFC 2104 HMAC-SHA256 keyed by a Home-owned 32-byte secret key.
pub(super) fn hmac_sha256(key: &[u8; 32], domain: &[u8], body: &[u8]) -> [u8; 32] {
    let mut ipad = [0x36u8; 64];
    let mut opad = [0x5Cu8; 64];
    for i in 0..32 {
        ipad[i] ^= key[i];
        opad[i] ^= key[i];
    }
    let mut inner = Sha256::new();
    inner.update(ipad);
    inner.update(domain);
    inner.update(body);
    let inner_digest: [u8; 32] = inner.finalize().into();

    let mut outer = Sha256::new();
    outer.update(opad);
    outer.update(inner_digest);
    outer.finalize().into()
}

/// Constant-time 32-byte comparison for MAC verification.
pub(super) fn ct_eq_32(a: &[u8; 32], b: &[u8; 32]) -> bool {
    let mut diff = 0u8;
    for i in 0..32 {
        diff |= a[i] ^ b[i];
    }
    diff == 0
}

const B64URL: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

pub(super) fn encode_base64url(input: &[u8], out: &mut String) {
    let mut chunks = input.chunks_exact(3);
    for chunk in &mut chunks {
        let n = (u32::from(chunk[0]) << 16) | (u32::from(chunk[1]) << 8) | u32::from(chunk[2]);
        out.push(B64URL[((n >> 18) & 0x3F) as usize] as char);
        out.push(B64URL[((n >> 12) & 0x3F) as usize] as char);
        out.push(B64URL[((n >> 6) & 0x3F) as usize] as char);
        out.push(B64URL[(n & 0x3F) as usize] as char);
    }
    match chunks.remainder() {
        [a] => {
            let n = u32::from(*a) << 16;
            out.push(B64URL[((n >> 18) & 0x3F) as usize] as char);
            out.push(B64URL[((n >> 12) & 0x3F) as usize] as char);
        }
        [a, b] => {
            let n = (u32::from(*a) << 16) | (u32::from(*b) << 8);
            out.push(B64URL[((n >> 18) & 0x3F) as usize] as char);
            out.push(B64URL[((n >> 12) & 0x3F) as usize] as char);
            out.push(B64URL[((n >> 6) & 0x3F) as usize] as char);
        }
        _ => {}
    }
}

fn decode_b64_char(byte: u8) -> Option<u32> {
    match byte {
        b'A'..=b'Z' => Some(u32::from(byte - b'A')),
        b'a'..=b'z' => Some(u32::from(byte - b'a') + 26),
        b'0'..=b'9' => Some(u32::from(byte - b'0') + 52),
        b'-' => Some(62),
        b'_' => Some(63),
        _ => None,
    }
}

/// Decodes canonical unpadded base64url, rejecting oversized inputs BEFORE
/// allocating any output buffer.
pub(super) fn decode_base64url(input: &str) -> Result<Vec<u8>, RetainedBasisTokenDecodeError> {
    use RetainedBasisTokenDecodeError as ErrKind;
    let bytes = input.as_bytes();
    if bytes.len() > MAX_WIRE_BASE64_LEN {
        return Err(ErrKind::MalformedField);
    }
    if bytes.is_empty() || bytes.len() % 4 == 1 {
        return Err(ErrKind::InvalidEncoding);
    }
    let decoded_len = (bytes.len() / 4) * 3
        + match bytes.len() % 4 {
            0 => 0,
            2 => 1,
            3 => 2,
            _ => unreachable!(),
        };
    if decoded_len > MAX_WIRE_DECODED_BYTES {
        return Err(ErrKind::MalformedField);
    }
    let mut out = Vec::with_capacity(decoded_len);
    let mut chunks = bytes.chunks_exact(4);
    for chunk in &mut chunks {
        let a = decode_b64_char(chunk[0]).ok_or(ErrKind::InvalidEncoding)?;
        let b = decode_b64_char(chunk[1]).ok_or(ErrKind::InvalidEncoding)?;
        let c = decode_b64_char(chunk[2]).ok_or(ErrKind::InvalidEncoding)?;
        let d = decode_b64_char(chunk[3]).ok_or(ErrKind::InvalidEncoding)?;
        let n = (a << 18) | (b << 12) | (c << 6) | d;
        out.push(((n >> 16) & 0xFF) as u8);
        out.push(((n >> 8) & 0xFF) as u8);
        out.push((n & 0xFF) as u8);
    }
    match chunks.remainder() {
        [] => {}
        [c0, c1] => {
            let a = decode_b64_char(*c0).ok_or(ErrKind::InvalidEncoding)?;
            let b = decode_b64_char(*c1).ok_or(ErrKind::InvalidEncoding)?;
            if (b & 0x0F) != 0 {
                return Err(ErrKind::InvalidEncoding);
            }
            out.push(((a << 2) | (b >> 4)) as u8);
        }
        [c0, c1, c2] => {
            let a = decode_b64_char(*c0).ok_or(ErrKind::InvalidEncoding)?;
            let b = decode_b64_char(*c1).ok_or(ErrKind::InvalidEncoding)?;
            let c = decode_b64_char(*c2).ok_or(ErrKind::InvalidEncoding)?;
            if (c & 0x03) != 0 {
                return Err(ErrKind::InvalidEncoding);
            }
            let n = (a << 18) | (b << 12) | (c << 6);
            out.push(((n >> 16) & 0xFF) as u8);
            out.push(((n >> 8) & 0xFF) as u8);
        }
        _ => return Err(ErrKind::InvalidEncoding),
    }
    Ok(out)
}
