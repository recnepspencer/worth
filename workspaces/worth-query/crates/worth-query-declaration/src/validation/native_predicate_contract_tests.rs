use worth_foundational::facade::{
    AspectValue, CanonicalBigInt, CanonicalDate, CanonicalDecimal, CanonicalF32, CanonicalF64,
    CanonicalRational, CanonicalTime, CanonicalTimestamp, CanonicalTimestampTz, ContentRefId,
    EntityId, InternedString, PartitionId, ScalarAspectType, Symbol,
};

use crate::authoring::{
    AspectFieldKey, AspectFieldSelector, AuthoredResultShapeField, DetailQueryBuilder,
    DetailResultShapeBuilder, EqualityPredicate, GuidedAuthoringPath, NativeComparisonPredicate,
    PredicateSelector, RootEntityKey, SetMembershipPredicate,
};
use crate::canonicalization::{CanonicalPredicateEntry, CanonicalPredicateOperand};
use crate::facade::typed::TypedEqualityField;
use crate::schema_view::{QuerySchemaView, SchemaFieldView};
use crate::validation::{validate_canonical_bundle, QueryValidationError};
worth_query_schema! {
    schema NativeTypedSchema("NativeRecord") {
        fields {
            field NullValue("native", "null", Null) => [equality(())];
            field BoolValue("native", "bool", Bool) => [equality(bool)];
            field Int8Value("native", "int8", Int8) => [equality(i8), native_comparable];
            field Int16Value("native", "int16", Int16) => [equality(i16), native_comparable];
            field Int32Value("native", "int32", Int32) => [equality(i32), native_comparable];
            field Int64Value("native", "int64", Int64) => [equality(i64), native_comparable];
            field UInt8Value("native", "uint8", UInt8) => [equality(u8), native_comparable];
            field UInt16Value("native", "uint16", UInt16) => [equality(u16), native_comparable];
            field UInt32Value("native", "uint32", UInt32) => [equality(u32), native_comparable];
            field UInt64Value("native", "uint64", UInt64) => [equality(u64), native_comparable];
            field Float32Value("native", "float32", Float32) => [equality(CanonicalF32), native_comparable];
            field Float64Value("native", "float64", Float64) => [equality(CanonicalF64), native_comparable];
            field DecimalValue("native", "decimal", Decimal) => [equality(CanonicalDecimal), native_comparable];
            field BigIntValue("native", "big-int", BigInt) => [equality(CanonicalBigInt), native_comparable];
            field RationalValue("native", "rational", Rational) => [equality(CanonicalRational), native_comparable];
            field RawStringValue("native", "raw-string", String) => [equality(String)];
            field InternedStringValue("native", "interned-string", String) => [equality(InternedString)];
            field UuidValue("native", "uuid", Uuid) => [equality([u8; 16])];
            field DateValue("native", "date", Date) => [equality(CanonicalDate), native_comparable];
            field TimeValue("native", "time", Time) => [equality(CanonicalTime), native_comparable];
            field TimestampValue("native", "timestamp", Timestamp) => [equality(CanonicalTimestamp), native_comparable];
            field TimestampTzValue("native", "timestamp-tz", TimestampTz) => [equality(CanonicalTimestampTz), native_comparable];
            field EntityRefValue("native", "entity-ref", EntityRef) => [equality(EntityId)];
            field ContentRefValue("native", "content-ref", ContentRef) => [equality(ContentRefId)];
        }
        relations {}
    }
}

#[test]
fn native_typed_schema_exposes_its_schema_view() {
    assert!(NativeTypedSchema::schema_view()
        .has_aspect(&crate::authoring::AspectName::new("native").expect("static aspect name")));
}

#[test]
fn equality_and_membership_preserve_every_exact_native_operand() {
    for value in scalar_samples() {
        let family = value.value_family();
        let equality = EqualityPredicate::from_target_field_key(target(), value.clone());
        assert_exact_operand(PredicateSelector::Equality(equality.clone()), &value);
        assert!(validate(family, |query| query.where_equal(equality)).is_ok());

        let membership =
            SetMembershipPredicate::from_target_field_key(target(), [value.clone()]).unwrap();
        let canonical = CanonicalPredicateEntry::from_authored(&PredicateSelector::SetMembership(
            membership.clone(),
        ));
        let CanonicalPredicateOperand::ScalarSet(values) = canonical.operand else {
            panic!("membership must retain a native scalar set");
        };
        assert_eq!(values.as_slice()[0].as_native(), &value);
        assert!(validate(family, |query| query.where_in(membership)).is_ok());
    }
}

#[test]
fn native_comparison_admits_numeric_and_temporal_families_without_widening() {
    for value in scalar_samples()
        .into_iter()
        .filter(|value| comparable(value.value_family()))
    {
        let family = value.value_family();
        let predicate =
            NativeComparisonPredicate::greater_than_native("native", "value", value.clone())
                .unwrap();
        assert_exact_operand(
            PredicateSelector::NativeComparison(predicate.clone()),
            &value,
        );
        assert!(validate(family, |query| query.where_greater_than(predicate)).is_ok());
    }
}

#[test]
fn incompatible_native_operators_deny_during_schema_validation() {
    let wrong_width = EqualityPredicate::from_target_field_key(target(), AspectValue::Int32(7));
    assert!(matches!(
        validate(ScalarAspectType::Int64, |query| query
            .where_equal(wrong_width)),
        Err(QueryValidationError::IncompatiblePredicateFamily { .. })
    ));

    let reference_comparison = NativeComparisonPredicate::greater_than_native(
        "native",
        "value",
        AspectValue::EntityRef(EntityId::new(PartitionId(1), 2, 3)),
    )
    .unwrap();
    assert!(matches!(
        validate(ScalarAspectType::EntityRef, |query| {
            query.where_greater_than(reference_comparison)
        }),
        Err(QueryValidationError::IncompatiblePredicateFamily { .. })
    ));

    let mixed = SetMembershipPredicate::from_target_field_key(
        target(),
        [AspectValue::UInt64(7), AspectValue::Int64(7)],
    )
    .unwrap();
    assert!(matches!(
        validate(ScalarAspectType::UInt64, |query| query.where_in(mixed)),
        Err(QueryValidationError::IncompatiblePredicateFamily { .. })
    ));
}

