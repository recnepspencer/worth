//! Shared canonical leaf emission, independent of its byte destination.
use std::convert::Infallible;

pub(crate) trait CanonicalAspectOutput {
    type Error;
    fn bytes(&mut self, bytes: &[u8]) -> Result<(), Self::Error>;
    fn length(&mut self, length: usize) -> Result<(), Self::Error>;
    fn byte(&mut self, value: u8) -> Result<(), Self::Error> {
        self.bytes(&[value])
    }
    fn text(&mut self, value: &str) -> Result<(), Self::Error> {
        self.length(value.len())?;
        self.bytes(value.as_bytes())
    }
}

pub(crate) struct CollectCanonicalAspect<'bytes>(pub(crate) &'bytes mut Vec<u8>);
impl CanonicalAspectOutput for CollectCanonicalAspect<'_> {
    type Error = Infallible;
    fn bytes(&mut self, bytes: &[u8]) -> Result<(), Infallible> {
        self.0.extend_from_slice(bytes);
        Ok(())
    }
    fn length(&mut self, length: usize) -> Result<(), Infallible> {
        self.bytes(&(length as u32).to_le_bytes())
    }
}
