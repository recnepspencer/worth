use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, AspectLocator, AspectValue, CanonicalFieldPath, FieldKey,
    LocatorAuthority,
};

use crate::aspect_wire::{
    decode_aspect_value as decode_canonical_aspect_value, encode_length_prefixed_aspect_value,
    AspectValueCanonicalCodecError,
};
use crate::identity::data::{EntityId, PartitionId};
use crate::transactions::data::aspect_field_patch::AspectFieldPatch;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeCodecError {
    detail: String,
}

impl NativeCodecError {
    pub fn new(detail: impl Into<String>) -> Self {
        Self {
            detail: detail.into(),
        }
    }

    pub fn detail(&self) -> &str {
        &self.detail
    }
}

impl From<AspectValueCanonicalCodecError> for NativeCodecError {
    fn from(error: AspectValueCanonicalCodecError) -> Self {
        Self::new(error.detail())
    }
}

pub fn encode_entity_id(bytes: &mut Vec<u8>, entity_id: EntityId) {
    encode_u32(bytes, entity_id.partition_id.0);
    encode_u64(bytes, entity_id.local_slot.0);
    encode_u32(bytes, entity_id.generation.0);
}

pub fn decode_entity_id(reader: &mut NativeCodecReader<'_>) -> Result<EntityId, NativeCodecError> {
    Ok(EntityId::new(
        PartitionId(reader.read_u32()?),
        reader.read_u64()?,
        reader.read_u32()?,
    ))
}

pub fn encode_aspect_field_locator(bytes: &mut Vec<u8>, locator: &AspectFieldLocator) {
    encode_string(bytes, locator.aspect().aspect_key().as_str());
    encode_u32(bytes, locator.field_path().fields().len() as u32);
    for field in locator.field_path().fields() {
        encode_string(bytes, field.as_str());
    }
}

pub fn decode_aspect_field_locator(
    reader: &mut NativeCodecReader<'_>,
) -> Result<AspectFieldLocator, NativeCodecError> {
    let aspect_key = decode_aspect_key(reader)?;
    let field_count = reader.read_u32()? as usize;
    let mut fields = Vec::with_capacity(field_count);
    for _ in 0..field_count {
        fields.push(decode_field_key(reader)?);
    }
    let field_path = CanonicalFieldPath::new(fields).ok_or_else(|| {
        NativeCodecError::new("native codec aspect field locator had empty field path")
    })?;
    Ok(AspectFieldLocator::from_aspect(
        AspectLocator::new(LocatorAuthority::Planned, aspect_key),
        field_path,
    ))
}

pub fn encode_string(bytes: &mut Vec<u8>, value: &str) {
    encode_u32(bytes, value.len() as u32);
    bytes.extend_from_slice(value.as_bytes());
}

pub fn decode_string(reader: &mut NativeCodecReader<'_>) -> Result<String, NativeCodecError> {
    let length = reader.read_u32()? as usize;
    let bytes = reader.read_exact(length)?;
    std::str::from_utf8(bytes)
        .map(str::to_owned)
        .map_err(|error| {
            NativeCodecError::new(format!("invalid utf-8 string in native codec: {error}"))
        })
}

fn decode_aspect_key(reader: &mut NativeCodecReader<'_>) -> Result<AspectKey, NativeCodecError> {
    let value = decode_string(reader)?;
    AspectKey::new(value).ok_or_else(|| {
        NativeCodecError::new("native codec aspect field locator had invalid aspect key")
    })
}

fn decode_field_key(reader: &mut NativeCodecReader<'_>) -> Result<FieldKey, NativeCodecError> {
    let value = decode_string(reader)?;
    FieldKey::new(value).ok_or_else(|| {
        NativeCodecError::new("native codec aspect field locator had invalid field key")
    })
}

pub fn encode_aspect_field_patch(
    bytes: &mut Vec<u8>,
    patch: &AspectFieldPatch,
) -> Result<(), NativeCodecError> {
    let patch_bytes = patch.to_canonical_bytes().map_err(|error| {
        NativeCodecError::new(format!(
            "native codec could not encode aspect field patch: {}",
            error.detail()
        ))
    })?;
    encode_u32(bytes, patch_bytes.len() as u32);
    bytes.extend_from_slice(&patch_bytes);
    Ok(())
}

pub fn decode_aspect_field_patch(
    reader: &mut NativeCodecReader<'_>,
) -> Result<AspectFieldPatch, NativeCodecError> {
    let length = reader.read_u32()? as usize;
    let bytes = reader.read_exact(length)?;
    AspectFieldPatch::from_canonical_bytes(bytes).map_err(|error| {
        NativeCodecError::new(format!(
            "native codec could not decode aspect field patch: {}",
            error.detail()
        ))
    })
}

pub fn encode_aspect_value(bytes: &mut Vec<u8>, value: &AspectValue) {
    encode_length_prefixed_aspect_value(bytes, value);
}

pub fn decode_aspect_value(
    reader: &mut NativeCodecReader<'_>,
) -> Result<AspectValue, NativeCodecError> {
    let length = reader.read_u32()? as usize;
    let bytes = reader.read_exact(length)?;
    decode_canonical_aspect_value(bytes).map_err(NativeCodecError::from)
}

pub fn encode_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

pub fn encode_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

pub struct NativeCodecReader<'a> {
    bytes: &'a [u8],
    cursor: usize,
}

impl<'a> NativeCodecReader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, cursor: 0 }
    }

    pub fn finish(self) -> Result<(), NativeCodecError> {
        if self.cursor == self.bytes.len() {
            Ok(())
        } else {
            Err(NativeCodecError::new(format!(
                "native codec left {} trailing bytes",
                self.bytes.len() - self.cursor
            )))
        }
    }

    pub fn read_u8(&mut self) -> Result<u8, NativeCodecError> {
        Ok(self.read_exact(1)?[0])
    }

    pub fn read_bool(&mut self) -> Result<bool, NativeCodecError> {
        match self.read_u8()? {
            0 => Ok(false),
            1 => Ok(true),
            value => Err(NativeCodecError::new(format!(
                "invalid native codec bool tag {value}"
            ))),
        }
    }

    pub fn read_u32(&mut self) -> Result<u32, NativeCodecError> {
        let mut buffer = [0_u8; 4];
        buffer.copy_from_slice(self.read_exact(4)?);
        Ok(u32::from_le_bytes(buffer))
    }

    pub fn read_u64(&mut self) -> Result<u64, NativeCodecError> {
        let mut buffer = [0_u8; 8];
        buffer.copy_from_slice(self.read_exact(8)?);
        Ok(u64::from_le_bytes(buffer))
    }

    pub fn read_exact(&mut self, length: usize) -> Result<&'a [u8], NativeCodecError> {
        if self.cursor + length > self.bytes.len() {
            return Err(NativeCodecError::new(format!(
                "native codec expected {length} bytes at offset {} but input ended early",
                self.cursor
            )));
        }
        let start = self.cursor;
        self.cursor += length;
        Ok(&self.bytes[start..self.cursor])
    }
}
