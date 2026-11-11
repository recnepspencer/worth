use worth_foundational::facade::{AspectFieldLocator, AspectValue};
mod borrowed_allocation;
pub(crate) use borrowed_allocation::decoded_bytes as field_patch_decoded_bytes;

use super::{
    decode_aspect_field_locator, decode_aspect_value, encode_aspect_field_locator,
    encode_length_prefixed_aspect_value, encode_u32, AspectValueCanonicalCodecError,
};
use crate::transactions::data::{validate_planned_aspect_field_locator, AspectFieldPatch};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AspectFieldPatchCodecError {
    detail: String,
}

impl AspectFieldPatchCodecError {
    pub(crate) fn new(detail: impl Into<String>) -> Self {
        Self {
            detail: detail.into(),
        }
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl std::fmt::Display for AspectFieldPatchCodecError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.detail)
    }
}

impl std::error::Error for AspectFieldPatchCodecError {}

impl From<AspectValueCanonicalCodecError> for AspectFieldPatchCodecError {
    fn from(error: AspectValueCanonicalCodecError) -> Self {
        Self::new(error.detail())
    }
}

pub(crate) fn encode_aspect_field_patch_canonical_bytes(
    patch: &AspectFieldPatch,
) -> Result<Vec<u8>, AspectFieldPatchCodecError> {
    let mut bytes = Vec::new();
    encode_u32(&mut bytes, patch.len() as u32);
    for (locator, value) in patch.iter() {
        encode_length_prefixed_aspect_field_locator(&mut bytes, locator);
        encode_length_prefixed_aspect_value(&mut bytes, value);
    }
    Ok(bytes)
}

pub(crate) fn decode_aspect_field_patch_canonical_bytes(
    bytes: &[u8],
) -> Result<AspectFieldPatch, AspectFieldPatchCodecError> {
    let mut reader = AspectFieldPatchReader::new(bytes);
    let field_count = reader.read_u32()? as usize;
    // Every entry needs two length prefixes before its actual contents.
    // Refuse impossible nested counts before allocating the concrete roster.
    if field_count > reader.remaining() / 8 {
        return Err(AspectFieldPatchCodecError::new(
            "field count exceeds the binary frame",
        ));
    }
    let mut locators = Vec::with_capacity(field_count);
    for ordinal in 0..field_count {
        let locator = reader.read_locator()?;
        let value = reader.read_length_prefixed_aspect_value()?;
        locators.push((locator, value, ordinal));
    }
    reader.finish()?;
    // The former map retained the last value for each locator. Descending
    // original ordinal makes that same entry the first of an equal-key run.
    locators
        .sort_unstable_by(|left, right| left.0.cmp(&right.0).then_with(|| right.2.cmp(&left.2)));
    locators.dedup_by(|later, first| later.0 == first.0);
    let locators = locators
        .into_iter()
        .map(|(locator, value, _)| (locator, value))
        .collect();
    Ok(AspectFieldPatch::from_sorted_owned_entries(locators))
}

fn encode_length_prefixed_aspect_field_locator(bytes: &mut Vec<u8>, locator: &AspectFieldLocator) {
    let locator_bytes = encode_aspect_field_locator(locator);
    encode_u32(bytes, locator_bytes.len() as u32);
    bytes.extend_from_slice(&locator_bytes);
}

fn decode_planned_aspect_field_locator(
    bytes: &[u8],
) -> Result<AspectFieldLocator, AspectFieldPatchCodecError> {
    let locator = decode_aspect_field_locator(bytes).map_err(|error| {
        AspectFieldPatchCodecError::new(format!(
            "aspect field patch locator decode failed: {}",
            error
        ))
    })?;
    validate_planned_aspect_field_locator(locator)
}

struct AspectFieldPatchReader<'a> {
    bytes: &'a [u8],
    cursor: usize,
}

impl<'a> AspectFieldPatchReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, cursor: 0 }
    }

    fn finish(self) -> Result<(), AspectFieldPatchCodecError> {
        if self.cursor == self.bytes.len() {
            Ok(())
        } else {
            Err(AspectFieldPatchCodecError::new(format!(
                "aspect field patch codec left {} trailing bytes",
                self.bytes.len() - self.cursor
            )))
        }
    }

    fn remaining(&self) -> usize {
        self.bytes.len() - self.cursor
    }

    fn read_locator(&mut self) -> Result<AspectFieldLocator, AspectFieldPatchCodecError> {
        let length = self.read_u32()? as usize;
        let bytes = self.read_exact(length)?;
        decode_planned_aspect_field_locator(bytes)
    }

    fn read_u32(&mut self) -> Result<u32, AspectFieldPatchCodecError> {
        let mut buffer = [0_u8; 4];
        buffer.copy_from_slice(self.read_exact(4)?);
        Ok(u32::from_le_bytes(buffer))
    }

    fn read_length_prefixed_aspect_value(
        &mut self,
    ) -> Result<AspectValue, AspectFieldPatchCodecError> {
        let length = self.read_u32()? as usize;
        let bytes = self.read_exact(length)?;
        decode_aspect_value(bytes).map_err(AspectFieldPatchCodecError::from)
    }

    fn read_exact(&mut self, length: usize) -> Result<&'a [u8], AspectFieldPatchCodecError> {
        if self.cursor + length > self.bytes.len() {
            return Err(AspectFieldPatchCodecError::new(format!(
                "aspect field patch codec expected {length} bytes at offset {} but input ended early",
                self.cursor
            )));
        }
        let start = self.cursor;
        self.cursor += length;
        Ok(&self.bytes[start..self.cursor])
    }
}
