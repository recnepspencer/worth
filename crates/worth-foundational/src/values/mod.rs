mod aspect_value;
mod references;
mod scalar_kind;
mod scalar_wrappers;

pub use aspect_value::AspectValue;
pub use references::{ContentRefId, EntityId, Generation, LocalSlot, PartitionId};
pub use scalar_kind::ScalarAspectType;
pub use scalar_wrappers::{
    decode_canonical_text_hex, encode_canonical_text_hex, CanonicalBigInt, CanonicalDate,
    CanonicalDecimal, CanonicalF32, CanonicalF64, CanonicalRational, CanonicalString,
    CanonicalTime, CanonicalTimestamp, CanonicalTimestampTz, InternedString, Symbol,
};

use crate::facade::ResponsibilityArea;

pub fn responsibility() -> ResponsibilityArea {
    ResponsibilityArea::new(
        "canonical_values",
        "canonical value carriers and representation-normalized scalar wrappers",
        "aspect contracts, mutation execution, or runtime storage layout",
    )
}
