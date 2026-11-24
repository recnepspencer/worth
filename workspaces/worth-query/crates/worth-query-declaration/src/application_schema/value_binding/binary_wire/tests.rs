use super::*;

struct VecWriter(Vec<u8>);
impl BinaryValueWriter for VecWriter {
    fn write(&mut self, bytes: &[u8]) -> Result<(), ValueCodecDenial> {
        self.0.extend_from_slice(bytes);
        Ok(())
    }
}

fn round_trip<T: WorthBinaryWireField + PartialEq + std::fmt::Debug>(value: T) {
    let mut writer = VecWriter(Vec::new());
    value.encode_binary_field(&mut writer).unwrap();
    assert_eq!(writer.0.len(), T::WIRE_BYTES);
    let decoded = T::decode_binary_field(&writer.0, &mut |_, _| Ok(())).unwrap();
    assert_eq!(decoded, value);
}

#[test]
fn canonical_wire_fields_round_trip_exact_widths() {
    round_trip(true);
    round_trip(false);
    round_trip(0xABu8);
    round_trip(0x1234u16);
    round_trip(0x1234_5678u32);
    round_trip(0x0102_0304_0506_0708u64);
    round_trip(-42i8);
    round_trip(-1024i16);
    round_trip(-100_000i32);
    round_trip(-9_000_000_000i64);
    round_trip([7u8; 16]);
    round_trip(CanonicalDigestId::new([0x5Au8; 32]));
    round_trip(CanonicalDate {
        days_from_unix_epoch: -365,
    });
    round_trip(CanonicalTime::new(12_345_678_900).unwrap());
    round_trip(CanonicalTimestamp {
        micros_since_unix_epoch: -1_700_000_000_000_000,
    });
    round_trip(CanonicalTimestampTz {
        utc_micros_since_unix_epoch: 1_700_000_000_000_000,
        offset_minutes: -420,
    });
    round_trip(CanonicalF32::from_f32(-3.25));
    round_trip(CanonicalF64::from_f64(-123.5));
    round_trip(CanonicalF64::from_f64(f64::NAN));
}

#[test]
fn non_canonical_and_malformed_wire_fields_refuse() {
    let mut admit = |_: usize, _: usize| Ok(());
    assert_eq!(
        bool::decode_binary_field(&[2], &mut admit),
        Err(ValueCodecDenial::NonCanonical)
    );
    assert_eq!(
        bool::decode_binary_field(&[], &mut admit),
        Err(ValueCodecDenial::Malformed)
    );

    let bad_time = CanonicalTime {
        nanos_since_midnight: CanonicalTime::NANOS_PER_DAY,
    };
    let mut w = VecWriter(Vec::new());
    assert_eq!(
        bad_time.encode_binary_field(&mut w),
        Err(ValueCodecDenial::NonCanonical)
    );
    assert_eq!(
        CanonicalTime::decode_binary_field(&CanonicalTime::NANOS_PER_DAY.to_le_bytes(), &mut admit),
        Err(ValueCodecDenial::NonCanonical)
    );

    let bad_tz = CanonicalTimestampTz {
        utc_micros_since_unix_epoch: 0,
        offset_minutes: 1_440,
    };
    assert_eq!(
        bad_tz.encode_binary_field(&mut w),
        Err(ValueCodecDenial::NonCanonical)
    );
    let mut tz_bytes = [0u8; 12];
    tz_bytes[8..12].copy_from_slice(&(-1_440i32).to_le_bytes());
    assert_eq!(
        CanonicalTimestampTz::decode_binary_field(&tz_bytes, &mut admit),
        Err(ValueCodecDenial::NonCanonical)
    );

    let non_canon_f32 = CanonicalF32(f32::NAN.to_bits() | 1);
    assert_eq!(
        non_canon_f32.encode_binary_field(&mut w),
        Err(ValueCodecDenial::NonCanonical)
    );
    assert_eq!(
        CanonicalF32::decode_binary_field(&non_canon_f32.0.to_le_bytes(), &mut admit),
        Err(ValueCodecDenial::NonCanonical)
    );

    let non_canon_f64 = CanonicalF64(f64::NAN.to_bits() | 1);
    assert_eq!(
        non_canon_f64.encode_binary_field(&mut w),
        Err(ValueCodecDenial::NonCanonical)
    );
    assert_eq!(
        CanonicalF64::decode_binary_field(&non_canon_f64.0.to_le_bytes(), &mut admit),
        Err(ValueCodecDenial::NonCanonical)
    );
}
