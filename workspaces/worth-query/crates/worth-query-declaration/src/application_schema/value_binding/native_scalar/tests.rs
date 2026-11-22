//! Stock native bindings round-trip canonical carriers and refuse the
//! non-canonical ones a public field can still construct.
use super::*;

fn round_trip<Binding>(value: Binding::Value)
where
    Binding: ApplicationReadableScalarValueBinding,
    Binding::Value: PartialEq + std::fmt::Debug,
{
    let encoded = Binding::encode(&value).expect("canonical values encode");
    assert_eq!(encoded.value_family(), Binding::SCALAR_FAMILY);
    assert_eq!(
        Binding::decode(&encoded).expect("canonical values decode"),
        value
    );
}

#[test]
fn canonical_native_values_round_trip_through_their_stock_bindings() {
    round_trip::<CanonicalDateApplicationValueBinding>(CanonicalDate {
        days_from_unix_epoch: i32::MIN,
    });
    round_trip::<CanonicalTimeApplicationValueBinding>(
        CanonicalTime::new(CanonicalTime::NANOS_PER_DAY - 1).unwrap(),
    );
    round_trip::<CanonicalTimestampApplicationValueBinding>(CanonicalTimestamp {
        micros_since_unix_epoch: -1,
    });
    round_trip::<CanonicalTimestampTzApplicationValueBinding>(CanonicalTimestampTz {
        utc_micros_since_unix_epoch: 1,
        offset_minutes: -1_439,
    });
    round_trip::<UuidApplicationValueBinding>([7; 16]);
    round_trip::<CanonicalF32ApplicationValueBinding>(CanonicalF32::from_f32(f32::NAN));
    round_trip::<CanonicalF64ApplicationValueBinding>(CanonicalF64::from_f64(-0.0));
    round_trip::<InlineBytesApplicationValueBinding>(InlineBytes::try_new(vec![0, 255]).unwrap());
}

#[test]
fn non_canonical_carriers_refuse_on_encode_and_decode() {
    let late = CanonicalTime {
        nanos_since_midnight: CanonicalTime::NANOS_PER_DAY,
    };
    assert!(CanonicalTimeApplicationValueBinding::encode(&late).is_err());
    assert!(CanonicalTimeApplicationValueBinding::decode(&AspectValue::Time(late)).is_err());
    let offset = CanonicalTimestampTz {
        utc_micros_since_unix_epoch: 0,
        offset_minutes: 1_440,
    };
    assert!(CanonicalTimestampTzApplicationValueBinding::encode(&offset).is_err());
    assert!(
        CanonicalTimestampTzApplicationValueBinding::decode(&AspectValue::TimestampTz(offset))
            .is_err()
    );
    let payload = CanonicalF64(f64::NAN.to_bits() ^ 1);
    assert!(CanonicalF64ApplicationValueBinding::encode(&payload).is_err());
    assert!(matches!(
        CanonicalDateApplicationValueBinding::decode(&AspectValue::Int32(3)),
        Err(ApplicationValueDecodeDenial::ScalarFamilyMismatch { .. })
    ));
}
