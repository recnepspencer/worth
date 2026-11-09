use super::*;
use worth_foundational::facade::AspectValue;

crate::worth_query_value_binding! {
    NestedBinding for Vec<String> {
        identity: "worth.tests.admitted-value.nested",
        scalar: String,
        canonical: ValueDecodeBounds::new(1024, 128, 8, 8),
    }
}

#[test]
fn admitted_recipe_dispatches_nested_seed_into_a_stack_slot() {
    let recipe = ApplicationFieldBindingRecipe::of::<NestedBinding>(
        ApplicationFieldBindingLocus::new("Entity", "Aspect", "Field"),
    );
    let raw = NestedBinding::encode(&vec!["nondefault".into(), "escaped\ntext".into()]).unwrap();
    let mut total = (0, 0);
    let mut result: Option<Vec<String>> = None;
    recipe.admitted_decode().unwrap()(
        &raw,
        &mut |work, bytes| {
            total.0 += work;
            total.1 += bytes;
            Ok(())
        },
        Some(&mut result),
    )
    .unwrap();
    assert_eq!(result.unwrap(), vec!["nondefault", "escaped\ntext"]);
    assert!(total.0 > 0);
    assert!(total.1 > 0);
}

#[test]
fn an_incorrect_output_slot_refuses_before_the_owner_or_admission_runs() {
    let recipe = ApplicationFieldBindingRecipe::of::<SwallowingBinding>(
        ApplicationFieldBindingLocus::new("Entity", "Aspect", "Field"),
    );
    let mut wrong: Option<String> = None;
    let result = recipe.admitted_decode().unwrap()(
        &AspectValue::UInt64(91),
        &mut |_, _| panic!("wrong slot must refuse first"),
        Some(&mut wrong),
    );
    assert!(matches!(
        result,
        Err(ApplicationValueDecodeDenial::CodecRejected { .. })
    ));
    assert!(wrong.is_none());
}

struct SwallowingBinding;
impl ApplicationScalarValueBinding for SwallowingBinding {
    type Value = u64;
    type Unit = ();
    type Decode = ApplicationValueDecodeAvailable;
    type Identity = ApplicationValueIsNotIdentity;
    type SignedAggregate = ApplicationValueSignedAggregateUnavailable;
    const IDENTITY_NAME: &'static str = "worth.tests.admitted-value.swallowing";
    const SCALAR_FAMILY: worth_foundational::facade::ScalarAspectType =
        worth_foundational::facade::ScalarAspectType::UInt64;
    const SUPPORTS_ADMITTED_DECODE: bool = true;
    fn validate(_: &u64) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
    fn encode(value: &u64) -> Result<AspectValue, ApplicationValueEncodeDenial> {
        Ok(AspectValue::UInt64(*value))
    }
}
impl ApplicationReadableScalarValueBinding for SwallowingBinding {
    fn decode(_: &AspectValue) -> Result<u64, ApplicationValueDecodeDenial> {
        panic!("ordinary decoding cannot stand in for admitted dispatch")
    }
    fn decode_admitted(
        _: &AspectValue,
        admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<u64, ApplicationValueDecodeDenial> {
        let _ = admission.admit(7, 17);
        let _ = admission.admit(1, 1);
        Ok(91)
    }
}

#[test]
fn erased_dispatch_vetoes_a_successful_owner_that_swallowed_admission_refusal() {
    let recipe = ApplicationFieldBindingRecipe::of::<SwallowingBinding>(
        ApplicationFieldBindingLocus::new("Entity", "Aspect", "Field"),
    );
    let mut calls = 0;
    let mut output: Option<u64> = None;
    let result = recipe.admitted_decode().unwrap()(
        &AspectValue::UInt64(91),
        &mut |_, _| {
            calls += 1;
            Err(ValueCodecDenial::Admission)
        },
        Some(&mut output),
    );
    assert!(output.is_none());
    assert!(matches!(
        result,
        Err(ApplicationValueDecodeDenial::Canonical {
            cause: ValueCodecDenial::Admission,
            ..
        })
    ));
    assert_eq!(calls, 1, "latched refusal cannot call the lender again");
}
