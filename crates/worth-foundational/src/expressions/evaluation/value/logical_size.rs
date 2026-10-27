//! Logical byte size of an already canonical expression value.
use super::{ExpressionValue, Repr};

impl ExpressionValue {
    /// The logical size charged for reading, retaining, or emitting a value.
    pub fn logical_bytes(&self) -> u64 {
        match &self.0 {
            Repr::Bool(_) | Repr::None => 1,
            Repr::Enum(_) => 4,
            Repr::Float32(_) => 4,
            Repr::Float64(_) | Repr::Quantity(_) => 8,
            Repr::Integer(_) => 16,
            Repr::Decimal { .. } => 17,
            Repr::String(text) => text.len() as u64,
            Repr::Bytes(bytes) | Repr::Id(bytes) => bytes.len() as u64,
            Repr::Bits { limbs, .. } => limbs.len() as u64 * 8,
            Repr::Logic4 { value, .. } => value.len() as u64 * 16,
            Repr::Some(inner) => inner.logical_bytes().saturating_add(1),
            Repr::List(parts) | Repr::Map(parts) | Repr::Record(parts) | Repr::Entry(parts) => {
                parts.bytes
            }
        }
    }
}
