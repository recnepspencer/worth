use worth_foundational::facade::{AspectValue, InternedString, StructAspectValue};

use crate::projection_consumption::ConsumedNativeValueView;

pub(super) fn canonical_snapshot_read_value(
    value: &worth_runtime_bridge::facade::SnapshotReadValue,
) -> String {
    match value {
        worth_runtime_bridge::facade::SnapshotReadValue::Scalar(value) => {
            canonical_aspect_value(value)
        }
        worth_runtime_bridge::facade::SnapshotReadValue::Struct(value) => {
            canonical_struct_aspect_value(value)
        }
    }
}

pub(super) fn canonical_consumed_native_value(value: ConsumedNativeValueView<'_>) -> String {
    match value {
        ConsumedNativeValueView::Scalar(value) => canonical_aspect_value(value),
        ConsumedNativeValueView::Struct(value) => canonical_struct_aspect_value(value),
        ConsumedNativeValueView::Absent(posture) => {
            let mut material = String::new();
            token(&mut material, "value.kind", "absent");
            token(
                &mut material,
                "value.absence",
                match posture {
                    worth_foundational::facade::AbsenceLaw::Required => "required",
                    worth_foundational::facade::AbsenceLaw::Optional => "optional",
                    worth_foundational::facade::AbsenceLaw::Defaulted => "defaulted",
                },
            );
            material
        }
    }
}

pub(super) fn raw_string_snapshot_value(
    value: &worth_runtime_bridge::facade::SnapshotReadValue,
) -> Option<&str> {
    match value {
        worth_runtime_bridge::facade::SnapshotReadValue::Scalar(AspectValue::String(
            InternedString::Raw(value),
        )) => Some(value.as_str()),
        _ => None,
    }
}

pub(super) fn canonical_aspect_value(value: &AspectValue) -> String {
    let mut material = String::new();
    match value {
        AspectValue::InlineBytes(value) => {
            token(&mut material, "value.kind", "inline-bytes");
            bytes(&mut material, "value.inline-bytes", value.as_bytes());
        }
        AspectValue::Null => token(&mut material, "value.kind", "null"),
        AspectValue::Bool(value) => {
            token(&mut material, "value.kind", "bool");
            token(
                &mut material,
                "value.bool",
                if *value { "true" } else { "false" },
            );
        }
        AspectValue::Int8(value) => signed(&mut material, "i8", i128::from(*value)),
        AspectValue::Int16(value) => signed(&mut material, "i16", i128::from(*value)),
        AspectValue::Int32(value) => signed(&mut material, "i32", i128::from(*value)),
        AspectValue::Int64(value) => signed(&mut material, "i64", i128::from(*value)),
        AspectValue::UInt8(value) => unsigned(&mut material, "i8", u128::from(*value)),
        AspectValue::UInt16(value) => unsigned(&mut material, "i16", u128::from(*value)),
        AspectValue::UInt32(value) => unsigned(&mut material, "i32", u128::from(*value)),
        AspectValue::UInt64(value) => unsigned(&mut material, "i64", u128::from(*value)),
        AspectValue::Float32(value) => float(&mut material, "f32", u64::from(value.bits())),
        AspectValue::Float64(value) => float(&mut material, "f64", value.bits()),
        AspectValue::Decimal(value) => {
            token(&mut material, "value.kind", "decimal");
            token(&mut material, "value.decimal.raw", value.as_str());
        }
        AspectValue::BigInt(value) => {
            token(&mut material, "value.kind", "bigint");
            token(&mut material, "value.bigint.raw", value.as_str());
        }
        AspectValue::Rational(value) => {
            token(&mut material, "value.kind", "rational");
            token(
                &mut material,
                "value.rational.numerator.raw",
                value.numerator.as_str(),
            );
            token(
                &mut material,
                "value.rational.denominator.raw",
                value.denominator.as_str(),
            );
        }
        AspectValue::String(value) => {
            token(&mut material, "value.kind", "text");
            interned(&mut material, "value.text", value);
        }
        AspectValue::Bytes(value) => {
            token(&mut material, "value.kind", "bytes-ref");
            token(&mut material, "value.bytes-ref", &value.0.to_string());
        }
        AspectValue::Uuid(value) => {
            token(&mut material, "value.kind", "uuid");
            bytes(&mut material, "value.uuid", value);
        }
        AspectValue::Date(value) => {
            token(&mut material, "value.kind", "date-days");
            token(
                &mut material,
                "value.date-days",
                &value.days_from_unix_epoch.to_string(),
            );
        }
        AspectValue::Time(value) => {
            token(&mut material, "value.kind", "time-nanos");
            token(
                &mut material,
                "value.time-nanos",
                &value.nanos_since_midnight.to_string(),
            );
        }
        AspectValue::Timestamp(value) => {
            token(&mut material, "value.kind", "timestamp-micros");
            token(
                &mut material,
                "value.timestamp-micros",
                &value.micros_since_unix_epoch.to_string(),
            );
        }
        AspectValue::TimestampTz(value) => {
            token(&mut material, "value.kind", "timestamp-tz");
            token(
                &mut material,
                "value.timestamp-tz.utc-micros",
                &value.utc_micros_since_unix_epoch.to_string(),
            );
            token(
                &mut material,
                "value.timestamp-tz.offset-minutes",
                &value.offset_minutes.to_string(),
            );
        }
        AspectValue::EntityRef(value) => {
            token(&mut material, "value.kind", "entity-ref");
            token(
                &mut material,
                "value.entity.partition",
                &value.partition_id.0.to_string(),
            );
            token(
                &mut material,
                "value.entity.slot",
                &value.local_slot.0.to_string(),
            );
            token(
                &mut material,
                "value.entity.generation",
                &value.generation.0.to_string(),
            );
        }
        AspectValue::ContentRef(value) => {
            token(&mut material, "value.kind", "content-ref");
            token(&mut material, "value.content-ref", &value.0.to_string());
        }
    }
    material
}

