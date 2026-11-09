//! Distinct content value forms reuse one scalar codec; each format lives in its binding.
use super::*;
use std::marker::PhantomData;
use worth_foundational::facade::{AspectValue, ScalarAspectType};
#[derive(Clone, Copy, Debug)]
pub struct BoundReference<C>(pub ContentRefId, PhantomData<fn() -> C>);
impl<C> BoundReference<C> {
    pub fn new(reference: ContentRefId) -> Self {
        Self(reference, PhantomData)
    }
}
pub struct ReferenceBinding<C>(PhantomData<fn() -> C>);
impl<C: ApplicationContentBinding> ApplicationScalarValueBinding for ReferenceBinding<C> {
    type Value = BoundReference<C>;
    type Unit = ();
    type Decode = ApplicationValueDecodeAvailable;
    type Identity = ApplicationValueIsNotIdentity;
    type SignedAggregate = ApplicationValueSignedAggregateUnavailable;
    const IDENTITY_NAME: &'static str = ContentRefApplicationValueBinding::IDENTITY_NAME;
    const SCALAR_FAMILY: ScalarAspectType = ScalarAspectType::ContentRef;
    const SUPPORTS_ADMITTED_DECODE: bool = true;
    const NATIVE_INPUT_ADMISSION: Option<ApplicationNativeInputDecodeProfile> =
        ContentRefApplicationValueBinding::NATIVE_INPUT_ADMISSION;
    fn validate(value: &Self::Value) -> Result<(), ApplicationValueValidationDenial> {
        ContentRefApplicationValueBinding::validate(&value.0)
    }
    fn encode(value: &Self::Value) -> Result<AspectValue, ApplicationValueEncodeDenial> {
        ContentRefApplicationValueBinding::encode(&value.0)
    }
}
impl<C: ApplicationContentBinding> ApplicationReadableScalarValueBinding for ReferenceBinding<C> {
    fn decode(value: &AspectValue) -> Result<Self::Value, ApplicationValueDecodeDenial> {
        ContentRefApplicationValueBinding::decode(value).map(BoundReference::new)
    }
    fn decode_admitted(
        value: &AspectValue,
        admission: &mut ValueDecodeAdmission<'_>,
    ) -> Result<Self::Value, ApplicationValueDecodeDenial> {
        ContentRefApplicationValueBinding::decode_admitted(value, admission)
            .map(BoundReference::new)
    }
}
impl<C: ApplicationContentBinding> ApplicationContentReferenceBinding for ReferenceBinding<C> {
    fn from_reference(reference: ContentRefId) -> Self::Value {
        BoundReference::new(reference)
    }
    fn reference(value: &Self::Value) -> ContentRefId {
        value.0
    }
}
