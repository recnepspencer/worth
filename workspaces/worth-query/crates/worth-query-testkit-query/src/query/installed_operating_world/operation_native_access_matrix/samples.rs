pub use worth_foundational::facade::{
    AbsenceLaw, AspectContract, AspectContractRevision, AspectEvolutionPolicy, AspectIdentity,
    AspectKey, AspectValue, CanonicalBigInt, CanonicalDate, CanonicalDecimal, CanonicalF32,
    CanonicalF64, CanonicalRational, CanonicalTime, CanonicalTimestamp, CanonicalTimestampTz,
    ContentRefId, FieldDeclaration, FieldKey, FieldRequirement, InternedString, PartitionId,
    ScalarAspectType, StructAspectShape, StructAspectValue, Symbol,
};

pub const MATRIX_ASPECT: &str = "native.matrix";

pub fn matrix_contract(revision: u64) -> AspectContract {
    matrix_contract_with_override(revision, None)
}

pub fn matrix_contract_with_override(
    revision: u64,
    scalar_override: Option<(usize, ScalarAspectType)>,
) -> AspectContract {
    let required = scalar_samples()
        .into_iter()
        .enumerate()
        .map(|(index, value)| {
            field_declaration(
                &format!("f{index:02}"),
                scalar_override
                    .filter(|(field, _)| *field == index)
                    .map_or(value.value_family(), |(_, family)| family),
                AbsenceLaw::Required,
            )
        });
    AspectContract::struct_aspect(
        matrix_aspect_key(),
        AspectIdentity(0x9150_1001),
        AspectContractRevision(revision),
        StructAspectShape::new(required.chain([
            field_declaration("optional", ScalarAspectType::String, AbsenceLaw::Optional),
            field_declaration("defaulted", ScalarAspectType::String, AbsenceLaw::Defaulted),
        ]))
        .unwrap(),
    )
}

pub fn matrix_value(row: u64) -> StructAspectValue {
    StructAspectValue::new(
        scalar_samples()
            .into_iter()
            .enumerate()
            .map(|(index, value)| (sample_field(index), row_variant(value, row))),
    )
    .unwrap()
}

pub fn matrix_value_with_order(row: u64, order: &str) -> StructAspectValue {
    let order_field = sample_field(15);
    StructAspectValue::new(matrix_value(row).fields().map(|(field, value)| {
        let value = if field == &order_field {
            AspectValue::String(InternedString::Raw(order.to_string()))
        } else {
            value.clone()
        };
        (field.clone(), value)
    }))
    .unwrap()
}

pub fn scalar_samples() -> Vec<AspectValue> {
    vec![
        AspectValue::Null,
        AspectValue::Bool(true),
        AspectValue::Int8(-7),
        AspectValue::Int16(-320),
        AspectValue::Int32(-32_000),
        AspectValue::Int64(-12),
        AspectValue::UInt8(7),
        AspectValue::UInt16(320),
        AspectValue::UInt32(32_000),
        AspectValue::UInt64(12),
        AspectValue::Float32(CanonicalF32::from_f32(1.5)),
        AspectValue::Float64(CanonicalF64::from_f64(2.5)),
        AspectValue::Decimal(CanonicalDecimal::new("12.50")),
        AspectValue::BigInt(CanonicalBigInt::new("-12345678901234567890")),
        AspectValue::Rational(
            CanonicalRational::new(CanonicalBigInt::new("22"), CanonicalBigInt::new("7")).unwrap(),
        ),
        AspectValue::String(InternedString::Raw("alpha".into())),
        AspectValue::Bytes(ContentRefId(41)),
        AspectValue::Uuid([7; 16]),
        AspectValue::Date(CanonicalDate {
            days_from_unix_epoch: 20_000,
        }),
        AspectValue::Time(CanonicalTime::new(1_000).unwrap()),
        AspectValue::Timestamp(CanonicalTimestamp {
            micros_since_unix_epoch: 123_456,
        }),
        AspectValue::TimestampTz(CanonicalTimestampTz {
            utc_micros_since_unix_epoch: 123_456,
            offset_minutes: -360,
        }),
        AspectValue::EntityRef(worth_foundational::facade::EntityId::new(
            PartitionId(9),
            10,
            11,
        )),
        AspectValue::ContentRef(ContentRefId(42)),
        AspectValue::String(InternedString::Symbol(Symbol(17))),
    ]
}

pub fn matrix_aspect_key() -> AspectKey {
    AspectKey::new(MATRIX_ASPECT).unwrap()
}

pub fn sample_field(index: usize) -> FieldKey {
    FieldKey::new(format!("f{index:02}")).unwrap()
}

pub fn optional_field() -> FieldKey {
    FieldKey::new("optional").unwrap()
}

pub fn defaulted_field() -> FieldKey {
    FieldKey::new("defaulted").unwrap()
}

pub fn field_declaration(
    key: &str,
    family: ScalarAspectType,
    absence: AbsenceLaw,
) -> FieldDeclaration {
    let requirement = match absence {
        AbsenceLaw::Required => FieldRequirement::Required,
        AbsenceLaw::Optional => FieldRequirement::Optional,
        AbsenceLaw::Defaulted => FieldRequirement::Defaulted,
    };
    FieldDeclaration::new(
        FieldKey::new(key).unwrap(),
        family,
        requirement,
        absence,
        AspectEvolutionPolicy::ExplicitBreakRequired,
    )
    .unwrap()
}

pub fn row_variant(value: AspectValue, row: u64) -> AspectValue {
    match value {
        AspectValue::String(InternedString::Raw(value)) => {
            AspectValue::String(InternedString::Raw(format!("{value}-{row}")))
        }
        other => other,
    }
}