#[test]
fn typed_schema_contract_mapping_preserves_every_native_family() {
    let samples = scalar_samples();
    let typed = vec![
        typed::<NullValue>(()),
        typed::<BoolValue>(true),
        typed::<Int8Value>(-7),
        typed::<Int16Value>(-320),
        typed::<Int32Value>(-32_000),
        typed::<Int64Value>(-12),
        typed::<UInt8Value>(7),
        typed::<UInt16Value>(320),
        typed::<UInt32Value>(32_000),
        typed::<UInt64Value>(12),
        typed::<Float32Value>(CanonicalF32::from_f32(1.5)),
        typed::<Float64Value>(CanonicalF64::from_f64(2.5)),
        typed::<DecimalValue>(CanonicalDecimal::new("12.50")),
        typed::<BigIntValue>(CanonicalBigInt::new("-12345678901234567890")),
        typed::<RationalValue>(
            CanonicalRational::new(CanonicalBigInt::new("22"), CanonicalBigInt::new("7")).unwrap(),
        ),
        typed::<RawStringValue>("alpha".to_string()),
        typed::<InternedStringValue>(InternedString::Symbol(Symbol(17))),
        typed::<UuidValue>([7; 16]),
        typed::<DateValue>(CanonicalDate {
            days_from_unix_epoch: 20_000,
        }),
        typed::<TimeValue>(CanonicalTime::new(1_000).unwrap()),
        typed::<TimestampValue>(CanonicalTimestamp {
            micros_since_unix_epoch: 123_456,
        }),
        typed::<TimestampTzValue>(CanonicalTimestampTz {
            utc_micros_since_unix_epoch: 123_456,
            offset_minutes: -360,
        }),
        typed::<EntityRefValue>(EntityId::new(PartitionId(9), 10, 11)),
        typed::<ContentRefValue>(ContentRefId([
            42, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ])),
    ];
    assert_eq!(typed, samples);
}

fn typed<Field: TypedEqualityField>(value: Field::Value) -> AspectValue {
    Field::into_scalar(value).into_native()
}

fn assert_exact_operand(predicate: PredicateSelector, expected: &AspectValue) {
    let canonical = CanonicalPredicateEntry::from_authored(&predicate);
    let CanonicalPredicateOperand::Scalar(actual) = canonical.operand else {
        panic!("predicate must retain one native scalar operand");
    };
    assert_eq!(actual.as_native(), expected);
}

fn validate(
    family: ScalarAspectType,
    configure: impl FnOnce(DetailQueryBuilder) -> DetailQueryBuilder,
) -> Result<crate::validation::ValidatedQueryBundle, QueryValidationError> {
    validate_canonical_bundle(
        canonical_bundle(configure),
        QuerySchemaView::new(
            format!("phase-27-native-{family:?}"),
            [
                field("identity", "id", ScalarAspectType::String),
                field("native", "value", family)
                    .membership_predicate_queryable()
                    .presence_predicate_queryable(),
            ],
            [],
        ),
    )
}

fn field(aspect: &str, name: &str, family: ScalarAspectType) -> SchemaFieldView {
    SchemaFieldView::new(
        crate::authoring::AspectName::new(aspect).unwrap(),
        crate::authoring::FieldName::new(name).unwrap(),
        family,
    )
}

fn canonical_bundle(
    configure: impl FnOnce(DetailQueryBuilder) -> DetailQueryBuilder,
) -> crate::canonicalization::CanonicalQueryBundle {
    let query = configure(
        DetailQueryBuilder::new(RootEntityKey::new("NativeRecord").unwrap())
            .project(AspectFieldSelector::new("identity", "id").unwrap()),
    )
    .build()
    .unwrap();
    let shape = DetailResultShapeBuilder::new()
        .field(AuthoredResultShapeField::new("identity", "id", "id").unwrap())
        .build()
        .unwrap();
    GuidedAuthoringPath::canonicalize_detail(query, shape).unwrap()
}

fn target() -> AspectFieldKey {
    AspectFieldKey::from_authoring_parts("native", "value").unwrap()
}

fn comparable(family: ScalarAspectType) -> bool {
    matches!(
        family,
        ScalarAspectType::Int8
            | ScalarAspectType::Int16
            | ScalarAspectType::Int32
            | ScalarAspectType::Int64
            | ScalarAspectType::UInt8
            | ScalarAspectType::UInt16
            | ScalarAspectType::UInt32
            | ScalarAspectType::UInt64
            | ScalarAspectType::Float32
            | ScalarAspectType::Float64
            | ScalarAspectType::Decimal
            | ScalarAspectType::BigInt
            | ScalarAspectType::Rational
            | ScalarAspectType::Date
            | ScalarAspectType::Time
            | ScalarAspectType::Timestamp
            | ScalarAspectType::TimestampTz
    )
}

fn scalar_samples() -> Vec<AspectValue> {
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
        AspectValue::String(InternedString::Symbol(Symbol(17))),
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
        AspectValue::EntityRef(EntityId::new(PartitionId(9), 10, 11)),
        AspectValue::ContentRef(ContentRefId([
            42, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ])),
    ]
}
