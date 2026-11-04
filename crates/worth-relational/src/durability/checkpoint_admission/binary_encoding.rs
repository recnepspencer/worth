//! Admission-image wire policy: fixed byte arrays use MessagePack bin values.
//! The adapter allocates nothing; ordinary vectors and non-byte tuples retain
//! their declared representation. This policy is local to unreleased WRTHCAI1.
use serde::{Serialize, Serializer};
mod byte_probe;
mod compounds;
mod serializer;
#[cfg(test)]
mod tests;
mod tuple;

pub(crate) struct BinaryFixedArrays<'a, T: ?Sized>(pub(crate) &'a T);

impl<T: Serialize + ?Sized> Serialize for BinaryFixedArrays<'_, T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer::BinarySerializer(serializer))
    }
}
