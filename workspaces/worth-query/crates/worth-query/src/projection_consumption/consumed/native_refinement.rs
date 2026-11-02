use worth_foundational::facade::{
    AspectValue, AspectValuePosture, CanonicalBigInt, CanonicalDate, CanonicalDecimal,
    CanonicalF32, CanonicalF64, CanonicalRational, CanonicalTime, CanonicalTimestamp,
    CanonicalTimestampTz, ContentRefId, EntityId, InternedString, ScalarAspectType,
    StructAspectValue,
};

use super::{ConsumedFieldValueFact, ConsumedNativeValueView};
use crate::projection_consumption::{ProjectionFactFieldPath, ProjectionSourceFamily};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConsumedNativeRefinementDenial {
    expected: AspectValuePosture,
    actual: AspectValuePosture,
    field_path: ProjectionFactFieldPath,
    source_family: ProjectionSourceFamily,
    source_identity: String,
    source_row_identity: String,
    projection_authority: String,
    contract_key: Option<worth_foundational::facade::AspectKey>,
    contract_identity: Option<worth_foundational::facade::AspectIdentity>,
    contract_revision: Option<worth_foundational::facade::AspectContractRevision>,
}

impl ConsumedNativeRefinementDenial {
    pub fn expected(&self) -> AspectValuePosture {
        self.expected
    }

    pub fn actual(&self) -> AspectValuePosture {
        self.actual
    }

    pub fn field_path(&self) -> &ProjectionFactFieldPath {
        &self.field_path
    }

    pub fn source_family(&self) -> ProjectionSourceFamily {
        self.source_family
    }

    pub fn source_identity(&self) -> &str {
        &self.source_identity
    }

    pub fn source_row_identity(&self) -> &str {
        &self.source_row_identity
    }

    pub fn projection_authority(&self) -> &str {
        &self.projection_authority
    }

    pub fn contract_key(&self) -> Option<&worth_foundational::facade::AspectKey> {
        self.contract_key.as_ref()
    }

    pub fn contract_identity(&self) -> Option<worth_foundational::facade::AspectIdentity> {
        self.contract_identity
    }

    pub fn contract_revision(&self) -> Option<worth_foundational::facade::AspectContractRevision> {
        self.contract_revision
    }
}

macro_rules! scalar_refinement {
    ($method:ident, $output:ty, $variant:ident, $family:ident) => {
        pub fn $method(&self) -> Result<&$output, ConsumedNativeRefinementDenial> {
            match self.native_value() {
                ConsumedNativeValueView::Scalar(AspectValue::$variant(value)) => Ok(value),
                actual => Err(self.refinement_denial(
                    AspectValuePosture::Scalar(ScalarAspectType::$family),
                    actual,
                )),
            }
        }
    };
}

impl ConsumedFieldValueFact {
    pub fn as_null(&self) -> Result<(), ConsumedNativeRefinementDenial> {
        match self.native_value() {
            ConsumedNativeValueView::Scalar(AspectValue::Null) => Ok(()),
            actual => {
                Err(self
                    .refinement_denial(AspectValuePosture::Scalar(ScalarAspectType::Null), actual))
            }
        }
    }

    scalar_refinement!(as_bool, bool, Bool, Bool);
    scalar_refinement!(as_int8, i8, Int8, Int8);
    scalar_refinement!(as_int16, i16, Int16, Int16);
    scalar_refinement!(as_int32, i32, Int32, Int32);
    scalar_refinement!(as_int64, i64, Int64, Int64);
    scalar_refinement!(as_uint8, u8, UInt8, UInt8);
    scalar_refinement!(as_uint16, u16, UInt16, UInt16);
    scalar_refinement!(as_uint32, u32, UInt32, UInt32);
    scalar_refinement!(as_uint64, u64, UInt64, UInt64);
    scalar_refinement!(as_float32, CanonicalF32, Float32, Float32);
    scalar_refinement!(as_float64, CanonicalF64, Float64, Float64);
    scalar_refinement!(as_decimal, CanonicalDecimal, Decimal, Decimal);
    scalar_refinement!(as_big_int, CanonicalBigInt, BigInt, BigInt);
    scalar_refinement!(as_rational, CanonicalRational, Rational, Rational);
    scalar_refinement!(as_interned_string, InternedString, String, String);
    scalar_refinement!(as_uuid, [u8; 16], Uuid, Uuid);
    scalar_refinement!(as_date, CanonicalDate, Date, Date);
    scalar_refinement!(as_time, CanonicalTime, Time, Time);
    scalar_refinement!(as_timestamp, CanonicalTimestamp, Timestamp, Timestamp);
    scalar_refinement!(
        as_timestamp_tz,
        CanonicalTimestampTz,
        TimestampTz,
        TimestampTz
    );
    scalar_refinement!(as_entity_ref, EntityId, EntityRef, EntityRef);
    scalar_refinement!(as_content_ref, ContentRefId, ContentRef, ContentRef);
    scalar_refinement!(
        as_inline_bytes,
        worth_foundational::InlineBytes,
        InlineBytes,
        InlineBytes
    );

