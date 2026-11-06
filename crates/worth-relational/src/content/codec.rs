use super::{RelationalContentControl as Control, RelationalContentDenial as Denial};
use std::any::Any;
use worth_foundational::facade::ContentFormat;

/// Query-free executable format meaning, installed by the Native schema owner.
/// Encoding correspondence must be independent of encoding and canonical parsing.
pub trait RelationalContentCodec: std::fmt::Debug + Send + Sync {
    fn format(&self) -> &ContentFormat;
    fn maximum_frame_bytes(&self) -> usize {
        super::MAXIMUM_CONTENT_FRAME_BYTES
    }
    fn payload_length(
        &self,
        input: &dyn Any,
        control: &mut Control<'_, '_, '_>,
    ) -> Result<usize, Denial>;
    fn encode(
        &self,
        input: &dyn Any,
        sink: &mut RelationalContentSink<'_>,
        control: &mut Control<'_, '_, '_>,
    ) -> Result<(), Denial>;
    fn validate(&self, payload: &[u8], control: &mut Control<'_, '_, '_>) -> Result<(), Denial>;
    fn verify_encoding(
        &self,
        input: &dyn Any,
        payload: &[u8],
        control: &mut Control<'_, '_, '_>,
    ) -> Result<(), Denial>;
}

/// One already funded payload; appending can neither grow nor escape its backing.
pub struct RelationalContentSink<'a> {
    pub(super) bytes: &'a mut Vec<u8>,
    pub(super) limit: usize,
}

impl RelationalContentSink<'_> {
    pub fn append(
        &mut self,
        bytes: &[u8],
        control: &mut Control<'_, '_, '_>,
    ) -> Result<(), Denial> {
        let required = self
            .bytes
            .len()
            .checked_add(bytes.len())
            .ok_or(Denial::ArithmeticOverflow)?;
        if required > self.limit {
            return Err(Denial::EncodingInconsistency);
        }
        for chunk in bytes.chunks(4096) {
            control.checkpoint(chunk.len() as u64)?;
            self.bytes.extend_from_slice(chunk);
        }
        Ok(())
    }
}
