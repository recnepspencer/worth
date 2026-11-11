//! The declaration factory constructs DATA; it cannot substitute its own identity.
use super::*;
use worth_query_declaration::facade::application_schema::ApplicationValueValidationDenial;
struct Number;
impl ApplicationStructuredValueBinding for Number {
    type Value = u64;
    const IDENTITY_NAME: &'static str = "worth.correction-input.number.v1";
    fn validate(_: &u64) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
}
#[test]
fn recorded_correction_input_factory_cannot_replace_the_retained_value() {
    let retained = ApplicationEncodedInput::<Number>::encode(5)
        .unwrap()
        .retain_canonical_input()
        .unwrap();
    let stopped = recover(&retained, |_, _, _| {
        ApplicationEncodedInput::<Number>::encode(8)
            .map_err(ApplicationRetainedInputDenial::Encoding)
    });
    assert!(matches!(
        stopped,
        Err(Denial::OriginalInputDecode(
            ApplicationRetainedInputDenial::IdentityMismatch
        ))
    ));
}
#[test]
fn recorded_correction_input_factory_keeps_its_native_refusal() {
    let retained = ApplicationEncodedInput::<Number>::encode(5)
        .unwrap()
        .retain_canonical_input()
        .unwrap();
    assert!(matches!(
        recover(&retained, |_, _, _| Err(
            ApplicationRetainedInputDenial::BindingMismatch
        )),
        Err(Denial::OriginalInputDecode(
            ApplicationRetainedInputDenial::BindingMismatch
        ))
    ));
}
#[test]
fn recorded_correction_input_is_reconstructed_from_the_exact_snapshot() {
    let retained = ApplicationEncodedInput::<Number>::encode(5)
        .unwrap()
        .retain_canonical_input()
        .unwrap();
    let restored = recover(
        &retained,
        ApplicationEncodedInput::<Number>::decode_retained,
    )
    .unwrap();
    assert_eq!(*restored.input(), 5);
    assert_eq!(restored.identity(), retained.identity());
}