fn canonical_struct_aspect_value(value: &StructAspectValue) -> String {
    let mut material = String::new();
    token(&mut material, "value.kind", "struct");
    token(
        &mut material,
        "value.struct.field-count",
        &value.fields().count().to_string(),
    );
    for (field, value) in value.fields() {
        token(&mut material, "value.struct.field", field.as_str());
        material.push_str(&canonical_aspect_value(value));
    }
    material
}

fn signed(material: &mut String, width: &str, value: i128) {
    token(material, "value.kind", "signed");
    token(material, "value.width", width);
    token(material, "value.signed", &value.to_string());
}

fn unsigned(material: &mut String, width: &str, value: u128) {
    token(material, "value.kind", "unsigned");
    token(material, "value.width", width);
    token(material, "value.unsigned", &value.to_string());
}

fn float(material: &mut String, width: &str, bits_value: u64) {
    token(material, "value.kind", "float");
    token(material, "value.width", width);
    token(material, "value.float-bits", &bits_value.to_string());
}

fn interned(material: &mut String, label: &str, value: &InternedString) {
    match value {
        InternedString::Raw(value) => token(material, &format!("{label}.raw"), value),
        InternedString::Symbol(value) => {
            token(material, &format!("{label}.symbol"), &value.0.to_string())
        }
    }
}

fn token(material: &mut String, label: &str, value: &str) {
    material.push_str(label);
    material.push('#');
    material.push_str(&value.len().to_string());
    material.push(':');
    material.push_str(value);
    material.push(';');
}

fn bytes(material: &mut String, label: &str, value: &[u8]) {
    material.push_str(label);
    material.push('#');
    material.push_str(&value.len().to_string());
    material.push(':');
    for byte in value {
        material.push_str(&format!("{byte:02x}"));
    }
    material.push(';');
}
