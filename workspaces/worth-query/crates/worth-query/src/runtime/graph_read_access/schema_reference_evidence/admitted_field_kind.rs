use crate::schema_view::ScalarAspectType;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryAdmittedNativeFieldFamily(ScalarAspectType);

impl WorthQueryAdmittedNativeFieldFamily {
    pub fn native_family(&self) -> ScalarAspectType {
        self.0
    }

    pub fn as_str(&self) -> &'static str {
        match self.0 {
            ScalarAspectType::InlineBytes => "inline-bytes",
            ScalarAspectType::Null => "null",
            ScalarAspectType::Bool => "bool",
            ScalarAspectType::Int8 => "int8",
            ScalarAspectType::Int16 => "int16",
            ScalarAspectType::Int32 => "int32",
            ScalarAspectType::Int64 => "int64",
            ScalarAspectType::UInt8 => "uint8",
            ScalarAspectType::UInt16 => "uint16",
            ScalarAspectType::UInt32 => "uint32",
            ScalarAspectType::UInt64 => "uint64",
            ScalarAspectType::Float32 => "float32",
            ScalarAspectType::Float64 => "float64",
            ScalarAspectType::Decimal => "decimal",
            ScalarAspectType::BigInt => "big-int",
            ScalarAspectType::Rational => "rational",
            ScalarAspectType::String => "string",
            ScalarAspectType::Bytes => "bytes",
            ScalarAspectType::Uuid => "uuid",
            ScalarAspectType::Date => "date",
            ScalarAspectType::Time => "time",
            ScalarAspectType::Timestamp => "timestamp",
            ScalarAspectType::TimestampTz => "timestamp-tz",
            ScalarAspectType::EntityRef => "entity-ref",
            ScalarAspectType::ContentRef => "content-ref",
        }
    }

    pub(crate) fn from_schema_field_kind(kind: &ScalarAspectType) -> Self {
        Self(*kind)
    }
}
