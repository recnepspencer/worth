use sha2::{Digest, Sha256};

/// External byte identity of an encoded checkpoint stream. This is not a
/// self-persisted integrity claim; record CRCs and selective aggregates govern
/// admission separately.
pub fn checkpoint_stream_encoded_digest(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}
