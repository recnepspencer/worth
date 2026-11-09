//! Typed serde dispatch; byte grammar and bound progression remain concrete.
use super::super::encoder::tag;
use super::compound::{self, Enumeration, MapKeys};
use super::reader::{CanonicalInputDecodeError as Error, Reader, Result};
use serde::de::{self, Visitor};
use worth_foundational::facade::ValueCodecDenial;

impl<'de> de::Deserializer<'de> for &mut Reader<'de> {
    type Error = Error;
    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        let kind = *self
            .remaining
            .first()
            .ok_or(Error(ValueCodecDenial::Malformed))?;
        match kind {
            tag::BOOL => self.deserialize_bool(visitor),
            tag::SIGNED => {
                let value = self.signed()?;
                match i64::try_from(value) {
                    Ok(value) => visitor.visit_i64(value),
                    Err(_) => visitor.visit_i128(value),
                }
            }
            tag::UNSIGNED => {
                let value = self.unsigned()?;
                match u64::try_from(value) {
                    Ok(value) => visitor.visit_u64(value),
                    Err(_) => visitor.visit_u128(value),
                }
            }
            tag::F32 => self.deserialize_f32(visitor),
            tag::F64 => self.deserialize_f64(visitor),
            tag::CHAR => self.deserialize_char(visitor),
            tag::STR => self.deserialize_str(visitor),
            tag::BYTES => self.deserialize_bytes(visitor),
            tag::NONE | tag::SOME => self.deserialize_option(visitor),
            tag::UNIT => self.deserialize_unit(visitor),
            tag::UNIT_STRUCT => {
                self.require(tag::UNIT_STRUCT)?;
                self.text()?;
                visitor.visit_unit()
            }
            tag::NEWTYPE_STRUCT => {
                self.require(tag::NEWTYPE_STRUCT)?;
                self.text()?;
                let prior = self.enter()?;
                let result = visitor.visit_newtype_struct(&mut *self);
                self.bounds = prior;
                result
            }
            tag::SEQ | tag::TUPLE => sequence(self, kind, visitor),
            tag::TUPLE_STRUCT => {
                self.require(tag::TUPLE_STRUCT)?;
                self.text()?;
                sequence_body(self, visitor)
            }
            tag::MAP => {
                self.require(tag::MAP)?;
                mapping_body(self, MapKeys::EncodedValues, visitor)
            }
            tag::STRUCT => {
                self.require(tag::STRUCT)?;
                self.text()?;
                mapping_body(self, MapKeys::StructFields, visitor)
            }
            tag::UNIT_VARIANT | tag::NEWTYPE_VARIANT | tag::TUPLE_VARIANT | tag::STRUCT_VARIANT => {
                super::buffered_enum::deserialize(self, visitor)
            }
            _ => Err(Error(ValueCodecDenial::Malformed)),
        }
    }
    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.require(tag::BOOL)?;
        match self.byte()? {
            0 => visitor.visit_bool(false),
            1 => visitor.visit_bool(true),
            _ => Err(Error(ValueCodecDenial::Malformed)),
        }
    }
    fn deserialize_i8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor
            .visit_i8(i8::try_from(self.signed()?).map_err(|_| Error(ValueCodecDenial::Malformed))?)
    }
    fn deserialize_i16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i16(
            i16::try_from(self.signed()?).map_err(|_| Error(ValueCodecDenial::Malformed))?,
        )
    }
    fn deserialize_i32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i32(
            i32::try_from(self.signed()?).map_err(|_| Error(ValueCodecDenial::Malformed))?,
        )
    }
    fn deserialize_i64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i64(
            i64::try_from(self.signed()?).map_err(|_| Error(ValueCodecDenial::Malformed))?,
        )
    }
    fn deserialize_i128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_i128(self.signed()?)
    }
    fn deserialize_u8<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u8(
            u8::try_from(self.unsigned()?).map_err(|_| Error(ValueCodecDenial::Malformed))?,
        )
    }
    fn deserialize_u16<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u16(
            u16::try_from(self.unsigned()?).map_err(|_| Error(ValueCodecDenial::Malformed))?,
        )
    }
    fn deserialize_u32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u32(
            u32::try_from(self.unsigned()?).map_err(|_| Error(ValueCodecDenial::Malformed))?,
        )
    }
    fn deserialize_u64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u64(
            u64::try_from(self.unsigned()?).map_err(|_| Error(ValueCodecDenial::Malformed))?,
        )
    }
    fn deserialize_u128<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        visitor.visit_u128(self.unsigned()?)
    }
    fn deserialize_f32<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.require(tag::F32)?;
        visitor.visit_f32(f32::from_bits(u32::from_be_bytes(
            self.take(4)?.try_into().expect("exact length"),
        )))
    }
    fn deserialize_f64<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.require(tag::F64)?;
        visitor.visit_f64(f64::from_bits(u64::from_be_bytes(
            self.take(8)?.try_into().expect("exact length"),
        )))
    }
    fn deserialize_char<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.require(tag::CHAR)?;
        let value = char::from_u32(u32::from_be_bytes(
            self.take(4)?.try_into().expect("exact length"),
        ))
        .ok_or(Error(ValueCodecDenial::Malformed))?;
        visitor.visit_char(value)
    }
    fn deserialize_str<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.require(tag::STR)?;
        visitor.visit_borrowed_str(self.text()?)
    }
    fn deserialize_string<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_str(visitor)
    }
    fn deserialize_bytes<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.require(tag::BYTES)?;
        let bytes = self.bytes()?;
        if self
            .bounds
            .maximum_sequence_length()
            .is_some_and(|maximum| bytes.len() > maximum)
        {
            return Err(Error(ValueCodecDenial::SequenceLength));
        }
        visitor.visit_borrowed_bytes(bytes)
    }
    fn deserialize_byte_buf<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_bytes(visitor)
    }
    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        match self.byte()? {
            tag::NONE => visitor.visit_none(),
            tag::SOME => {
                let prior = self.enter()?;
                let result = visitor.visit_some(&mut *self);
                self.bounds = prior;
                result
            }
            _ => Err(Error(ValueCodecDenial::Malformed)),
        }
    }
    fn deserialize_unit<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.require(tag::UNIT)?;
        visitor.visit_unit()
    }
    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        self.require(tag::UNIT_STRUCT)?;
        self.name(name)?;
        visitor.visit_unit()
    }
    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value> {
        self.require(tag::NEWTYPE_STRUCT)?;
        self.name(name)?;
        let prior = self.enter()?;
        let result = visitor.visit_newtype_struct(&mut *self);
        self.bounds = prior;
        result
    }
    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        sequence(self, tag::SEQ, visitor)
    }
    fn deserialize_tuple<V: Visitor<'de>>(self, _: usize, visitor: V) -> Result<V::Value> {
        sequence(self, tag::TUPLE, visitor)
    }
    fn deserialize_tuple_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        _: usize,
        visitor: V,
    ) -> Result<V::Value> {
        self.require(tag::TUPLE_STRUCT)?;
        self.name(name)?;
        sequence_body(self, visitor)
    }
    fn deserialize_map<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.require(tag::MAP)?;
        mapping_body(self, MapKeys::EncodedValues, visitor)
    }
    fn deserialize_struct<V: Visitor<'de>>(
        self,
        name: &'static str,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        self.require(tag::STRUCT)?;
        self.name(name)?;
        mapping_body(self, MapKeys::StructFields, visitor)
    }
    fn deserialize_enum<V: Visitor<'de>>(
        self,
        name: &'static str,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        enumeration(self, Some(name), visitor)
    }
    fn deserialize_identifier<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_str(visitor)
    }
    fn deserialize_ignored_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value> {
        self.deserialize_any(visitor)
    }
}
fn sequence<'de, V: Visitor<'de>>(
    reader: &mut Reader<'de>,
    tag: u8,
    visitor: V,
) -> Result<V::Value> {
    reader.require(tag)?;
    sequence_body(reader, visitor)
}
fn sequence_body<'de, V: Visitor<'de>>(reader: &mut Reader<'de>, visitor: V) -> Result<V::Value> {
    let prior = reader.enter()?;
    let result = compound::sequence(reader, visitor);
    reader.bounds = prior;
    result
}
fn mapping_body<'de, V: Visitor<'de>>(
    reader: &mut Reader<'de>,
    keys: MapKeys,
    visitor: V,
) -> Result<V::Value> {
    let prior = reader.enter()?;
    let result = compound::mapping(reader, keys, visitor);
    reader.bounds = prior;
    result
}
fn enumeration<'de, V: Visitor<'de>>(
    reader: &mut Reader<'de>,
    name: Option<&str>,
    visitor: V,
) -> Result<V::Value> {
    let kind = reader.byte()?;
    if !matches!(
        kind,
        tag::UNIT_VARIANT | tag::NEWTYPE_VARIANT | tag::TUPLE_VARIANT | tag::STRUCT_VARIANT
    ) {
        return Err(Error(ValueCodecDenial::Malformed));
    }
    match name {
        Some(name) => reader.name(name)?,
        None => {
            reader.text()?;
        }
    }
    let variant = reader.text()?;
    let prior = if kind == tag::UNIT_VARIANT {
        reader.bounds
    } else {
        reader.enter()?
    };
    let result = visitor.visit_enum(Enumeration {
        reader: &mut *reader,
        variant,
        kind,
    });
    reader.bounds = prior;
    result
}
