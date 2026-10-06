use worth_foundational::facade::{AspectValue, BoundaryProtocolVersion, InternedString};

use super::restoration::{hex_bytes, restore_record};
use super::Denial;

#[test]
fn durable_field_restore_accepts_the_exact_committed_shape() {
    let restored = restore_record(valid_restored_fields()).unwrap();
    assert_eq!(restored.correlation().bytes(), &[7; 32]);
    assert_eq!(restored.correlation_family().as_str(), "family");
    assert_eq!(restored.effect(), "effect");
    assert_eq!(restored.protocol_identity().as_str(), "test.effect");
    assert_eq!(restored.protocol_version(), BoundaryProtocolVersion::new(1));
    assert_eq!(restored.maximum_payload_bytes(), 24);
    assert_eq!(restored.payload(), [1, 2]);
    assert_eq!(restored.outcome_identity(), 9);
    assert_eq!(restored.operation_slot(), Some("notify"));
}

#[test]
fn durable_field_restore_rejects_every_omission_and_wrong_storage_type() {
    let valid = valid_restored_fields();
    for omitted in 0..valid.len() {
        let mut values = valid.clone();
        values.remove(omitted);
        assert_eq!(restore_record(values), Err(Denial::Malformed));
    }
    for corrupted in 0..valid.len() {
        let mut values = valid.clone();
        values[corrupted] = if matches!(&values[corrupted], AspectValue::UInt64(_)) {
            string("not-an-integer")
        } else {
            AspectValue::UInt64(17)
        };
        assert_eq!(restore_record(values), Err(Denial::Malformed));
    }
}

#[test]
fn durable_field_restore_rejects_malformed_protocol_and_encoded_bytes() {
    let odd_digest = format!("{}0", "00".repeat(32));
    for (field, invalid) in [
        (0, string("not-a-digest")),
        (0, string(&odd_digest)),
        (3, string("test.effect.v1")),
        (4, AspectValue::UInt64(0)),
        (4, AspectValue::UInt64(u64::from(u32::MAX) + 1)),
        (6, string("not-hex")),
        (6, string("010")),
    ] {
        let mut values = valid_restored_fields();
        values[field] = invalid;
        assert_eq!(restore_record(values), Err(Denial::Malformed));
    }
}

fn valid_restored_fields() -> Vec<AspectValue> {
    vec![
        string(&hex_bytes(&[7; 32])),
        string("family"),
        string("effect"),
        string("test.effect"),
        AspectValue::UInt64(1),
        AspectValue::UInt64(24),
        string("0102"),
        AspectValue::UInt64(9),
        string("notify"),
    ]
}

fn string(value: &str) -> AspectValue {
    AspectValue::String(InternedString::from(value.to_owned()))
}
