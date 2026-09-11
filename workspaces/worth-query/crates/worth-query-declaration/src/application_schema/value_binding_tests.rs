use worth_foundational::facade::{AspectValue, ScalarAspectType};

use super::{
    ApplicationEncodedScalarValue, ApplicationFieldBindingLocus, ApplicationFieldBindingRecipe,
    ApplicationReadableScalarValueBinding, ApplicationScalarValueBinding,
    ApplicationStructuredValueBinding, ApplicationValueDecodeAvailable,
    ApplicationValueDecodeDenial, ApplicationValueEncodeDenial, ApplicationValueIsNotIdentity,
    ApplicationValueSignedAggregateUnavailable, ApplicationValueValidationDenial,
    I64ApplicationValueBinding, U64ApplicationValueBinding,
};
use crate::portable_identity::WorthQueryPortableTypeIdentity;

#[derive(Debug, Eq, PartialEq)]
struct PositiveLength(u64);

impl PositiveLength {
    fn get(value: &Self) -> u64 {
        value.0
    }

    fn new(value: u64) -> Option<Self> {
        (value != u64::MAX).then_some(Self(value))
    }
}

fn validate_positive_length(value: &PositiveLength) -> Result<(), &'static str> {
    (value.0 > 0).then_some(()).ok_or("length must be positive")
}

