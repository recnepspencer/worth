//! Expose enum data in Serde's buffered string/single-entry-map content model.
use super::super::encoder::tag;
use super::{
    compound,
    reader::{CanonicalInputDecodeError as Error, Reader, Result},
};
use serde::de::{self, DeserializeSeed, Visitor};
use worth_foundational::facade::ValueCodecDenial;

pub(super) fn deserialize<'de, V: Visitor<'de>>(
    reader: &mut Reader<'de>,
    visitor: V,
) -> Result<V::Value> {
    let kind = reader.byte()?;
    reader.text()?;
    let variant = reader.text()?;
    if kind == tag::UNIT_VARIANT {
        return visitor.visit_borrowed_str(variant);
    }
    let prior = reader.enter()?;
    let mut access = BufferedVariant {
        reader,
        variant,
        kind,
        key_read: false,
        value_read: false,
    };
    let result = visitor.visit_map(&mut access);
    let complete = access.key_read && access.value_read;
    access.reader.bounds = prior;
    let value = result?;
    if !complete {
        return Err(Error(ValueCodecDenial::Malformed));
    }
    Ok(value)
}
struct BufferedVariant<'a, 'de> {
    reader: &'a mut Reader<'de>,
    variant: &'de str,
    kind: u8,
    key_read: bool,
    value_read: bool,
}
impl<'de> de::MapAccess<'de> for &mut BufferedVariant<'_, 'de> {
    type Error = Error;
    fn next_key_seed<K: DeserializeSeed<'de>>(&mut self, seed: K) -> Result<Option<K::Value>> {
        if self.value_read {
            return Ok(None);
        }
        if self.key_read {
            return Err(Error(ValueCodecDenial::Malformed));
        }
        self.key_read = true;
        seed.deserialize(de::value::BorrowedStrDeserializer::<Error>::new(
            self.variant,
        ))
        .map(Some)
    }
    fn next_value_seed<V: DeserializeSeed<'de>>(&mut self, seed: V) -> Result<V::Value> {
        if !self.key_read || self.value_read {
            return Err(Error(ValueCodecDenial::Malformed));
        }
        self.value_read = true;
        match self.kind {
            tag::NEWTYPE_VARIANT => seed.deserialize(&mut *self.reader),
            tag::TUPLE_VARIANT => compound::sequence_seed(self.reader, seed),
            tag::STRUCT_VARIANT => compound::mapping_seed(self.reader, seed),
            _ => Err(Error(ValueCodecDenial::Malformed)),
        }
    }
}
