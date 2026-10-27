use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, AspectLocator, AspectValueLocator, CanonicalFieldPath, FieldKey,
    LocatorAuthority,
};

use super::error::AspectValueLocatorCanonicalCodecError;
use super::tags;
#[cfg(test)]
use worth_foundational::facade::BoundarySourceLocator;

pub(crate) fn decode_aspect_value_locator(
    bytes: &[u8],
) -> Result<AspectValueLocator, AspectValueLocatorCanonicalCodecError> {
    let mut reader = AspectValueLocatorReader::new(bytes);
    let tag = reader.read_u8()?;
    let authority = reader.read_authority()?;
    let aspect_key = reader.read_aspect_key()?;
    let locator = match tag {
        tags::WHOLE_ASPECT => {
            AspectValueLocator::whole_aspect(AspectLocator::new(authority, aspect_key))
        }
        tags::STRUCT_FIELD => {
            let field_path = reader.read_field_path()?;
            AspectValueLocator::struct_field(AspectFieldLocator::new(
                authority, aspect_key, field_path,
            ))
        }
        other => {
            return Err(AspectValueLocatorCanonicalCodecError::new(format!(
                "unknown aspect value locator tag {other}"
            )));
        }
    };
    reader.finish()?;
    Ok(locator)
}

pub(crate) fn decode_aspect_field_locator(
    bytes: &[u8],
) -> Result<AspectFieldLocator, AspectValueLocatorCanonicalCodecError> {
    match decode_aspect_value_locator(bytes)? {
        AspectValueLocator::StructField(locator) => Ok(locator),
        AspectValueLocator::WholeAspect(_) => Err(AspectValueLocatorCanonicalCodecError::new(
            "expected aspect field locator bytes, found whole-aspect locator",
        )),
    }
}

#[cfg(test)]
pub(crate) fn decode_boundary_source_locator(
    bytes: &[u8],
) -> Result<BoundarySourceLocator, AspectValueLocatorCanonicalCodecError> {
    match decode_aspect_value_locator(bytes)? {
        AspectValueLocator::WholeAspect(aspect) => Ok(BoundarySourceLocator::aspect(aspect)),
        AspectValueLocator::StructField(field) => Ok(BoundarySourceLocator::aspect_field(field)),
    }
}

struct AspectValueLocatorReader<'bytes> {
    bytes: &'bytes [u8],
    cursor: usize,
}

impl<'bytes> AspectValueLocatorReader<'bytes> {
    fn new(bytes: &'bytes [u8]) -> Self {
        Self { bytes, cursor: 0 }
    }

    fn finish(self) -> Result<(), AspectValueLocatorCanonicalCodecError> {
        if self.cursor == self.bytes.len() {
            Ok(())
        } else {
            Err(AspectValueLocatorCanonicalCodecError::new(format!(
                "aspect value locator codec left {} trailing bytes",
                self.bytes.len() - self.cursor
            )))
        }
    }

    fn read_authority(
        &mut self,
    ) -> Result<LocatorAuthority, AspectValueLocatorCanonicalCodecError> {
        match self.read_u8()? {
            tags::AUTHORITY_AUTHORITATIVE => Ok(LocatorAuthority::Authoritative),
            tags::AUTHORITY_DERIVED => Ok(LocatorAuthority::Derived),
            tags::AUTHORITY_PROJECTED => Ok(LocatorAuthority::Projected),
            tags::AUTHORITY_SUPPORT_ONLY => Ok(LocatorAuthority::SupportOnly),
            tags::AUTHORITY_PLANNED => Ok(LocatorAuthority::Planned),
            tags::AUTHORITY_RECEIPT_BEARING => Ok(LocatorAuthority::ReceiptBearing),
            other => Err(AspectValueLocatorCanonicalCodecError::new(format!(
                "unknown aspect value locator authority tag {other}"
            ))),
        }
    }

    fn read_aspect_key(&mut self) -> Result<AspectKey, AspectValueLocatorCanonicalCodecError> {
        let encoded_aspect_key = self.read_string()?;
        AspectKey::new(&encoded_aspect_key).ok_or_else(|| {
            AspectValueLocatorCanonicalCodecError::new(format!(
                "invalid aspect value locator aspect key `{encoded_aspect_key}`"
            ))
        })
    }

    fn read_field_path(
        &mut self,
    ) -> Result<CanonicalFieldPath, AspectValueLocatorCanonicalCodecError> {
        let field_count = self.read_u32()? as usize;
        let mut fields = Vec::with_capacity(field_count);
        for _ in 0..field_count {
            let encoded_field_key = self.read_string()?;
            fields.push(FieldKey::new(&encoded_field_key).ok_or_else(|| {
                AspectValueLocatorCanonicalCodecError::new(format!(
                    "invalid aspect value locator field key `{encoded_field_key}`"
                ))
            })?);
        }
        CanonicalFieldPath::new(fields).ok_or_else(|| {
            AspectValueLocatorCanonicalCodecError::new("empty aspect value locator field path")
        })
    }

    fn read_string(&mut self) -> Result<String, AspectValueLocatorCanonicalCodecError> {
        let length = self.read_u32()? as usize;
        let bytes = self.read_exact(length)?;
        std::str::from_utf8(bytes)
            .map(str::to_string)
            .map_err(|error| {
                AspectValueLocatorCanonicalCodecError::new(format!(
                    "invalid UTF-8 in aspect value locator string: {error}"
                ))
            })
    }

    fn read_u8(&mut self) -> Result<u8, AspectValueLocatorCanonicalCodecError> {
        Ok(self.read_exact(1)?[0])
    }

    fn read_u32(&mut self) -> Result<u32, AspectValueLocatorCanonicalCodecError> {
        let mut buffer = [0_u8; 4];
        buffer.copy_from_slice(self.read_exact(4)?);
        Ok(u32::from_le_bytes(buffer))
    }

    fn read_exact(
        &mut self,
        length: usize,
    ) -> Result<&'bytes [u8], AspectValueLocatorCanonicalCodecError> {
        if self.cursor + length > self.bytes.len() {
            return Err(AspectValueLocatorCanonicalCodecError::new(format!(
                "aspect value locator codec expected {length} bytes at offset {} but input ended early",
                self.cursor
            )));
        }
        let start = self.cursor;
        self.cursor += length;
        Ok(&self.bytes[start..self.cursor])
    }
}
