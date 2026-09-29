//! Prefix-free canonical encoding of any `Serialize` value.
//!
//! Every value starts with a one-byte tag and ends where its tag says it ends,
//! so no encoding is a prefix of another and two different values of one type
//! never share bytes. Text, byte and name lengths are LEB128 prefixes. A
//! compound value writes a `MORE` byte before each member and `END` after the
//! last, so it streams into a hasher without knowing its length up front.
//! Integers encode by value in a sign tag plus minimal LEB128 (signed values
//! zigzag first), so a narrower and a wider type holding one value agree.
//! Type, field and variant names are encoded, map entries are sorted by their
//! encoded key, and floats encode their exact bits.

use serde::ser::{self, Serialize};
use sha2::{Digest, Sha256};

mod compound;

use compound::Compound;

#[derive(Debug)]
pub(super) struct CanonicalEncodeError;

impl std::fmt::Display for CanonicalEncodeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("value serialization rejected")
    }
}

impl std::error::Error for CanonicalEncodeError {}

impl ser::Error for CanonicalEncodeError {
    fn custom<T: std::fmt::Display>(_: T) -> Self {
        Self
    }
}

/// Where encoded bytes go: a hasher for an identity, a buffer for a map entry
/// that must be sorted before it counts.
pub(super) trait Sink {
    fn put(&mut self, bytes: &[u8]);

    /// Records bytes a compound value held in memory to sort its map entries.
    fn buffered(&mut self, bytes: usize);
}

/// A map entry encoded on its own so its enclosing map can sort it.
#[derive(Default)]
struct EntryBuffer {
    bytes: Vec<u8>,
    nested_buffered: usize,
}

impl Sink for EntryBuffer {
    fn put(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }

    fn buffered(&mut self, bytes: usize) {
        self.nested_buffered = self.nested_buffered.saturating_add(bytes);
    }
}

impl Sink for Vec<u8> {
    fn put(&mut self, bytes: &[u8]) {
        self.extend_from_slice(bytes);
    }

    fn buffered(&mut self, _: usize) {}
}

/// Feeds a hasher and counts what the encoding cost, so an identity reports
/// the work that derived it.
pub(super) struct HashingSink {
    digest: Sha256,
    hashed_bytes: usize,
    buffered_bytes: usize,
}

impl HashingSink {
    pub(super) fn new() -> Self {
        Self {
            digest: Sha256::new(),
            hashed_bytes: 0,
            buffered_bytes: 0,
        }
    }

    /// Bytes fed to SHA-256 so far, framing included.
    pub(super) const fn hashed_bytes(&self) -> usize {
        self.hashed_bytes
    }

    /// Map entry bytes held in memory so far.
    pub(super) const fn buffered_bytes(&self) -> usize {
        self.buffered_bytes
    }

    pub(super) fn finish(self) -> [u8; 32] {
        self.digest.finalize().into()
    }
}

impl Sink for HashingSink {
    fn put(&mut self, bytes: &[u8]) {
        Digest::update(&mut self.digest, bytes);
        self.hashed_bytes = self.hashed_bytes.saturating_add(bytes.len());
    }

    fn buffered(&mut self, bytes: usize) {
        self.buffered_bytes = self.buffered_bytes.saturating_add(bytes);
    }
}

pub(super) fn encode_into<S: Sink, T: Serialize + ?Sized>(
    sink: &mut S,
    value: &T,
) -> Result<(), CanonicalEncodeError> {
    value.serialize(Encoder { sink })
}

/// Encodes one map entry side and returns its bytes with what its own nested
/// maps buffered.
fn encode_entry<T: Serialize + ?Sized>(value: &T) -> Result<EntryBuffer, CanonicalEncodeError> {
    let mut buffer = EntryBuffer::default();
    encode_into(&mut buffer, value)?;
    Ok(buffer)
}

