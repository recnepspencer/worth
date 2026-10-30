use {
    worth_foundational::CanonicalDate, worth_foundational::CanonicalTimestamp,
    worth_foundational::CanonicalTimestampTz, worth_foundational::ContentRefId,
};

#[test]
fn canonical_value_families_preserve_width_precision_and_reference_kind() {
    let rational = worth_foundational::CanonicalRational::new(
        worth_foundational::CanonicalBigInt::new("22"),
        worth_foundational::CanonicalBigInt::new("7"),
    )
    .expect("non-zero denominator");
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
        worth_foundational::AspectValue::Float32(worth_foundational::CanonicalF32::from_f32(1.5)),
        worth_foundational::AspectValue::Float64(worth_foundational::CanonicalF64::from_f64(1.5)),
        worth_foundational::AspectValue::Decimal(worth_foundational::CanonicalDecimal::new(
            "12.30",
        )),
        worth_foundational::AspectValue::BigInt(worth_foundational::CanonicalBigInt::new(
            "12345678901234567890",
        )),
        worth_foundational::AspectValue::Rational(rational),
        worth_foundational::AspectValue::String(worth_foundational::CanonicalString::from("name")),
        worth_foundational::AspectValue::Bytes(ContentRefId(7)),
        worth_foundational::AspectValue::Uuid([1; 16]),
        worth_foundational::AspectValue::Date(CanonicalDate {
            days_from_unix_epoch: 20_000,
        }),
        worth_foundational::AspectValue::Time(
            worth_foundational::CanonicalTime::new(1_000).expect("time in range"),
        ),
        worth_foundational::AspectValue::Timestamp(CanonicalTimestamp {
            micros_since_unix_epoch: 42,
        }),
        worth_foundational::AspectValue::TimestampTz(CanonicalTimestampTz {
            utc_micros_since_unix_epoch: 42,
            offset_minutes: -420,
        }),
        worth_foundational::AspectValue::EntityRef(worth_foundational::EntityId::new(
            worth_foundational::PartitionId::main(),
            9,
            0,
        )),
        worth_foundational::AspectValue::ContentRef(ContentRefId(9)),
    ];

    let families: Vec<_> = values
        .iter()
        .map(worth_foundational::AspectValue::value_family)
        .collect();

    assert_eq!(families.len(), 24);
    assert!(families.contains(&worth_foundational::ScalarAspectType::Int8));
    assert!(families.contains(&worth_foundational::ScalarAspectType::UInt8));
    assert!(families.contains(&worth_foundational::ScalarAspectType::EntityRef));
    assert!(families.contains(&worth_foundational::ScalarAspectType::ContentRef));
    assert_ne!(
        worth_foundational::AspectValue::Bytes(ContentRefId(9)),
        worth_foundational::AspectValue::ContentRef(ContentRefId(9))
    );
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

#[test]
fn string_values_preserve_relational_interning_surface() {
    let raw =
        worth_foundational::AspectValue::String(worth_foundational::CanonicalString::from("name"));
    let symbol = worth_foundational::AspectValue::String(
        worth_foundational::InternedString::Symbol(worth_foundational::Symbol(7)),
    );

    assert_ne!(raw, symbol);
    assert_eq!(
        symbol.value_family(),
        worth_foundational::ScalarAspectType::String
    );
}
