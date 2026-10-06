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

use serde::Serialize;

mod compound;
mod serializer;
mod sink;

use serializer::Encoder;
pub use sink::CanonicalEncodingCharge;
pub(super) use sink::{
    CallbackAdmission, CanonicalEncodeError, EncodingAdmission, HashingSink, Sink,
};

pub(super) fn encode_into<S: Sink, T: Serialize + ?Sized>(
    sink: &mut S,
    value: &T,
) -> Result<(), CanonicalEncodeError<S::AdmissionError>> {
    sink.ensure_active()?;
    value.serialize(Encoder { sink })
}

fn encode_entry<'a, S: Sink, T: Serialize + ?Sized>(
    sink: &'a mut S,
    value: &T,
) -> Result<sink::EntryBuffer<'a, S::AdmissionError>, CanonicalEncodeError<S::AdmissionError>> {
    let mut buffer = sink::EntryBuffer::new(sink);
    encode_into(&mut buffer, value)?;
    Ok(buffer)
}
