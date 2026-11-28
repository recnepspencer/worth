//! Incremental SHA-256 over byte domains defined by the caller.
use sha2::{Digest, Sha256};

#[derive(Default)]
pub struct Sha256Digest(Sha256);

impl Sha256Digest {
    pub fn update(&mut self, bytes: &[u8]) {
        self.0.update(bytes);
    }

    pub fn finish(self) -> [u8; 32] {
        self.0.finalize().into()
    }
}
