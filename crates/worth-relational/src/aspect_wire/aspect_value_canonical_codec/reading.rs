use worth_foundational::facade::{
    AspectValue, CanonicalBigInt, CanonicalDate, CanonicalDecimal, CanonicalF32, CanonicalF64,
    CanonicalRational, CanonicalTime, CanonicalTimestamp, CanonicalTimestampTz, ContentRefId,
    EntityId, InternedString, PartitionId, Symbol,
};

use super::{tags, AspectValueCanonicalCodecError};

pub(crate) fn decode_aspect_value(
    bytes: &[u8],
) -> Result<AspectValue, AspectValueCanonicalCodecError> {
    let mut reader = AspectValueBodyReader::new(bytes);
    let value = reader.read_aspect_value_body()?;
    reader.finish()?;
    Ok(value)
}

pub(crate) struct LengthPrefixedAspectValueReader<'a> {
    bytes: &'a [u8],
    cursor: usize,
}

impl<'a> LengthPrefixedAspectValueReader<'a> {
    pub(crate) fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, cursor: 0 }
    }

    pub(crate) fn finish(self) -> Result<(), AspectValueCanonicalCodecError> {
        if self.cursor == self.bytes.len() {
            Ok(())
        } else {
            Err(AspectValueCanonicalCodecError::new(format!(
                "canonical aspect value frame codec left {} trailing bytes",
                self.bytes.len() - self.cursor
            )))
        }
    }

    pub(crate) fn read_length_prefixed_aspect_value(
        &mut self,
    ) -> Result<AspectValue, AspectValueCanonicalCodecError> {
        let length = self.read_u32()? as usize;
        let bytes = self.read_exact(length)?;
        decode_aspect_value(bytes)
    }

    fn read_u32(&mut self) -> Result<u32, AspectValueCanonicalCodecError> {
        let mut buffer = [0_u8; 4];
        buffer.copy_from_slice(self.read_exact(4)?);
        Ok(u32::from_le_bytes(buffer))
    }

    fn read_exact(&mut self, length: usize) -> Result<&'a [u8], AspectValueCanonicalCodecError> {
        if self.cursor + length > self.bytes.len() {
            return Err(AspectValueCanonicalCodecError::new(format!(
                "canonical aspect value frame codec expected {length} bytes at offset {} but input ended early",
                self.cursor
            )));
        }
        let start = self.cursor;
        self.cursor += length;
        Ok(&self.bytes[start..self.cursor])
    }
}

struct AspectValueBodyReader<'a> {
    bytes: &'a [u8],
    cursor: usize,
}

