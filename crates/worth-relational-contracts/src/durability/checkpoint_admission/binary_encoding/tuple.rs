//! Delay only bounded fixed tuples, never allocate a serializer staging vector.
use super::{byte_probe::ByteProbe, BinaryFixedArrays};
use serde::{ser::SerializeTuple, Serialize, Serializer};

// Every fixed digest/root/seal in this protocol is at most 64 octets.
const MAXIMUM_FIXED_BYTES: usize = 64;
pub(super) struct FixedByteTuple<S: Serializer> {
    serializer: Option<S>,
    sequence: Option<S::SerializeTuple>,
    bytes: [u8; MAXIMUM_FIXED_BYTES],
    count: usize,
    length: usize,
}
impl<S: Serializer> FixedByteTuple<S> {
    pub(super) fn new(serializer: S, length: usize) -> Result<Self, S::Error> {
        let (serializer, sequence) = if length > 0 && length <= MAXIMUM_FIXED_BYTES {
            (Some(serializer), None)
        } else {
            (None, Some(serializer.serialize_tuple(length)?))
        };
        Ok(Self {
            serializer,
            sequence,
            bytes: [0; MAXIMUM_FIXED_BYTES],
            count: 0,
            length,
        })
    }
}
impl<S: Serializer> SerializeTuple for FixedByteTuple<S> {
    type Ok = S::Ok;
    type Error = S::Error;
    fn serialize_element<T: Serialize + ?Sized>(&mut self, value: &T) -> Result<(), Self::Error> {
        if self.count >= self.length {
            return Err(serde::ser::Error::custom(
                "fixed tuple exceeds its declared length",
            ));
        }
        if let Some(serializer) = &self.serializer {
            if let Ok(byte) = value.serialize(ByteProbe(serializer.is_human_readable())) {
                self.bytes[self.count] = byte;
                self.count += 1;
                return Ok(());
            }
            let serializer = self
                .serializer
                .take()
                .expect("pending tuple has its serializer");
            let mut sequence = serializer.serialize_tuple(self.length)?;
            for byte in &self.bytes[..self.count] {
                sequence.serialize_element(byte)?;
            }
            self.sequence = Some(sequence);
        }
        self.sequence
            .as_mut()
            .expect("non-byte tuple has its sequence")
            .serialize_element(&BinaryFixedArrays(value))?;
        self.count += 1;
        Ok(())
    }
    fn end(self) -> Result<Self::Ok, Self::Error> {
        if self.count != self.length {
            return Err(serde::ser::Error::custom(
                "fixed tuple is shorter than its declared length",
            ));
        }
        match self.serializer {
            Some(serializer) => serializer.serialize_bytes(&self.bytes[..self.count]),
            None => self
                .sequence
                .expect("ordinary tuple has its sequence")
                .end(),
        }
    }
}
