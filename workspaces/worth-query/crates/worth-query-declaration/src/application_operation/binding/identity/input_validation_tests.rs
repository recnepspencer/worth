//! Typed inputs are validated before any identity or preencoded carrier exists.
use super::*;
use crate::application_schema::ApplicationValueValidationDenial;
use serde::{Serialize, Serializer};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

struct Input {
    value: u64,
    serialized: Arc<AtomicUsize>,
}
impl Serialize for Input {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.serialized.fetch_add(1, Ordering::SeqCst);
        serializer.serialize_u64(self.value)
    }
}
struct Binding;
impl ApplicationStructuredValueBinding for Binding {
    type Value = Input;
    const IDENTITY_NAME: &'static str = "worth.input-validation.positive.v1";
    fn validate(input: &Input) -> Result<(), ApplicationValueValidationDenial> {
        if input.value == 0 {
            Err(ApplicationValueValidationDenial::rejected(
                Self::IDENTITY,
                "value must be positive",
            ))
        } else {
            Ok(())
        }
    }
}

#[test]
fn ordinary_and_preencoded_inputs_reject_before_serializing_invalid_values() {
    let serialized = Arc::new(AtomicUsize::new(0));
    let invalid = Input {
        value: 0,
        serialized: serialized.clone(),
    };
    assert!(matches!(
        ApplicationEncodedInput::<Binding>::encode(invalid),
        Err(ApplicationValueEncodeDenial::Validation(_))
    ));
    assert_eq!(serialized.load(Ordering::SeqCst), 0);
    let invalid = Input {
        value: 0,
        serialized: serialized.clone(),
    };
    let mut admit = |_| Ok::<(), std::convert::Infallible>(());
    assert!(matches!(
        input_identity_admitted::<Binding, _, _>(&invalid, &mut admit),
        Err(ApplicationMutationIdentityAdmittedDenial::Input(
            ApplicationValueEncodeDenial::Validation(_)
        ))
    ));
    assert_eq!(serialized.load(Ordering::SeqCst), 0);
    let valid = Input {
        value: 1,
        serialized,
    };
    let expected = canonical_identity(INPUT_DOMAIN, Binding::IDENTITY.as_str(), &valid).unwrap();
    let ordinary = input_identity::<Binding>(&valid).unwrap();
    let admitted = input_identity_admitted::<Binding, _, _>(&valid, &mut admit).unwrap();
    assert_eq!(ordinary.identity(), expected.identity());
    assert_eq!(admitted.identity(), expected.identity());
    assert_eq!(ordinary.work(), admitted.work());
}
