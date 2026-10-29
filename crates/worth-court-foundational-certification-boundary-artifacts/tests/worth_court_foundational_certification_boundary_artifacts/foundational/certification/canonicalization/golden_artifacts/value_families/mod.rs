use {
    worth_foundational::AspectValue, worth_foundational::CanonicalBigInt,
    worth_foundational::CanonicalDate, worth_foundational::CanonicalDecimal,
    worth_foundational::CanonicalF32, worth_foundational::CanonicalF64,
    worth_foundational::CanonicalRational, worth_foundational::CanonicalTime,
    worth_foundational::CanonicalTimestamp, worth_foundational::CanonicalTimestampTz,
    worth_foundational::ContentRefId, worth_foundational::EntityId,
    worth_foundational::PartitionId, worth_foundational::ScalarAspectType,
};

#[test]
fn value_family_golden_covers_every_value_variant() {
    let values = vec![
        worth_foundational::AspectValue::Null,
        worth_foundational::AspectValue::Bool(true),
        worth_foundational::AspectValue::Int8(-8),
        worth_foundational::AspectValue::Int16(-16),
        worth_foundational::AspectValue::Int32(-32),
        worth_foundational::AspectValue::Int64(-64),
        worth_foundational::AspectValue::UInt8(8),
        worth_foundational::AspectValue::UInt16(16),
        worth_foundational::AspectValue::UInt32(32),
        worth_foundational::AspectValue::UInt64(64),
        worth_foundational::AspectValue::Float32(worth_foundational::CanonicalF32::from_bits(
            0x3f80_0000,
        )),
        worth_foundational::AspectValue::Float64(worth_foundational::CanonicalF64::from_bits(
            0x3ff0_0000_0000_0000,
        )),
        worth_foundational::AspectValue::Decimal(worth_foundational::CanonicalDecimal::new(
            "12.34",
        )),
        worth_foundational::AspectValue::BigInt(worth_foundational::CanonicalBigInt::new(
            "12345678901234567890",
        )),
        worth_foundational::AspectValue::Rational(
            worth_foundational::CanonicalRational::new(
                worth_foundational::CanonicalBigInt::new("22"),
                worth_foundational::CanonicalBigInt::new("7"),
            )
            .expect("non-zero denominator"),
        ),
        worth_foundational::AspectValue::String("Ada".into()),
        worth_foundational::AspectValue::Bytes(ContentRefId(77)),
        worth_foundational::AspectValue::Uuid([1; 16]),
        worth_foundational::AspectValue::Date(CanonicalDate {
            days_from_unix_epoch: 20_000,
        }),
        worth_foundational::AspectValue::Time(
            worth_foundational::CanonicalTime::new(42).expect("valid time"),
        ),
        worth_foundational::AspectValue::Timestamp(CanonicalTimestamp {
            micros_since_unix_epoch: 123,
        }),
        worth_foundational::AspectValue::TimestampTz(CanonicalTimestampTz {
            utc_micros_since_unix_epoch: 123,
            offset_minutes: -420,
        }),
        worth_foundational::AspectValue::EntityRef(worth_foundational::EntityId::new(
            worth_foundational::PartitionId::main(),
            9,
            1,
        )),
        worth_foundational::AspectValue::ContentRef(ContentRefId(88)),
    ];

    let families: Vec<_> = values
        .iter()
        .map(worth_foundational::AspectValue::value_family)
        .collect();

    assert_eq!(
        families,
        vec![
            worth_foundational::ScalarAspectType::Null,
            worth_foundational::ScalarAspectType::Bool,
            worth_foundational::ScalarAspectType::Int8,
            worth_foundational::ScalarAspectType::Int16,
            worth_foundational::ScalarAspectType::Int32,
            worth_foundational::ScalarAspectType::Int64,
            worth_foundational::ScalarAspectType::UInt8,
            worth_foundational::ScalarAspectType::UInt16,
            worth_foundational::ScalarAspectType::UInt32,
            worth_foundational::ScalarAspectType::UInt64,
            worth_foundational::ScalarAspectType::Float32,
            worth_foundational::ScalarAspectType::Float64,
            worth_foundational::ScalarAspectType::Decimal,
            worth_foundational::ScalarAspectType::BigInt,
            worth_foundational::ScalarAspectType::Rational,
            worth_foundational::ScalarAspectType::String,
            worth_foundational::ScalarAspectType::Bytes,
            worth_foundational::ScalarAspectType::Uuid,
            worth_foundational::ScalarAspectType::Date,
            worth_foundational::ScalarAspectType::Time,
            worth_foundational::ScalarAspectType::Timestamp,
            worth_foundational::ScalarAspectType::TimestampTz,
            worth_foundational::ScalarAspectType::EntityRef,
            worth_foundational::ScalarAspectType::ContentRef,
        ]
    );
}
