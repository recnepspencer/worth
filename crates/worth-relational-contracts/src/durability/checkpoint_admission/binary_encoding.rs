//! Admission-image wire policy: fixed byte arrays use MessagePack bin values.
//! The adapter allocates nothing; ordinary vectors and non-byte tuples retain
//! their declared representation. This policy is local to unreleased WRTHCAI1.
use serde::{Serialize, Serializer};
pub mod byte_probe;
pub mod compounds;
pub mod serializer;
#[cfg(test)]
mod tests;
pub mod tuple;

pub struct BinaryFixedArrays<'a, T: ?Sized>(pub &'a T);

impl<T: Serialize + ?Sized> Serialize for BinaryFixedArrays<'_, T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer::BinarySerializer(serializer))
    }
}