mod tag {
    pub const BOOL: u8 = 1;
    pub const SIGNED: u8 = 2;
    pub const UNSIGNED: u8 = 3;
    pub const F32: u8 = 4;
    pub const F64: u8 = 5;
    pub const CHAR: u8 = 6;
    pub const STR: u8 = 7;
    pub const BYTES: u8 = 8;
    pub const NONE: u8 = 9;
    pub const SOME: u8 = 10;
    pub const UNIT: u8 = 11;
    pub const UNIT_STRUCT: u8 = 12;
    pub const UNIT_VARIANT: u8 = 13;
    pub const NEWTYPE_STRUCT: u8 = 14;
    pub const NEWTYPE_VARIANT: u8 = 15;
    pub const SEQ: u8 = 16;
    pub const TUPLE: u8 = 17;
    pub const TUPLE_STRUCT: u8 = 18;
    pub const TUPLE_VARIANT: u8 = 19;
    pub const MAP: u8 = 20;
    pub const STRUCT: u8 = 21;
    pub const STRUCT_VARIANT: u8 = 22;
}

const END: u8 = 0;
const MORE: u8 = 1;

/// Unsigned LEB128: seven bits per byte, low group first, high bit set while
/// more groups follow. The shortest form is the only one written.
fn put_varint<S: Sink>(sink: &mut S, mut value: u128) {
    let mut buffer = [0_u8; 19];
    let mut length = 0;
    loop {
        let group = (value & 0x7f) as u8;
        value >>= 7;
        buffer[length] = if value == 0 { group } else { group | 0x80 };
        length += 1;
        if value == 0 {
            break;
        }
    }
    sink.put(&buffer[..length]);
}

fn put_len<S: Sink>(sink: &mut S, len: usize) {
    put_varint(sink, len as u128);
}

fn put_text<S: Sink>(sink: &mut S, text: &str) {
    put_len(sink, text.len());
    sink.put(text.as_bytes());
}

struct Encoder<'a, S> {
    sink: &'a mut S,
}

impl<'a, S: Sink> Encoder<'a, S> {
    fn tagged(self, tag: u8, payload: &[u8]) -> Result<(), CanonicalEncodeError> {
        self.sink.put(&[tag]);
        self.sink.put(payload);
        Ok(())
    }

    fn compound(self, tag: u8, names: &[&str]) -> Compound<'a, S> {
        self.sink.put(&[tag]);
        for name in names {
            put_text(self.sink, name);
        }
        Compound::new(self.sink)
    }
}

impl<'a, S: Sink> ser::Serializer for Encoder<'a, S> {
    type Ok = ();
    type Error = CanonicalEncodeError;
    type SerializeSeq = Compound<'a, S>;
    type SerializeTuple = Compound<'a, S>;
    type SerializeTupleStruct = Compound<'a, S>;
    type SerializeTupleVariant = Compound<'a, S>;
    type SerializeMap = Compound<'a, S>;
    type SerializeStruct = Compound<'a, S>;
    type SerializeStructVariant = Compound<'a, S>;

