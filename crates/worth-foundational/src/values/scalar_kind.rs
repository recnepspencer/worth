use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ScalarAspectType {
    Null,
    Bool,
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float32,
    Float64,
    Decimal,
    BigInt,
    Rational,
    String,
    Bytes,
    Uuid,
    Date,
    Time,
    Timestamp,
    TimestampTz,
    EntityRef,
    ContentRef,
    InlineBytes,
}

impl ScalarAspectType {
    /// Stable vocabulary for canonical cross-boundary identity material.
    pub const fn canonical_name(self) -> &'static str {
        match self {
            Self::Null => "null",
            Self::Bool => "bool",
            Self::Int8 => "int8",
            Self::Int16 => "int16",
            Self::Int32 => "int32",
            Self::Int64 => "int64",
            Self::UInt8 => "uint8",
            Self::UInt16 => "uint16",
            Self::UInt32 => "uint32",
            Self::UInt64 => "uint64",
            Self::Float32 => "float32",
            Self::Float64 => "float64",
            Self::Decimal => "decimal",
            Self::BigInt => "big-int",
            Self::Rational => "rational",
            Self::String => "string",
            Self::Bytes => "bytes",
            Self::Uuid => "uuid",
            Self::Date => "date",
            Self::Time => "time",
            Self::Timestamp => "timestamp",
            Self::TimestampTz => "timestamp-tz",
            Self::EntityRef => "entity-ref",
            Self::ContentRef => "content-ref",
            Self::InlineBytes => "inline-bytes",
        }
    }
}
