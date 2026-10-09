use sha2::{Digest, Sha256};

/// Incremental SHA-256 of an extent payload assembled chunk by chunk in
/// logical order, so a reader never holds the whole payload to digest it.
#[derive(Debug, Clone, Default)]
pub struct ExtentPayloadDigestCalculator {
    digest: Sha256,
}

impl ExtentPayloadDigestCalculator {
    pub fn update(&mut self, chunk_payload: &[u8]) {
        self.digest.update(chunk_payload);
    }

    pub fn finish(self) -> [u8; 32] {
        self.digest.finalize().into()
    }
}

/// The digest `ExtentPayloadDigestCalculator` produces for a complete payload.
pub fn extent_payload_digest(payload: &[u8]) -> [u8; 32] {
    Sha256::digest(payload).into()
}