crate::worth_query_value_binding! {
    PositiveLengthBinding for PositiveLength {
        identity: "worth.tests.positive-length.v1",
        scalar: UInt64,
        unit: "worth.units.metre.v1",
        frame: "worth.frames.model-local.v1",
        validate: validate_positive_length,
        encode: PositiveLength::get,
        decode: PositiveLength::new,
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Count(u64);

impl Count {
    fn get(value: &Self) -> u64 {
        value.0
    }

    fn new(value: u64) -> Option<Self> {
        Some(Self(value))
    }
}

crate::worth_query_value_binding! {
    CountBinding for Count {
        identity: "worth.tests.count.v1",
        scalar: UInt64,
        encode: Count::get,
        decode: Count::new,
    }
}

#[test]
fn local_marker_round_trips_query_free_value_with_explicit_metadata() {
    let encoded = PositiveLengthBinding::encode(&PositiveLength(42)).unwrap();
    assert_eq!(encoded, AspectValue::UInt64(42));
    assert_eq!(
        PositiveLengthBinding::decode(&encoded).unwrap(),
        PositiveLength(42)
    );
    assert_eq!(
        PositiveLengthBinding::IDENTITY.as_str(),
        "worth.tests.positive-length.v1"
    );
    assert_eq!(
        PositiveLengthBinding::UNIT.unwrap().as_str(),
        "worth.units.metre.v1"
    );
    assert_eq!(
        PositiveLengthBinding::FRAME.unwrap().as_str(),
        "worth.frames.model-local.v1"
    );
}

#[test]
fn binding_validation_rejects_invalid_values_in_both_directions() {
    let encode_denial = PositiveLengthBinding::encode(&PositiveLength(0)).unwrap_err();
    let ApplicationValueEncodeDenial::Validation(encode_denial) = encode_denial else {
        panic!("invalid domain value bypassed binding validation")
    };
    assert_eq!(encode_denial.reason(), "length must be positive");

    assert!(matches!(
        PositiveLengthBinding::decode(&AspectValue::UInt64(0)),
        Err(ApplicationValueDecodeDenial::Validation(_))
    ));
    assert!(matches!(
        PositiveLengthBinding::decode(&AspectValue::UInt64(u64::MAX)),
        Err(ApplicationValueDecodeDenial::CodecRejected { .. })
    ));
}

#[test]
fn binding_rejects_the_wrong_foundational_scalar_family() {
    let denial = PositiveLengthBinding::decode(&AspectValue::Int64(42)).unwrap_err();
    assert!(matches!(
        denial,
        ApplicationValueDecodeDenial::ScalarFamilyMismatch {
            expected: ScalarAspectType::UInt64,
            observed: ScalarAspectType::Int64,
            ..
        }
    ));
}

#[test]
fn omitted_unit_frame_and_validator_remain_explicitly_absent() {
    assert_eq!(CountBinding::UNIT, None);
    assert_eq!(CountBinding::FRAME, None);
    assert_eq!(
        CountBinding::decode(&CountBinding::encode(&Count(0)).unwrap()).unwrap(),
        Count(0)
    );
}

struct TransferInput {
    amount: u64,
}

struct TransferInputBinding;

impl ApplicationStructuredValueBinding for TransferInputBinding {
    type Value = TransferInput;

    const IDENTITY_NAME: &'static str = "worth.tests.transfer-input.v1";

    fn validate(value: &Self::Value) -> Result<(), ApplicationValueValidationDenial> {
        (value.amount > 0).then_some(()).ok_or_else(|| {
            ApplicationValueValidationDenial::rejected(Self::IDENTITY, "amount must be positive")
        })
    }
}

#[test]
fn structured_binding_validates_without_scalar_coercion() {
    assert_eq!(
        TransferInputBinding::IDENTITY,
        WorthQueryPortableTypeIdentity::declared("worth.tests.transfer-input.v1")
    );
    assert!(TransferInputBinding::validate(&TransferInput { amount: 5 }).is_ok());
    assert!(TransferInputBinding::validate(&TransferInput { amount: 0 }).is_err());
}

#[test]
fn primitive_bindings_have_explicit_capabilities_and_stable_identity() {
    assert_eq!(U64ApplicationValueBinding::IDENTITY_NAME, "worth.rust.u64");
    assert_eq!(
        U64ApplicationValueBinding::decode(&U64ApplicationValueBinding::encode(&42).unwrap())
            .unwrap(),
        42
    );
    assert_eq!(
        <I64ApplicationValueBinding as super::ApplicationSignedAggregateValueBinding>::decode_aggregate(-7)
            .unwrap(),
        -7
    );
}

#[derive(Debug, Eq, PartialEq)]
struct HostileValidatedValue(u64);

struct HostileValidatedValueBinding;

impl ApplicationScalarValueBinding for HostileValidatedValueBinding {
    type Value = HostileValidatedValue;
    type Unit = ();
    type Decode = ApplicationValueDecodeAvailable;
    type Identity = ApplicationValueIsNotIdentity;
    type SignedAggregate = ApplicationValueSignedAggregateUnavailable;

    const IDENTITY_NAME: &'static str = "worth.tests.hostile-validated-value.v1";
    const SCALAR_FAMILY: ScalarAspectType = ScalarAspectType::UInt64;

    fn validate(value: &Self::Value) -> Result<(), ApplicationValueValidationDenial> {
        (value.0 > 0).then_some(()).ok_or_else(|| {
            ApplicationValueValidationDenial::rejected(Self::IDENTITY, "value must be positive")
        })
    }

    fn encode(value: &Self::Value) -> Result<AspectValue, ApplicationValueEncodeDenial> {
        Ok(AspectValue::UInt64(value.0))
    }
}

impl ApplicationReadableScalarValueBinding for HostileValidatedValueBinding {
    fn decode(value: &AspectValue) -> Result<Self::Value, ApplicationValueDecodeDenial> {
        match value {
            AspectValue::UInt64(value) => Ok(HostileValidatedValue(*value)),
            _ => Err(ApplicationValueDecodeDenial::CodecRejected {
                binding_identity: Self::IDENTITY,
            }),
        }
    }
}

#[test]
fn framework_scalar_funnels_enforce_validation_for_hostile_manual_binding() {
    assert!(matches!(
        ApplicationEncodedScalarValue::<HostileValidatedValueBinding>::try_new(
            HostileValidatedValue(0)
        ),
        Err(ApplicationValueEncodeDenial::Validation(_))
    ));

    let recipe = ApplicationFieldBindingRecipe::of::<HostileValidatedValueBinding>(
        ApplicationFieldBindingLocus::new("entity", "aspect", "field"),
    );
    assert!(matches!(
        (recipe.encode())(&HostileValidatedValue(0)),
        Err(ApplicationValueEncodeDenial::Validation(_))
    ));
    assert!(matches!(
        recipe.decode().unwrap()(&AspectValue::UInt64(0)),
        Err(ApplicationValueDecodeDenial::Validation(_))
    ));

    struct Query;
    struct Parameter;
    let parameter = crate::application_query::ApplicationQueryParameterRef::<
        Query,
        Parameter,
        HostileValidatedValueBinding,
    >::from_query_identifier("minimum");
    assert!(matches!(
        crate::application_query::ApplicationQueryParameterSet::new()
            .bind(parameter, HostileValidatedValue(0)),
        Err(ApplicationValueEncodeDenial::Validation(_))
    ));
}