    fn serialize_bool(self, value: bool) -> Result<(), Self::Error> {
        self.tagged(tag::BOOL, &[u8::from(value)])
    }
    fn serialize_i8(self, value: i8) -> Result<(), Self::Error> {
        self.serialize_i128(value.into())
    }
    fn serialize_i16(self, value: i16) -> Result<(), Self::Error> {
        self.serialize_i128(value.into())
    }
    fn serialize_i32(self, value: i32) -> Result<(), Self::Error> {
        self.serialize_i128(value.into())
    }
    fn serialize_i64(self, value: i64) -> Result<(), Self::Error> {
        self.serialize_i128(value.into())
    }
    fn serialize_i128(self, value: i128) -> Result<(), Self::Error> {
        let zigzag = ((value << 1) ^ (value >> 127)) as u128;
        self.sink.put(&[tag::SIGNED]);
        put_varint(self.sink, zigzag);
        Ok(())
    }
    fn serialize_u8(self, value: u8) -> Result<(), Self::Error> {
        self.serialize_u128(value.into())
    }
    fn serialize_u16(self, value: u16) -> Result<(), Self::Error> {
        self.serialize_u128(value.into())
    }
    fn serialize_u32(self, value: u32) -> Result<(), Self::Error> {
        self.serialize_u128(value.into())
    }
    fn serialize_u64(self, value: u64) -> Result<(), Self::Error> {
        self.serialize_u128(value.into())
    }
    fn serialize_u128(self, value: u128) -> Result<(), Self::Error> {
        self.sink.put(&[tag::UNSIGNED]);
        put_varint(self.sink, value);
        Ok(())
    }
    fn serialize_f32(self, value: f32) -> Result<(), Self::Error> {
        self.tagged(tag::F32, &value.to_bits().to_be_bytes())
    }
    fn serialize_f64(self, value: f64) -> Result<(), Self::Error> {
        self.tagged(tag::F64, &value.to_bits().to_be_bytes())
    }
    fn serialize_char(self, value: char) -> Result<(), Self::Error> {
        self.tagged(tag::CHAR, &u32::from(value).to_be_bytes())
    }
    fn serialize_str(self, value: &str) -> Result<(), Self::Error> {
        self.sink.put(&[tag::STR]);
        put_text(self.sink, value);
        Ok(())
    }
    fn serialize_bytes(self, value: &[u8]) -> Result<(), Self::Error> {
        self.sink.put(&[tag::BYTES]);
        put_len(self.sink, value.len());
        self.sink.put(value);
        Ok(())
    }
    fn serialize_none(self) -> Result<(), Self::Error> {
        self.tagged(tag::NONE, &[])
    }
    fn serialize_some<T: Serialize + ?Sized>(self, value: &T) -> Result<(), Self::Error> {
        self.sink.put(&[tag::SOME]);
        value.serialize(self)
    }
    fn serialize_unit(self) -> Result<(), Self::Error> {
        self.tagged(tag::UNIT, &[])
    }
    fn serialize_unit_struct(self, name: &'static str) -> Result<(), Self::Error> {
        self.sink.put(&[tag::UNIT_STRUCT]);
        put_text(self.sink, name);
        Ok(())
    }
    fn serialize_unit_variant(
        self,
        name: &'static str,
        _: u32,
        variant: &'static str,
    ) -> Result<(), Self::Error> {
        self.sink.put(&[tag::UNIT_VARIANT]);
        put_text(self.sink, name);
        put_text(self.sink, variant);
        Ok(())
    }
    fn serialize_newtype_struct<T: Serialize + ?Sized>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        self.sink.put(&[tag::NEWTYPE_STRUCT]);
        put_text(self.sink, name);
        value.serialize(self)
    }
    fn serialize_newtype_variant<T: Serialize + ?Sized>(
        self,
        name: &'static str,
        _: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        self.sink.put(&[tag::NEWTYPE_VARIANT]);
        put_text(self.sink, name);
        put_text(self.sink, variant);
        value.serialize(self)
    }
    fn serialize_seq(self, _: Option<usize>) -> Result<Compound<'a, S>, Self::Error> {
        Ok(self.compound(tag::SEQ, &[]))
    }
    fn serialize_tuple(self, _: usize) -> Result<Compound<'a, S>, Self::Error> {
        Ok(self.compound(tag::TUPLE, &[]))
    }
    fn serialize_tuple_struct(
        self,
        name: &'static str,
        _: usize,
    ) -> Result<Compound<'a, S>, Self::Error> {
        Ok(self.compound(tag::TUPLE_STRUCT, &[name]))
    }
    fn serialize_tuple_variant(
        self,
        name: &'static str,
        _: u32,
        variant: &'static str,
        _: usize,
    ) -> Result<Compound<'a, S>, Self::Error> {
        Ok(self.compound(tag::TUPLE_VARIANT, &[name, variant]))
    }
    fn serialize_map(self, _: Option<usize>) -> Result<Compound<'a, S>, Self::Error> {
        Ok(self.compound(tag::MAP, &[]))
    }
    fn serialize_struct(
        self,
        name: &'static str,
        _: usize,
    ) -> Result<Compound<'a, S>, Self::Error> {
        Ok(self.compound(tag::STRUCT, &[name]))
    }
    fn serialize_struct_variant(
        self,
        name: &'static str,
        _: u32,
        variant: &'static str,
        _: usize,
    ) -> Result<Compound<'a, S>, Self::Error> {
        Ok(self.compound(tag::STRUCT_VARIANT, &[name, variant]))
    }
}
