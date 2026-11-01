//! Hash the exact authoritative payload without retaining encoded body bytes.
use super::CanonicalCommitEnvelope;
use sha2::{Digest, Sha256};
use std::io::{Result as IoResult, Write};

struct PayloadDigest {
    hash: Sha256,
    bytes: u64,
}
impl Write for PayloadDigest {
    fn write(&mut self, bytes: &[u8]) -> IoResult<usize> {
        self.bytes = self
            .bytes
            .checked_add(bytes.len() as u64)
            .ok_or_else(|| std::io::Error::other("canonical payload byte count overflow"))?;
        self.hash.update(bytes);
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
        self.authoritative_payload_observation()
            .map(|(_, digest)| digest)
    }
    pub(crate) fn authoritative_payload_observation(
        &self,
    ) -> Result<(u64, [u8; 32]), rmp_serde::encode::Error> {
        let mut digest = PayloadDigest {
            hash: Sha256::new(),
            bytes: 0,
        };
        self.write_authoritative_payload(&mut digest)?;
        Ok((digest.bytes, digest.hash.finalize().into()))
    }
}