    pub fn as_struct(&self) -> Result<&StructAspectValue, ConsumedNativeRefinementDenial> {
        match self.native_value() {
            ConsumedNativeValueView::Struct(value) => Ok(value),
            actual => Err(self.refinement_denial(AspectValuePosture::Struct, actual)),
        }
    }

    fn refinement_denial(
        &self,
        expected: AspectValuePosture,
        actual: ConsumedNativeValueView<'_>,
    ) -> ConsumedNativeRefinementDenial {
        let native_contract = self.native_contract_context();
        ConsumedNativeRefinementDenial {
            expected,
            actual: match actual {
                ConsumedNativeValueView::Scalar(value) => {
                    AspectValuePosture::Scalar(value.value_family())
                }
                ConsumedNativeValueView::Struct(_) => AspectValuePosture::Struct,
                ConsumedNativeValueView::Absent(posture) => AspectValuePosture::Absent(posture),
            },
            field_path: self.field_path().clone(),
            source_family: self.source_family(),
            source_identity: self.source_identity().to_string(),
            source_row_identity: self.source_row_identity().to_string(),
            projection_authority: self.projection_authority().to_string(),
            contract_key: native_contract.map(|context| context.key().clone()),
            contract_identity: native_contract.map(|context| context.identity()),
            contract_revision: native_contract.map(|context| context.revision()),
        }
    }
}

#[cfg(test)]
mod tests {
    use worth_foundational::facade::{
        AspectValue, AspectValuePosture, CanonicalBigInt, CanonicalDate, CanonicalDecimal,
        CanonicalF32, CanonicalF64, CanonicalRational, CanonicalTime, CanonicalTimestamp,
        CanonicalTimestampTz, ContentRefId, EntityId, FieldKey, InternedString, PartitionId,
        ScalarAspectType, StructAspectValue, Symbol,
    };

    use super::ProjectionSourceFamily;
    use crate::projection_consumption::consumed::ConsumedNativeValue;
    use crate::projection_consumption::contracts::bind_materialized_projection_contract;
    use crate::projection_consumption::{
        declare_projection_consumption, evaluate_projection_consumption_eligibility,
        ConsumedFieldValueFact, ProjectMaterializedFacts, ProjectionConsumptionBindingContext,
        ProjectionConsumptionEligibility, ProjectionConsumptionSource,
    };

    #[test]
    fn borrowed_refinement_preserves_every_foundational_scalar_family() {
        for value in scalar_samples() {
            let fact = scalar_fact(value.clone());
            match value {
                AspectValue::InlineBytes(expected) => {
                    assert_eq!(fact.as_inline_bytes(), Ok(&expected))
                }
                AspectValue::Null => fact.as_null().unwrap(),
                AspectValue::Bool(expected) => assert_eq!(fact.as_bool(), Ok(&expected)),
                AspectValue::Int8(expected) => assert_eq!(fact.as_int8(), Ok(&expected)),
                AspectValue::Int16(expected) => assert_eq!(fact.as_int16(), Ok(&expected)),
                AspectValue::Int32(expected) => assert_eq!(fact.as_int32(), Ok(&expected)),
                AspectValue::Int64(expected) => assert_eq!(fact.as_int64(), Ok(&expected)),
                AspectValue::UInt8(expected) => assert_eq!(fact.as_uint8(), Ok(&expected)),
                AspectValue::UInt16(expected) => assert_eq!(fact.as_uint16(), Ok(&expected)),
                AspectValue::UInt32(expected) => assert_eq!(fact.as_uint32(), Ok(&expected)),
                AspectValue::UInt64(expected) => assert_eq!(fact.as_uint64(), Ok(&expected)),
                AspectValue::Float32(expected) => assert_eq!(fact.as_float32(), Ok(&expected)),
                AspectValue::Float64(expected) => assert_eq!(fact.as_float64(), Ok(&expected)),
                AspectValue::Decimal(expected) => assert_eq!(fact.as_decimal(), Ok(&expected)),
                AspectValue::BigInt(expected) => assert_eq!(fact.as_big_int(), Ok(&expected)),
                AspectValue::Rational(expected) => assert_eq!(fact.as_rational(), Ok(&expected)),
                AspectValue::String(expected) => {
                    assert_eq!(fact.as_interned_string(), Ok(&expected))
                }
                AspectValue::Uuid(expected) => assert_eq!(fact.as_uuid(), Ok(&expected)),
                AspectValue::Date(expected) => assert_eq!(fact.as_date(), Ok(&expected)),
                AspectValue::Time(expected) => assert_eq!(fact.as_time(), Ok(&expected)),
                AspectValue::Timestamp(expected) => {
                    assert_eq!(fact.as_timestamp(), Ok(&expected))
                }
                AspectValue::TimestampTz(expected) => {
                    assert_eq!(fact.as_timestamp_tz(), Ok(&expected))
                }
                AspectValue::EntityRef(expected) => {
                    assert_eq!(fact.as_entity_ref(), Ok(&expected))
                }
                AspectValue::ContentRef(expected) => {
                    assert_eq!(fact.as_content_ref(), Ok(&expected))
                }
            }
        }
    }

