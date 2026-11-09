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

#[test]
fn retained_input_keeps_the_canonical_stream_through_erasure_and_cloning() {
    use crate::facade::application_operation::ApplicationRetainedInput;
    use std::any::Any;
    use std::sync::Arc;
    let encoded = ApplicationEncodedInput::<CorpusBinding>::encode(corpus()).unwrap();
    let identity = *encoded.identity();
    let retained = encoded.retain_canonical_input().unwrap();
    let original = retained.recorded_bytes().to_vec();
    let erased: Arc<dyn Any + Send + Sync> = Arc::new(retained.clone());
    drop(retained);
    let actual = erased
        .downcast_ref::<ApplicationRetainedInput<CorpusBinding>>()
        .unwrap();
    assert_eq!(actual.identity(), &identity);
    assert_eq!(actual.recorded_bytes(), original);
    let restored = ApplicationEncodedInput::<CorpusBinding>::decode_retained(
        actual.recorded_bytes(),
        CanonicalDigestId::new(identity),
        bounds(),
    )
    .unwrap();
    assert_eq!(restored.input(), &corpus());
}

#[test]
fn retained_input_shares_the_existing_canonical_capture() {
    let encoded = ApplicationEncodedInput::<CapturedScalarBinding>::encode(31).unwrap();
    let original = encoded.canonical_input().unwrap().clone();
    let retained = encoded.retain_canonical_input().unwrap();
    assert!(std::ptr::eq(
        original.recorded_bytes(),
        retained.recorded_bytes()
    ));
    assert_eq!(*retained.into_shared_input(), 31);
}

impl<'de> serde::Deserialize<'de> for MutableInput {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        <u64 as serde::Deserialize>::deserialize(deserializer).map(|value| Self(Cell::new(value)))
    }
}
#[test]
fn retained_input_reconstructs_the_snapshot_instead_of_a_later_live_value() {
    let live =
        ApplicationEncodedInput::<MutableBinding>::encode(MutableInput(Cell::new(3))).unwrap();
    let retained = live.clone().retain_canonical_input().unwrap();
    live.input().0.set(5);
    let restored = ApplicationEncodedInput::<MutableBinding>::decode_retained(
        retained.recorded_bytes(),
        CanonicalDigestId::new(*retained.identity()),
        bounds(),
    )
    .unwrap();
    assert_eq!(restored.input().0.get(), 3);
    assert_eq!(live.input().0.get(), 5);
    assert!(matches!(
        live.retain_canonical_input(),
        Err(Denial::IdentityMismatch)
    ));
}
