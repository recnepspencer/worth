//! Bounded, length-delimited cursors over one authored record family.
use super::{fields::Reader, BridgeAuthoredSectionDenial as D};

pub(super) struct Rows<'a> {
    input: Reader<'a>,
    remaining: usize,
}
impl<'a> Rows<'a> {
    pub(super) fn new(bytes: &'a [u8]) -> Result<Self, D> {
        let mut input = Reader::new(bytes);
        let remaining = usize::try_from(input.number()?).map_err(|_| D::SizeLimit)?;
        if remaining > bytes.len() / 8 {
            return Err(D::Malformed);
        }
        Ok(Self { input, remaining })
    }
    pub(super) fn len(&self) -> usize {
        self.remaining
    }
    pub(super) fn next(&mut self) -> Result<Option<&'a [u8]>, D> {
        if self.remaining == 0 {
            return Ok(None);
        }
        self.remaining -= 1;
        self.input.blob().map(Some)
    }
    pub(super) fn finish(self) -> Result<(), D> {
        if self.remaining != 0 {
            return Err(D::Malformed);
        }
        self.input.finish()
    }
}