impl<'a> AspectValueBodyReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, cursor: 0 }
    }

    fn finish(self) -> Result<(), AspectValueCanonicalCodecError> {
        if self.cursor == self.bytes.len() {
            Ok(())
        } else {
            Err(AspectValueCanonicalCodecError::new(format!(
                "canonical aspect value codec left {} trailing bytes",
                self.bytes.len() - self.cursor
            )))
        }
    }

    fn read_aspect_value_body(&mut self) -> Result<AspectValue, AspectValueCanonicalCodecError> {
        match self.read_u8()? {
            tags::INLINE_BYTES => {
                let length = self.read_u32()? as usize;
                worth_foundational::InlineBytes::admit_length(length)
                    .map_err(|cause| AspectValueCanonicalCodecError::new(cause.to_string()))?;
                let value =
                    worth_foundational::InlineBytes::try_new(self.read_exact(length)?.to_vec())
                        .map_err(|cause| AspectValueCanonicalCodecError::new(cause.to_string()))?;
                Ok(AspectValue::InlineBytes(value))
            }
            tags::NULL => Ok(AspectValue::Null),
            tags::BOOL => Ok(AspectValue::Bool(self.read_bool()?)),
            tags::INT8 => Ok(AspectValue::Int8(self.read_u8()? as i8)),
            tags::INT16 => Ok(AspectValue::Int16(self.read_i16()?)),
            tags::INT32 => Ok(AspectValue::Int32(self.read_i32()?)),
            tags::INT64 => Ok(AspectValue::Int64(self.read_i64()?)),
            tags::UINT8 => Ok(AspectValue::UInt8(self.read_u8()?)),
            tags::UINT16 => Ok(AspectValue::UInt16(self.read_u16()?)),
            tags::UINT32 => Ok(AspectValue::UInt32(self.read_u32()?)),
            tags::UINT64 => Ok(AspectValue::UInt64(self.read_u64()?)),
            tags::FLOAT32 => Ok(AspectValue::Float32(CanonicalF32::from_bits(
                self.read_u32()?,
            ))),
            tags::FLOAT64 => Ok(AspectValue::Float64(CanonicalF64::from_bits(
                self.read_u64()?,
            ))),
            tags::STRING => self.read_string_value(),
            tags::DECIMAL => Ok(AspectValue::Decimal(CanonicalDecimal::new(
                self.read_string()?,
            ))),
            tags::BIG_INT => Ok(AspectValue::BigInt(self.read_big_int()?)),
            tags::RATIONAL => self.read_rational_value(),
            tags::UUID => Ok(AspectValue::Uuid(self.read_uuid()?)),
            tags::DATE => Ok(AspectValue::Date(CanonicalDate {
                days_from_unix_epoch: self.read_i32()?,
            })),
            tags::TIME => self.read_time_value(),
            tags::TIMESTAMP => Ok(AspectValue::Timestamp(CanonicalTimestamp {
                micros_since_unix_epoch: self.read_i64()?,
            })),
            tags::TIMESTAMP_TZ => Ok(AspectValue::TimestampTz(CanonicalTimestampTz {
                utc_micros_since_unix_epoch: self.read_i64()?,
                offset_minutes: self.read_i32()?,
            })),
            tags::ENTITY_REF => Ok(AspectValue::EntityRef(self.read_entity_id()?)),
            tags::CONTENT_REF => Ok(AspectValue::ContentRef(self.read_content_ref()?)),
            tag => Err(AspectValueCanonicalCodecError::new(format!(
                "unknown canonical aspect value tag {tag}"
            ))),
        }
    }

    fn read_string_value(&mut self) -> Result<AspectValue, AspectValueCanonicalCodecError> {
        match self.read_u8()? {
            tags::RAW_STRING => Ok(AspectValue::String(InternedString::Raw(
                self.read_string()?,
            ))),
            tags::SYMBOL_STRING => Ok(AspectValue::String(InternedString::Symbol(Symbol(
                self.read_u32()?,
            )))),
            tag => Err(AspectValueCanonicalCodecError::new(format!(
                "unknown canonical aspect string tag {tag}"
            ))),
        }
    }

    fn read_big_int(&mut self) -> Result<CanonicalBigInt, AspectValueCanonicalCodecError> {
        Ok(CanonicalBigInt::new(self.read_string()?))
    }

    fn read_rational_value(&mut self) -> Result<AspectValue, AspectValueCanonicalCodecError> {
        let numerator = self.read_big_int()?;
        let denominator = self.read_big_int()?;
        let rational = CanonicalRational::new(numerator, denominator).ok_or_else(|| {
            AspectValueCanonicalCodecError::new(
                "canonical aspect value codec rejected rational with zero denominator",
            )
        })?;
        Ok(AspectValue::Rational(rational))
    }

    fn read_time_value(&mut self) -> Result<AspectValue, AspectValueCanonicalCodecError> {
        let nanos_since_midnight = self.read_u64()?;
        let time = CanonicalTime::new(nanos_since_midnight).ok_or_else(|| {
            AspectValueCanonicalCodecError::new(
                "canonical aspect value codec rejected time outside one day",
            )
        })?;
        Ok(AspectValue::Time(time))
    }

    fn read_uuid(&mut self) -> Result<[u8; 16], AspectValueCanonicalCodecError> {
        let mut uuid = [0_u8; 16];
        uuid.copy_from_slice(self.read_exact(16)?);
        Ok(uuid)
    }

    fn read_content_ref(&mut self) -> Result<ContentRefId, AspectValueCanonicalCodecError> {
        let mut digest = [0; 32];
        digest.copy_from_slice(self.read_exact(32)?);
        Ok(ContentRefId(digest))
    }

    fn read_entity_id(&mut self) -> Result<EntityId, AspectValueCanonicalCodecError> {
        let partition_id = PartitionId(self.read_u32()?);
        let local_slot = self.read_u64()?;
        let generation = self.read_u32()?;
        Ok(EntityId::new(partition_id, local_slot, generation))
    }

    fn read_string(&mut self) -> Result<String, AspectValueCanonicalCodecError> {
        let length = self.read_u32()? as usize;
        let bytes = self.read_exact(length)?;
        std::str::from_utf8(bytes)
            .map(str::to_owned)
            .map_err(|error| {
                AspectValueCanonicalCodecError::new(format!(
                    "invalid utf-8 string in canonical aspect value codec: {error}"
                ))
            })
    }

    fn read_bool(&mut self) -> Result<bool, AspectValueCanonicalCodecError> {
        match self.read_u8()? {
            0 => Ok(false),
            1 => Ok(true),
            value => Err(AspectValueCanonicalCodecError::new(format!(
                "invalid canonical aspect bool tag {value}"
            ))),
        }
    }

    fn read_u8(&mut self) -> Result<u8, AspectValueCanonicalCodecError> {
        Ok(self.read_exact(1)?[0])
    }

    fn read_u16(&mut self) -> Result<u16, AspectValueCanonicalCodecError> {
        let mut buffer = [0_u8; 2];
        buffer.copy_from_slice(self.read_exact(2)?);
        Ok(u16::from_le_bytes(buffer))
    }

    fn read_i16(&mut self) -> Result<i16, AspectValueCanonicalCodecError> {
        let mut buffer = [0_u8; 2];
        buffer.copy_from_slice(self.read_exact(2)?);
        Ok(i16::from_le_bytes(buffer))
    }

    fn read_u32(&mut self) -> Result<u32, AspectValueCanonicalCodecError> {
        let mut buffer = [0_u8; 4];
        buffer.copy_from_slice(self.read_exact(4)?);
        Ok(u32::from_le_bytes(buffer))
    }

    fn read_i32(&mut self) -> Result<i32, AspectValueCanonicalCodecError> {
        let mut buffer = [0_u8; 4];
        buffer.copy_from_slice(self.read_exact(4)?);
        Ok(i32::from_le_bytes(buffer))
    }

    fn read_u64(&mut self) -> Result<u64, AspectValueCanonicalCodecError> {
        let mut buffer = [0_u8; 8];
        buffer.copy_from_slice(self.read_exact(8)?);
        Ok(u64::from_le_bytes(buffer))
    }

    fn read_i64(&mut self) -> Result<i64, AspectValueCanonicalCodecError> {
        let mut buffer = [0_u8; 8];
        buffer.copy_from_slice(self.read_exact(8)?);
        Ok(i64::from_le_bytes(buffer))
    }

    fn read_exact(&mut self, length: usize) -> Result<&'a [u8], AspectValueCanonicalCodecError> {
        if self.cursor + length > self.bytes.len() {
            return Err(AspectValueCanonicalCodecError::new(format!(
                "canonical aspect value codec expected {length} bytes at offset {} but input ended early",
                self.cursor
            )));
        }
        let start = self.cursor;
        self.cursor += length;
        Ok(&self.bytes[start..self.cursor])
    }
}
