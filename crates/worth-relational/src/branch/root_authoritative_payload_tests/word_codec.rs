//! A single exact signed word; no storage, hashing or root authority lives here.
use crate::content::*;
use std::any::Any;
use worth_foundational::facade::ContentFormat;

#[derive(Debug)]
pub(super) struct WordCodec(pub(super) ContentFormat);
impl RelationalContentCodec for WordCodec {
    fn format(&self) -> &ContentFormat {
        &self.0
    }
    fn payload_length(
        &self,
        input: &dyn Any,
        control: &mut RelationalContentControl<'_, '_, '_>,
    ) -> Result<usize, RelationalContentDenial> {
        control.checkpoint(1)?;
        if input.is::<i64>() {
            Ok(8)
        } else {
            Err(RelationalContentDenial::InputTypeMismatch)
        }
    }
    fn encode(
        &self,
        input: &dyn Any,
        sink: &mut RelationalContentSink<'_>,
        control: &mut RelationalContentControl<'_, '_, '_>,
    ) -> Result<(), RelationalContentDenial> {
        let word = input
            .downcast_ref::<i64>()
            .ok_or(RelationalContentDenial::InputTypeMismatch)?;
        sink.append(&word.to_le_bytes(), control)
    }
    fn validate(
        &self,
        bytes: &[u8],
        control: &mut RelationalContentControl<'_, '_, '_>,
    ) -> Result<(), RelationalContentDenial> {
        control.checkpoint(8)?;
        if bytes.len() == 8 {
            Ok(())
        } else {
            Err(RelationalContentDenial::NonCanonicalContent)
        }
    }
    fn verify_encoding(
        &self,
        input: &dyn Any,
        bytes: &[u8],
        control: &mut RelationalContentControl<'_, '_, '_>,
    ) -> Result<(), RelationalContentDenial> {
        control.checkpoint(8)?;
        let expected = input
            .downcast_ref::<i64>()
            .ok_or(RelationalContentDenial::InputTypeMismatch)?;
        let actual = i64::from_le_bytes(
            bytes
                .try_into()
                .map_err(|_| RelationalContentDenial::EncodingInconsistency)?,
        );
        if actual == *expected {
            Ok(())
        } else {
            Err(RelationalContentDenial::EncodingInconsistency)
        }
    }
}
