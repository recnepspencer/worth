use super::{InternedString, Symbol};
use serde::ser::Impossible;
use serde::{Serialize, Serializer};

#[derive(Debug, PartialEq, Eq)]
struct Variant {
    owner: &'static str,
    index: u32,
    name: &'static str,
    payload: serde_json::Value,
}

struct EnumWire;
macro_rules! refuse {
    ($($name:ident($($arg:ident: $kind:ty),*) -> $output:ty;)*) => {
        $(fn $name(self, $(_: $kind),*) -> Result<$output, Self::Error> {
            Err(serde::ser::Error::custom("the wire probe expects an enum variant"))
        })*
    };
}

impl Serializer for EnumWire {
    type Ok = Variant;
    type Error = serde_json::Error;
    type SerializeSeq = Impossible<Variant, Self::Error>;
    type SerializeTuple = Impossible<Variant, Self::Error>;
    type SerializeTupleStruct = Impossible<Variant, Self::Error>;
    type SerializeTupleVariant = Impossible<Variant, Self::Error>;
    type SerializeMap = Impossible<Variant, Self::Error>;
    type SerializeStruct = Impossible<Variant, Self::Error>;
    type SerializeStructVariant = Impossible<Variant, Self::Error>;

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        owner: &'static str,
        index: u32,
        name: &'static str,
        value: &T,
    ) -> Result<Variant, Self::Error> {
        Ok(Variant {
            owner,
            index,
            name,
            payload: serde_json::to_value(value)?,
        })
    }
    fn serialize_some<T: ?Sized + Serialize>(self, _: &T) -> Result<Variant, Self::Error> {
        Err(serde::ser::Error::custom("expected an enum variant"))
    }
    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _: &'static str,
        _: &T,
    ) -> Result<Variant, Self::Error> {
        Err(serde::ser::Error::custom("expected an enum variant"))
    }
    refuse! {
        serialize_bool(v: bool) -> Variant;
        serialize_i8(v: i8) -> Variant;
        serialize_i16(v: i16) -> Variant;
        serialize_i32(v: i32) -> Variant;
        serialize_i64(v: i64) -> Variant;
        serialize_u8(v: u8) -> Variant;
        serialize_u16(v: u16) -> Variant;
        serialize_u32(v: u32) -> Variant;
        serialize_u64(v: u64) -> Variant;
        serialize_f32(v: f32) -> Variant;
        serialize_f64(v: f64) -> Variant;
        serialize_char(v: char) -> Variant;
        serialize_str(v: &str) -> Variant;
        serialize_bytes(v: &[u8]) -> Variant;
        serialize_none() -> Variant;
        serialize_unit() -> Variant;
        serialize_unit_struct(n: &'static str) -> Variant;
        serialize_unit_variant(n: &'static str, i: u32, v: &'static str) -> Variant;
        serialize_seq(n: Option<usize>) -> Self::SerializeSeq;
        serialize_tuple(n: usize) -> Self::SerializeTuple;
        serialize_tuple_struct(n: &'static str, c: usize) -> Self::SerializeTupleStruct;
        serialize_tuple_variant(n: &'static str, i: u32, v: &'static str, c: usize) -> Self::SerializeTupleVariant;
        serialize_map(n: Option<usize>) -> Self::SerializeMap;
        serialize_struct(n: &'static str, c: usize) -> Self::SerializeStruct;
        serialize_struct_variant(n: &'static str, i: u32, v: &'static str, c: usize) -> Self::SerializeStructVariant;
    }
}

#[test]
fn cached_text_keeps_legacy_named_and_numeric_wire_discriminators() {
    #[derive(Serialize)]
    #[serde(rename = "InternedString")]
    enum Legacy {
        Raw(String),
        Symbol(Symbol),
    }
    for text in ["", "alpha", "β"] {
        let legacy = Legacy::Raw(text.to_owned()).serialize(EnumWire).unwrap();
        let raw = InternedString::Raw(text.to_owned());
        let stored =
            InternedString::Stored(crate::value_shape::StoredValueText::new(text.to_owned()));
        assert_eq!(raw.serialize(EnumWire).unwrap(), legacy);
        assert_eq!(stored.serialize(EnumWire).unwrap(), legacy);
    }
    assert_eq!(
        InternedString::Symbol(Symbol(7))
            .serialize(EnumWire)
            .unwrap(),
        Legacy::Symbol(Symbol(7)).serialize(EnumWire).unwrap()
    );
}
