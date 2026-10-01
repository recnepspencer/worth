use sha2::{Digest, Sha256};

/// An observed content digest is meaningful only when the occurrence and
/// stored-content framing remain independently readable after inner damage.
pub(super) fn readable_chunk_digest(
    frame: &[u8],
    store: [u8; 16],
    session: [u8; 16],
    ordinal: u64,
) -> Option<[u8; 32]> {
    if frame.len() < 140
        || &frame[..8] != b"WRC11BLB"
        || frame[8] != 2
        || frame[48..64] != store[..]
        || frame[64..80] != session[..]
        || u64::from_le_bytes(frame[80..88].try_into().ok()?) != ordinal
        || 48 + u32::from_le_bytes(frame[12..16].try_into().ok()?) as usize != frame.len()
    {
        return None;
    }
    let content = &frame[128..];
    let length = u32::from_le_bytes(content[8..12].try_into().ok()?) as usize;
    if content[..4] != [1, 0, 0, 0] || content.len() != 12 + length {
        return None;
    }
    Some(Sha256::digest(content).into())
}
