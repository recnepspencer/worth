//! Detect a Serde u8 by its declared type, never by its numerical value.
use serde::{ser::Impossible, Serialize, Serializer};
pub(super) struct ByteProbe(pub(super) bool);
#[derive(Debug)]
pub(super) struct NotByte;
impl std::fmt::Display for NotByte {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("not a byte")
    }
}
impl std::error::Error for NotByte {}
impl serde::ser::Error for NotByte {
    fn custom<T: std::fmt::Display>(_: T) -> Self {
        Self
    }
}
impl Serializer for ByteProbe {
    type Ok = u8;
    type Error = NotByte;
    type SerializeSeq = Impossible<u8, NotByte>;
    type SerializeTuple = Impossible<u8, NotByte>;
    type SerializeTupleStruct = Impossible<u8, NotByte>;
    type SerializeTupleVariant = Impossible<u8, NotByte>;
    type SerializeMap = Impossible<u8, NotByte>;
    type SerializeStruct = Impossible<u8, NotByte>;
    type SerializeStructVariant = Impossible<u8, NotByte>;
    fn serialize_bool(self, _: bool) -> Result<u8, NotByte> {
        Err(NotByte)
    }
    fn serialize_i8(self, _: i8) -> Result<u8, NotByte> {
        Err(NotByte)
    }
    fn serialize_i16(self, _: i16) -> Result<u8, NotByte> {
        Err(NotByte)
    }
    fn serialize_i32(self, _: i32) -> Result<u8, NotByte> {
        Err(NotByte)
    }
    fn serialize_i64(self, _: i64) -> Result<u8, NotByte> {
        Err(NotByte)
    }
    fn serialize_i128(self, _: i128) -> Result<u8, NotByte> {
        Err(NotByte)
    }
    fn serialize_u8(self, value: u8) -> Result<u8, NotByte> {
        Ok(value)
    }
    fn serialize_u16(self, _: u16) -> Result<u8, NotByte> {
        Err(NotByte)
    }
    fn serialize_u32(self, _: u32) -> Result<u8, NotByte> {
        Err(NotByte)
    }
    fn serialize_u64(self, _: u64) -> Result<u8, NotByte> {
        Err(NotByte)
    }
    fn serialize_u128(self, _: u128) -> Result<u8, NotByte> {
        Err(NotByte)
    }
    fn serialize_f32(self, _: f32) -> Result<u8, NotByte> {
        Err(NotByte)
    }
    fn serialize_f64(self, _: f64) -> Result<u8, NotByte> {
        Err(NotByte)
    }
    fn serialize_char(self, _: char) -> Result<u8, NotByte> {
        Err(NotByte)
    }
    fn serialize_str(self, _: &str) -> Result<u8, NotByte> {
        Err(NotByte)
    }
    fn serialize_bytes(self, _: &[u8]) -> Result<u8, NotByte> {
        Err(NotByte)
    }
    fn serialize_none(self) -> Result<u8, NotByte> {
        Err(NotByte)
    }
    fn serialize_some<T: Serialize + ?Sized>(self, _: &T) -> Result<u8, NotByte> {
        Err(NotByte)
    }
    fn serialize_unit(self) -> Result<u8, NotByte> {
        Err(NotByte)
    }
    fn serialize_unit_struct(self, _: &'static str) -> Result<u8, NotByte> {
        Err(NotByte)
    }
    fn serialize_unit_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
    ) -> Result<u8, NotByte> {
        Err(NotByte)
    }
    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        _: &'static str,
        _: &T,
    ) -> Result<u8, NotByte> {
        Err(NotByte)
    }
    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: &T,
    ) -> Result<u8, NotByte> {
        Err(NotByte)
    }
    fn serialize_seq(self, _: Option<usize>) -> Result<Self::SerializeSeq, NotByte> {
        Err(NotByte)
    }
    fn serialize_tuple(self, _: usize) -> Result<Self::SerializeTuple, NotByte> {
        Err(NotByte)
    }
    fn serialize_tuple_struct(
        self,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleStruct, NotByte> {
        Err(NotByte)
    }
    fn serialize_tuple_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeTupleVariant, NotByte> {
        Err(NotByte)
    }
    fn serialize_map(self, _: Option<usize>) -> Result<Self::SerializeMap, NotByte> {
        Err(NotByte)
    }
    fn serialize_struct(self, _: &'static str, _: usize) -> Result<Self::SerializeStruct, NotByte> {
        Err(NotByte)
    }
    fn serialize_struct_variant(
        self,
        _: &'static str,
        _: u32,
        _: &'static str,
        _: usize,
    ) -> Result<Self::SerializeStructVariant, NotByte> {
        Err(NotByte)
    }
    fn is_human_readable(&self) -> bool {
        self.0
    }
    fn collect_str<T: std::fmt::Display + ?Sized>(self, _: &T) -> Result<u8, NotByte> {
        // Serde's default would allocate a String merely to reject this value.
        Err(NotByte)
    }
}
