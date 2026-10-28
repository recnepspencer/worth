use worth_foundational::{
    AspectValue, CanonicalBigInt, CanonicalDate, CanonicalDecimal, CanonicalF32, CanonicalF64,
    CanonicalRational, CanonicalTime, CanonicalTimestamp, CanonicalTimestampTz, ContentRefId,
    EntityId, PartitionId, ScalarAspectType,
};

#[test]
fn value_family_golden_covers_every_milestone_1_value_variant() {
    let values = vec![
        AspectValue::Null,
        AspectValue::Bool(true),
        AspectValue::Int8(-8),
        AspectValue::Int16(-16),
        AspectValue::Int32(-32),
        AspectValue::Int64(-64),
        AspectValue::UInt8(8),
        AspectValue::UInt16(16),
        AspectValue::UInt32(32),
        AspectValue::UInt64(64),
        AspectValue::Float32(CanonicalF32::from_bits(0x3f80_0000)),
        AspectValue::Float64(CanonicalF64::from_bits(0x3ff0_0000_0000_0000)),
        AspectValue::Decimal(CanonicalDecimal::new("12.34")),
        AspectValue::BigInt(CanonicalBigInt::new("12345678901234567890")),
        AspectValue::Rational(
            CanonicalRational::new(CanonicalBigInt::new("22"), CanonicalBigInt::new("7"))
                .expect("non-zero denominator"),
        ),
        AspectValue::String("Ada".into()),
        AspectValue::Bytes(ContentRefId(77)),
        AspectValue::Uuid([1; 16]),
        AspectValue::Date(CanonicalDate {
            days_from_unix_epoch: 20_000,
        }),
        AspectValue::Time(CanonicalTime::new(42).expect("valid time")),
        AspectValue::Timestamp(CanonicalTimestamp {
            micros_since_unix_epoch: 123,
        }),
        AspectValue::TimestampTz(CanonicalTimestampTz {
            utc_micros_since_unix_epoch: 123,
            offset_minutes: -420,
        }),
        AspectValue::EntityRef(EntityId::new(PartitionId::main(), 9, 1)),
        AspectValue::ContentRef(ContentRefId(88)),
    ];

    let families: Vec<_> = values.iter().map(AspectValue::value_family).collect();

    assert_eq!(
        families,
        vec![
            ScalarAspectType::Null,
            ScalarAspectType::Bool,
            ScalarAspectType::Int8,
            ScalarAspectType::Int16,
            ScalarAspectType::Int32,
            ScalarAspectType::Int64,
            ScalarAspectType::UInt8,
            ScalarAspectType::UInt16,
            ScalarAspectType::UInt32,
            ScalarAspectType::UInt64,
            ScalarAspectType::Float32,
            ScalarAspectType::Float64,
            ScalarAspectType::Decimal,
            ScalarAspectType::BigInt,
            ScalarAspectType::Rational,
            ScalarAspectType::String,
            ScalarAspectType::Bytes,
            ScalarAspectType::Uuid,
            ScalarAspectType::Date,
            ScalarAspectType::Time,
            ScalarAspectType::Timestamp,
            ScalarAspectType::TimestampTz,
            ScalarAspectType::EntityRef,
            ScalarAspectType::ContentRef,
        ]
    );
}
