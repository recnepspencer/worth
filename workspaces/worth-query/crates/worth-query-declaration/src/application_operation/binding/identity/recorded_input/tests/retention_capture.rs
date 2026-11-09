//! Mutable callers cannot relabel a later value as the admitted original.
use super::*;
use crate::facade::application_schema::{
    ApplicationStructuredValueBinding, ApplicationValueValidationDenial,
};
use std::cell::Cell;
struct MutableInput(Cell<u64>);
impl serde::Serialize for MutableInput {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serde::Serialize::serialize(&self.0.get(), serializer)
    }
}
struct MutableBinding;
impl ApplicationStructuredValueBinding for MutableBinding {
    type Value = MutableInput;
    const IDENTITY_NAME: &'static str = "worth.recorded-input.mutable.v1";
    fn validate(_: &Self::Value) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
}
#[test]
fn retention_capture_refuses_input_changed_after_its_identity_was_encoded() {
    let input =
        ApplicationEncodedInput::<MutableBinding>::encode(MutableInput(Cell::new(3))).unwrap();
    input.input().0.set(5);
    assert!(matches!(
        input.capture_for_retention(),
        Err(Denial::IdentityMismatch)
    ));
}
