//! Hash the exact authoritative payload without retaining encoded body bytes.
use super::CanonicalCommitEnvelope;
use sha2::{Digest, Sha256};
use std::io::{Result as IoResult, Write};

struct PayloadDigest(Sha256);
impl Write for PayloadDigest {
    fn write(&mut self, bytes: &[u8]) -> IoResult<usize> {
        self.0.update(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> IoResult<()> {
        Ok(())
    }
}
impl CanonicalCommitEnvelope {
    pub(crate) fn authoritative_payload_digest(
        &self,
    ) -> Result<[u8; 32], rmp_serde::encode::Error> {
        let mut digest = PayloadDigest(Sha256::new());
        self.write_authoritative_payload(&mut digest)?;
        Ok(digest.0.finalize().into())
    }
}
