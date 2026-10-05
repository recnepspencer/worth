use worth_foundational::facade::{
    AspectValue, CanonicalBigInt, ContentRefId, EntityId, InternedString,
};

use super::{encode_string, encode_u32, tags};

pub(crate) fn encode_length_prefixed_aspect_value(bytes: &mut Vec<u8>, value: &AspectValue) {
    let mut value_bytes = Vec::new();
    encode_aspect_value_body(&mut value_bytes, value);
    encode_u32(bytes, value_bytes.len() as u32);
    bytes.extend_from_slice(&value_bytes);
}

pub(crate) fn encode_aspect_value(value: &AspectValue) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(encoded_aspect_value_len(value).unwrap_or(0));
    encode_aspect_value_body(&mut bytes, value);
    bytes
}

/// Exact canonical key backing length before encoding allocates its Vec.
pub(crate) fn encoded_aspect_value_len(value: &AspectValue) -> Option<usize> {
    let text = |text: &str| {
        u32::try_from(text.len()).ok()?;
        4usize.checked_add(text.len())
    };
    match value {
        AspectValue::Null => Some(1),
        AspectValue::Bool(_) | AspectValue::Int8(_) | AspectValue::UInt8(_) => Some(2),
        AspectValue::Int16(_) | AspectValue::UInt16(_) => Some(3),
        AspectValue::Int32(_)
        | AspectValue::UInt32(_)
        | AspectValue::Float32(_)
        | AspectValue::Date(_) => Some(5),
        AspectValue::Int64(_)
        | AspectValue::UInt64(_)
        | AspectValue::Float64(_)
        | AspectValue::Time(_)
        | AspectValue::Timestamp(_)
        | AspectValue::Bytes(_)
        | AspectValue::ContentRef(_) => Some(9),
        AspectValue::String(InternedString::Raw(value)) => 2usize.checked_add(text(value)?),
        AspectValue::String(InternedString::Symbol(_)) => Some(6),
        AspectValue::Decimal(value) => 1usize.checked_add(text(value.as_str())?),
        AspectValue::BigInt(value) => 1usize.checked_add(text(value.as_str())?),
        AspectValue::Rational(value) => 1usize
            .checked_add(text(value.numerator.as_str())?)?
            .checked_add(text(value.denominator.as_str())?),
        AspectValue::Uuid(_) | AspectValue::EntityRef(_) => Some(17),
        AspectValue::TimestampTz(_) => Some(13),
    }
}

fn encode_aspect_value_body(bytes: &mut Vec<u8>, value: &AspectValue) {
    match value {
        AspectValue::Null => bytes.push(tags::NULL),
        AspectValue::Bool(value) => {
            bytes.push(tags::BOOL);
            bytes.push(u8::from(*value));
        }
        AspectValue::Int8(value) => {
            bytes.push(tags::INT8);
            bytes.push(*value as u8);
        }
        AspectValue::Int16(value) => {
            bytes.push(tags::INT16);
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        AspectValue::Int32(value) => {
            bytes.push(tags::INT32);
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        AspectValue::Int64(value) => {
            bytes.push(tags::INT64);
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        AspectValue::UInt8(value) => {
            bytes.push(tags::UINT8);
            bytes.push(*value);
        }
        AspectValue::UInt16(value) => {
            bytes.push(tags::UINT16);
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        AspectValue::UInt32(value) => {
            bytes.push(tags::UINT32);
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        AspectValue::UInt64(value) => {
            bytes.push(tags::UINT64);
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        AspectValue::Float32(value) => {
            bytes.push(tags::FLOAT32);
            bytes.extend_from_slice(&value.bits().to_le_bytes());
        }
        AspectValue::Float64(value) => {
            bytes.push(tags::FLOAT64);
            bytes.extend_from_slice(&value.bits().to_le_bytes());
        }
        AspectValue::String(value) => {
            bytes.push(tags::STRING);
            encode_interned_string(bytes, value);
        }
        AspectValue::Decimal(value) => {
            bytes.push(tags::DECIMAL);
            encode_string(bytes, value.as_str());
        }
        AspectValue::BigInt(value) => {
            bytes.push(tags::BIG_INT);
            encode_big_int(bytes, value);
        }
        AspectValue::Rational(value) => {
            bytes.push(tags::RATIONAL);
            encode_big_int(bytes, &value.numerator);
            encode_big_int(bytes, &value.denominator);
        }
        AspectValue::Bytes(value) => {
            bytes.push(tags::BYTES);
            encode_content_ref(bytes, *value);
        }
        AspectValue::Uuid(value) => {
            bytes.push(tags::UUID);
            bytes.extend_from_slice(value);
        }
        AspectValue::Date(value) => {
            bytes.push(tags::DATE);
            bytes.extend_from_slice(&value.days_from_unix_epoch.to_le_bytes());
        }
        AspectValue::Time(value) => {
            bytes.push(tags::TIME);
            bytes.extend_from_slice(&value.nanos_since_midnight.to_le_bytes());
        }
        AspectValue::Timestamp(value) => {
            bytes.push(tags::TIMESTAMP);
            bytes.extend_from_slice(&value.micros_since_unix_epoch.to_le_bytes());
        }
        AspectValue::TimestampTz(value) => {
            bytes.push(tags::TIMESTAMP_TZ);
            bytes.extend_from_slice(&value.utc_micros_since_unix_epoch.to_le_bytes());
            bytes.extend_from_slice(&value.offset_minutes.to_le_bytes());
        }
        AspectValue::EntityRef(value) => {
            bytes.push(tags::ENTITY_REF);
            encode_entity_id(bytes, *value);
        }
        AspectValue::ContentRef(value) => {
            bytes.push(tags::CONTENT_REF);
            encode_content_ref(bytes, *value);
        }
    }
}

fn encode_interned_string(bytes: &mut Vec<u8>, value: &InternedString) {
    match value {
        InternedString::Raw(text) => {
            bytes.push(tags::RAW_STRING);
            encode_string(bytes, text);
        }
        InternedString::Symbol(symbol) => {
            bytes.push(tags::SYMBOL_STRING);
            encode_u32(bytes, symbol.0);
        }
    }
}

fn encode_big_int(bytes: &mut Vec<u8>, value: &CanonicalBigInt) {
    encode_string(bytes, value.as_str());
}

fn encode_content_ref(bytes: &mut Vec<u8>, value: ContentRefId) {
    bytes.extend_from_slice(&value.0.to_le_bytes());
}

fn encode_entity_id(bytes: &mut Vec<u8>, value: EntityId) {
    bytes.extend_from_slice(&value.partition_id.0.to_le_bytes());
    bytes.extend_from_slice(&value.local_slot.0.to_le_bytes());
    bytes.extend_from_slice(&value.generation.0.to_le_bytes());
}
