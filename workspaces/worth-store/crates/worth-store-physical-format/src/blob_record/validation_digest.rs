use sha2::{Digest, Sha256};

/// SHA-256 mechanism for C.11 blob record validation evidence outside persisted
/// fields: the complete inspected frame and the exact-scope preimage that the
/// runtime integrity owner constructs. The format fixes the algorithm; the
/// caller owns which bytes are evidence.
pub fn blob_record_v1_validation_digest(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}