    #[test]
    fn refinement_denial_carries_shape_path_source_and_projection_authority() {
        let fact = scalar_fact(AspectValue::UInt32(7));
        let denial = fact.as_int32().unwrap_err();

        assert_eq!(
            denial.expected(),
            AspectValuePosture::Scalar(ScalarAspectType::Int32)
        );
        assert_eq!(
            denial.actual(),
            AspectValuePosture::Scalar(ScalarAspectType::UInt32)
        );
        assert_eq!(
            denial.field_path().terminal_projection_for_boundary(),
            "native.value"
        );
        assert_eq!(
            denial.source_family(),
            ProjectionSourceFamily::QueryReadReceipt
        );
        assert_eq!(denial.source_identity(), "read-graph:test");
        assert_eq!(denial.source_row_identity(), "row:test");
        assert!(!denial.projection_authority().is_empty());

        let structured = StructAspectValue::new([(
            FieldKey::new("label").unwrap(),
            AspectValue::String("native".into()),
        )])
        .unwrap();
        let structured_fact = ConsumedFieldValueFact::new_native(
            &contract(),
            "row:test",
            field_path(),
            ConsumedNativeValue::struct_value(structured.clone()),
        );
        assert_eq!(structured_fact.as_struct(), Ok(&structured));
        assert_eq!(
            structured_fact.as_uint32().unwrap_err().actual(),
            AspectValuePosture::Struct
        );
    }

    fn scalar_fact(value: AspectValue) -> ConsumedFieldValueFact {
        ConsumedFieldValueFact::new(&contract(), "row:test", field_path(), value)
    }

    fn contract() -> crate::projection_consumption::MaterializedProjectionContract {
        let source = ProjectionConsumptionSource::test_only(
            ProjectionSourceFamily::QueryReadReceipt,
            Some("query:test"),
            Some("basis:test"),
            Some("result:test"),
            Some("result-shape:test"),
            "read-graph:test",
        );
        let binding = ProjectionConsumptionBindingContext::test_only(
            "result-shape:test",
            "authorized-projection:test",
            crate::projection_consumption::test_authorized_field_paths(&["native.value"]),
        );
        let declaration = declare_projection_consumption(
            source,
            binding,
            ProjectMaterializedFacts::declare().display_field_path(field_path()),
        )
        .unwrap();
        let ProjectionConsumptionEligibility::Admitted(admitted) =
            evaluate_projection_consumption_eligibility(&declaration)
        else {
            panic!("native refinement fixture must admit")
        };
        bind_materialized_projection_contract(&admitted)
    }

    fn field_path() -> crate::projection_consumption::ProjectionFactFieldPath {
        crate::projection_consumption::projection_fact_field_path_from_segments([
            FieldKey::new("native").unwrap(),
            FieldKey::new("value").unwrap(),
        ])
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
                CanonicalRational::new(CanonicalBigInt::new("22"), CanonicalBigInt::new("7"))
                    .unwrap(),
            ),
            AspectValue::String(InternedString::Raw("alpha".into())),
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
                42, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0,
            ])),
            AspectValue::InlineBytes(
                worth_foundational::InlineBytes::try_new(vec![0, 128, 255]).unwrap(),
            ),
            AspectValue::String(InternedString::Symbol(Symbol(17))),
        ]
    }
}
