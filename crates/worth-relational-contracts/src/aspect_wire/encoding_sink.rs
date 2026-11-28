//! Shared canonical leaf emission, independent of its byte destination.
use std::convert::Infallible;

pub trait CanonicalAspectOutput {
    type Error;
    fn bytes(&mut self, bytes: &[u8]) -> Result<(), Self::Error>;
    fn length(&mut self, length: usize) -> Result<(), Self::Error>;
    fn u32_le(&mut self, value: u32) -> Result<(), Self::Error> {
        self.bytes(&value.to_le_bytes())
    }
    fn u64_le(&mut self, value: u64) -> Result<(), Self::Error> {
        self.bytes(&value.to_le_bytes())
    }
    fn i64_le(&mut self, value: i64) -> Result<(), Self::Error> {
        self.bytes(&value.to_le_bytes())
    }
    fn bytes_with_u64_length(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        self.u64_le(bytes.len() as u64)?;
        self.bytes(bytes)
    }
    fn u128_be(&mut self, value: u128) -> Result<(), Self::Error> {
        self.bytes(&value.to_be_bytes())
    }
    fn byte(&mut self, value: u8) -> Result<(), Self::Error> {
        self.bytes(&[value])
    }
    fn text(&mut self, value: &str) -> Result<(), Self::Error> {
        self.length(value.len())?;
        self.bytes(value.as_bytes())
    }
}

pub struct CollectCanonicalAspect<'bytes>(pub &'bytes mut Vec<u8>);
impl CanonicalAspectOutput for CollectCanonicalAspect<'_> {
    type Error = Infallible;
    fn bytes(&mut self, bytes: &[u8]) -> Result<(), Infallible> {
        self.0.extend_from_slice(bytes);
        Ok(())
    }
    fn length(&mut self, length: usize) -> Result<(), Infallible> {
        self.u32_le(length as u32)
    }
}

/// SHA-256 destination for the same canonical bytes written to a buffer.
#[derive(Default)]
pub struct CanonicalHasher(sha2::Sha256);

impl CanonicalHasher {
    pub fn finish(self) -> [u8; 32] {
        use sha2::Digest;
        self.0.finalize().into()
    }
}

impl CanonicalAspectOutput for CanonicalHasher {
    type Error = Infallible;

    fn bytes(&mut self, bytes: &[u8]) -> Result<(), Infallible> {
        use sha2::Digest;
        self.0.update(bytes);
        Ok(())
    }

    fn length(&mut self, length: usize) -> Result<(), Infallible> {
        self.u32_le(length as u32)
    }
}
