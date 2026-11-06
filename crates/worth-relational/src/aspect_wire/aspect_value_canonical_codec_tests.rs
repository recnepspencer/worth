use std::collections::BTreeMap;

use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, AspectValue, CanonicalBigInt, CanonicalDate, CanonicalDecimal,
    CanonicalF32, CanonicalF64, CanonicalRational, CanonicalTime, CanonicalTimestamp,
    CanonicalTimestampTz, ContentRefId, EntityId, FieldKey, InternedString, LocatorAuthority,
    PartitionId,
};

use super::{
    decode_aspect_value, encode_aspect_field_locator, encode_aspect_value,
    encode_length_prefixed_aspect_value, encode_u32,
};
use crate::commit_strategies::data::{
    decode_aspect_field_patch as decode_native_aspect_field_patch,
    decode_aspect_value as decode_native_aspect_value,
    encode_aspect_field_patch as encode_native_aspect_field_patch,
    encode_aspect_value as encode_native_aspect_value, NativeCodecReader,
};
use crate::transactions::data::AspectFieldPatch;

#[test]
fn native_strategy_codec_uses_canonical_aspect_value_bytes() {
    for value in canonical_aspect_value_samples() {
        let mut canonical_bytes = Vec::new();
        encode_length_prefixed_aspect_value(&mut canonical_bytes, &value);

        let mut native_bytes = Vec::new();
        encode_native_aspect_value(&mut native_bytes, &value);

        assert_eq!(native_bytes, canonical_bytes);

        let mut reader = NativeCodecReader::new(&native_bytes);
        assert_eq!(decode_native_aspect_value(&mut reader).unwrap(), value);
        reader.finish().unwrap();
    }
}

#[test]
fn reported_value_families_keep_one_canonical_tag_layout() {
    let reported_samples = [
        (
            AspectValue::InlineBytes(
                worth_foundational::InlineBytes::try_new(vec![0, 128, 255]).unwrap(),
            ),
            &[24, 3, 0, 0, 0, 0, 128, 255][..],
        ),
        (
            AspectValue::Int64(-12),
            &[5, 244, 255, 255, 255, 255, 255, 255, 255][..],
        ),
        (
            AspectValue::String(InternedString::Raw("stable".to_string())),
            &[12, 0, 6, 0, 0, 0, b's', b't', b'a', b'b', b'l', b'e'][..],
        ),
        (
            AspectValue::Float32(CanonicalF32::from_f32(1.5)),
            &[10, 0, 0, 192, 63][..],
        ),
    ];

    for (value, expected_body) in reported_samples {
        assert_eq!(encode_aspect_value(&value), expected_body);
        assert_eq!(decode_aspect_value(expected_body).unwrap(), value);
    }
}

#[test]
fn aspect_field_patch_canonical_bytes_use_shared_aspect_value_bodies() {
    let mut expected_bytes = Vec::new();
    let patch = patch_from_samples(&canonical_aspect_value_samples());

    encode_u32(&mut expected_bytes, patch.len() as u32);
    for (target, value) in patch.iter() {
        let target_bytes = encode_aspect_field_locator(target);
        encode_u32(&mut expected_bytes, target_bytes.len() as u32);
        expected_bytes.extend_from_slice(&target_bytes);
        encode_length_prefixed_aspect_value(&mut expected_bytes, value);
    }

    assert_eq!(patch.to_canonical_bytes().unwrap(), expected_bytes);
    assert_eq!(
        AspectFieldPatch::from_canonical_bytes(&expected_bytes).unwrap(),
        patch
    );
    // The original map decoder selected the last occurrence of an equal
    // locator. The concrete roster decoder must preserve that wire behavior.
    let locator = crate::transactions::data::planned_single_field_locator(
        AspectKey::new("strategy.spec").unwrap(),
        FieldKey::new("same").unwrap(),
    );
    let first = AspectFieldPatch::from_locator(locator.clone(), AspectValue::Int64(1))
        .to_canonical_bytes()
        .unwrap();
    let last = AspectFieldPatch::from_locator(locator.clone(), AspectValue::Int64(2))
        .to_canonical_bytes()
        .unwrap();
    let mut repeated = Vec::new();
    encode_u32(&mut repeated, 2);
    repeated.extend_from_slice(&first[4..]);
    repeated.extend_from_slice(&last[4..]);
    let decoded = AspectFieldPatch::from_canonical_bytes(&repeated).unwrap();
    assert_eq!(decoded.len(), 1);
    assert_eq!(decoded.get(&locator), Some(&AspectValue::Int64(2)));
    let impossible_count = 12u32.to_le_bytes();
    let denied = AspectFieldPatch::from_canonical_bytes(&impossible_count).unwrap_err();
    assert_eq!(denied.detail(), "field count exceeds the binary frame");
}

#[test]
fn aspect_field_patch_locator_bytes_are_canonical_planned_field_locator_bytes() {
    let locator = crate::transactions::data::planned_single_field_locator(
        AspectKey::new("strategy.spec").expect("valid aspect key"),
        FieldKey::new("replicas").expect("valid field key"),
    );

    let locator_bytes = encode_aspect_field_locator(&locator);
    assert!(!locator_bytes.is_empty());

    let non_planned_locator = AspectFieldLocator::new(
        LocatorAuthority::Derived,
        locator.aspect().aspect_key().clone(),
        locator.field_path().clone(),
    );
    let mut patch_bytes = Vec::new();
    encode_u32(&mut patch_bytes, 1);
    let locator_bytes = encode_aspect_field_locator(&non_planned_locator);
    encode_u32(&mut patch_bytes, locator_bytes.len() as u32);
    patch_bytes.extend_from_slice(&locator_bytes);
    encode_length_prefixed_aspect_value(&mut patch_bytes, &AspectValue::Bool(true));

    let error = AspectFieldPatch::from_canonical_bytes(&patch_bytes).unwrap_err();

    assert!(error.detail().contains("planned authority"));
}

