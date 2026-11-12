//! Serde's member visitors consume the existing END/MORE compound grammar.
use super::super::encoder::tag;
use super::reader::{CanonicalInputDecodeError as Error, Reader, Result};
use serde::de::{self, DeserializeSeed, Visitor};
use worth_foundational::facade::ValueCodecDenial;

pub(super) fn sequence<'de, V: Visitor<'de>>(
    reader: &mut Reader<'de>,
    visitor: V,
) -> Result<V::Value> {
    let mut access = Sequence {
        reader,
        count: 0,
        finished: false,
    };
    let value = visitor.visit_seq(&mut access)?;
    access.finish()?;
    Ok(value)
}
struct Sequence<'a, 'de> {
    reader: &'a mut Reader<'de>,
    count: usize,
    finished: bool,
}
impl Sequence<'_, '_> {
    fn finish(&mut self) -> Result<()> {
        if !self.finished {
            self.reader.require(0)?;
            self.finished = true;
        }
        Ok(())
    }
}
impl<'de> de::SeqAccess<'de> for &mut Sequence<'_, 'de> {
    type Error = Error;
    fn next_element_seed<T: DeserializeSeed<'de>>(&mut self, seed: T) -> Result<Option<T::Value>> {
        if self.finished {
            return Ok(None);
        }
        if !self.reader.member(&mut self.count)? {
            self.finished = true;
            return Ok(None);
        }
        seed.deserialize(&mut *self.reader).map(Some)
    }
}
#[derive(Clone, Copy)]
pub(super) enum MapKeys {
    EncodedValues,
    StructFields,
}
pub(super) fn mapping<'de, V: Visitor<'de>>(
    reader: &mut Reader<'de>,
    keys: MapKeys,
    visitor: V,
) -> Result<V::Value> {
    let mut access = Mapping {
        reader,
        keys,
        count: 0,
        finished: false,
        pending_value: false,
    };
    let value = visitor.visit_map(&mut access)?;
    access.finish()?;
    Ok(value)
}
struct Mapping<'a, 'de> {
    reader: &'a mut Reader<'de>,
    keys: MapKeys,
    count: usize,
    finished: bool,
    pending_value: bool,
}
impl Mapping<'_, '_> {
    fn finish(&mut self) -> Result<()> {
        if self.pending_value {
            return Err(Error(ValueCodecDenial::Malformed));
        }
        if !self.finished {
            self.reader.require(0)?;
            self.finished = true;
        }
        Ok(())
    }
}
impl<'de> de::MapAccess<'de> for &mut Mapping<'_, 'de> {
    type Error = Error;
    fn next_key_seed<K: DeserializeSeed<'de>>(&mut self, seed: K) -> Result<Option<K::Value>> {
        if self.pending_value {
            return Err(Error(ValueCodecDenial::Malformed));
        }
        if self.finished {
            return Ok(None);
        }
        if !self.reader.member(&mut self.count)? {
            self.finished = true;
            return Ok(None);
        }
        self.pending_value = true;
        match self.keys {
            MapKeys::EncodedValues => seed.deserialize(&mut *self.reader).map(Some),
            MapKeys::StructFields => seed
                .deserialize(de::value::BorrowedStrDeserializer::<Error>::new(
                    self.reader.text()?,
                ))
                .map(Some),
        }
    }
    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value> {
        if !self.pending_value {
            return Err(Error(ValueCodecDenial::Malformed));
        }
        self.pending_value = false;
        seed.deserialize(&mut *self.reader)
    }
}

pub(super) struct Enumeration<'a, 'de> {
    pub(super) reader: &'a mut Reader<'de>,
    pub(super) variant: &'de str,
    pub(super) kind: u8,
}
impl<'a, 'de> de::EnumAccess<'de> for Enumeration<'a, 'de> {
    type Error = Error;
    type Variant = Self;
    fn variant_seed<V: DeserializeSeed<'de>>(self, seed: V) -> Result<(V::Value, Self)> {
        let variant = seed.deserialize(de::value::BorrowedStrDeserializer::<Error>::new(
            self.variant,
        ))?;
        Ok((variant, self))
    }
}
impl<'de> de::VariantAccess<'de> for Enumeration<'_, 'de> {
    type Error = Error;
    fn unit_variant(self) -> Result<()> {
        if self.kind == tag::UNIT_VARIANT {
            Ok(())
        } else {
            Err(Error(ValueCodecDenial::Malformed))
        }
    }
    fn newtype_variant_seed<T: DeserializeSeed<'de>>(self, seed: T) -> Result<T::Value> {
        match self.kind {
            tag::NEWTYPE_VARIANT => seed.deserialize(self.reader),
            // WorthValue's named variant seed asks for its payload as a map.
            // The wire still has the original STRUCT_VARIANT body and tags;
            // use the same complete-body adapter as buffered Serde enums.
            tag::STRUCT_VARIANT => mapping_seed(self.reader, seed),
            _ => Err(Error(ValueCodecDenial::Malformed)),
        }
    }
    fn tuple_variant<V: Visitor<'de>>(self, _: usize, visitor: V) -> Result<V::Value> {
        if self.kind != tag::TUPLE_VARIANT {
            return Err(Error(ValueCodecDenial::Malformed));
        }
        sequence(self.reader, visitor)
    }
    fn struct_variant<V: Visitor<'de>>(
        self,
        _: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value> {
        if self.kind != tag::STRUCT_VARIANT {
            return Err(Error(ValueCodecDenial::Malformed));
        }
        mapping(self.reader, MapKeys::StructFields, visitor)
    }
}

// Buffered enum payloads contain the same compound body without a second tag.
pub(super) fn sequence_seed<'de, T: DeserializeSeed<'de>>(
    reader: &mut Reader<'de>,
    seed: T,
) -> Result<T::Value> {
    let mut access = Sequence {
        reader,
        count: 0,
        finished: false,
    };
    let value = seed.deserialize(de::value::SeqAccessDeserializer::new(&mut access))?;
    access.finish()?;
    Ok(value)
}
pub(super) fn mapping_seed<'de, T: DeserializeSeed<'de>>(
    reader: &mut Reader<'de>,
    seed: T,
) -> Result<T::Value> {
    let mut access = Mapping {
        reader,
        keys: MapKeys::StructFields,
        count: 0,
        finished: false,
        pending_value: false,
    };
    let value = seed.deserialize(de::value::MapAccessDeserializer::new(&mut access))?;
    access.finish()?;
    Ok(value)
}