#[test]
fn native_aspect_field_patch_codec_wraps_canonical_field_patch_bytes() {
    let patch = patch_from_samples(&canonical_aspect_value_samples());
    let patch_bytes = patch.to_canonical_bytes().unwrap();
    let mut expected_native_bytes = Vec::new();
    encode_u32(&mut expected_native_bytes, patch_bytes.len() as u32);
    expected_native_bytes.extend_from_slice(&patch_bytes);

    let mut native_bytes = Vec::new();
    encode_native_aspect_field_patch(&mut native_bytes, &patch).unwrap();

    assert_eq!(native_bytes, expected_native_bytes);

    let mut reader = NativeCodecReader::new(&native_bytes);
    assert_eq!(
        decode_native_aspect_field_patch(&mut reader).unwrap(),
        patch
    );
    reader.finish().unwrap();
}

#[test]
fn native_aspect_value_decode_returns_error_for_malformed_canonical_body() {
    let native_bytes = [1, 0, 0, 0, 255];
    let mut reader = NativeCodecReader::new(&native_bytes);

    let error = decode_native_aspect_value(&mut reader).unwrap_err();

    assert!(error
        .detail()
        .contains("unknown canonical aspect value tag 255"));
}

#[test]
fn native_field_patch_decode_returns_error_for_malformed_canonical_patch() {
    let native_bytes = [1, 0, 0, 0, 0];
    let mut reader = NativeCodecReader::new(&native_bytes);

    let error = decode_native_aspect_field_patch(&mut reader).unwrap_err();

    assert!(error.detail().contains("input ended early"));
}

#[test]
fn malformed_temporal_and_rational_bodies_return_codec_errors() {
    let mut invalid_time = vec![19];
    invalid_time.extend_from_slice(&CanonicalTime::NANOS_PER_DAY.to_le_bytes());
    let invalid_rational = [15, 1, 0, 0, 0, b'1', 1, 0, 0, 0, b'0'];

    assert!(decode_aspect_value(&invalid_time)
        .unwrap_err()
        .detail()
        .contains("time outside one day"));
    assert!(decode_aspect_value(&invalid_rational)
        .unwrap_err()
        .detail()
        .contains("zero denominator"));
}

#[test]
fn truncated_reference_body_returns_codec_error() {
    let truncated_entity_reference = [22, 1, 0, 0, 0, 2, 0, 0, 0];

    assert!(decode_aspect_value(&truncated_entity_reference)
        .unwrap_err()
        .detail()
        .contains("input ended early"));
}

fn canonical_aspect_value_samples() -> Vec<AspectValue> {
    vec![
        AspectValue::Null,
        AspectValue::Bool(true),
        AspectValue::Int8(-7),
        AspectValue::Int16(-320),
        AspectValue::Int32(-32000),
        AspectValue::Int64(-12),
        AspectValue::UInt8(7),
        AspectValue::UInt16(320),
        AspectValue::UInt32(32000),
        AspectValue::UInt64(12),
        AspectValue::Float32(CanonicalF32::from_f32(1.5)),
        AspectValue::Float64(CanonicalF64::from_f64(2.5)),
        AspectValue::Decimal(CanonicalDecimal::new("12.50")),
        AspectValue::BigInt(CanonicalBigInt::new("-12345678901234567890")),
        AspectValue::Rational(
            CanonicalRational::new(CanonicalBigInt::new("22"), CanonicalBigInt::new("7"))
                .expect("non-zero denominator"),
        ),
        AspectValue::String(InternedString::Raw("alpha".to_string())),
        AspectValue::Uuid([7; 16]),
        AspectValue::Date(CanonicalDate {
            days_from_unix_epoch: 20_000,
        }),
        AspectValue::Time(CanonicalTime::new(1_000).expect("valid time")),
        AspectValue::Timestamp(CanonicalTimestamp {
            micros_since_unix_epoch: 123_456,
        }),
        AspectValue::TimestampTz(CanonicalTimestampTz {
            utc_micros_since_unix_epoch: 123_456,
            offset_minutes: -360,
        }),
        AspectValue::EntityRef(EntityId::new(PartitionId(9), 10, 11)),
        AspectValue::ContentRef(ContentRefId([
            42, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ])),
    ]
}

fn patch_from_samples(samples: &[AspectValue]) -> AspectFieldPatch {
    let aspect_key = AspectKey::new("strategy.spec").expect("valid test aspect key");
    AspectFieldPatch::from(
        samples
            .iter()
            .enumerate()
            .map(|(index, value)| {
                let field = FieldKey::new(format!("field_{index}")).expect("valid test field key");
                (
                    crate::transactions::data::planned_single_field_locator(
                        aspect_key.clone(),
                        field,
                    ),
                    value.clone(),
                )
            })
            .collect::<BTreeMap<_, _>>(),
    )
}
