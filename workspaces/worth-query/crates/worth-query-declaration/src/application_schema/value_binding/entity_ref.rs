//! Full physical coordinates are descriptive values, never entity authority.
use super::*;
use worth_foundational::facade::{AspectValue, EntityId, ScalarAspectType};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EntityRefApplicationValueBinding;

impl ApplicationScalarValueBinding for EntityRefApplicationValueBinding {
    type Value = EntityId;
    type Unit = ();
    type Decode = ApplicationValueDecodeAvailable;
    type Identity = ApplicationValueIsNotIdentity;
    type SignedAggregate = ApplicationValueSignedAggregateUnavailable;
    const IDENTITY_NAME: &'static str = "worth.native.entity-ref.v1";
    const SCALAR_FAMILY: ScalarAspectType = ScalarAspectType::EntityRef;

    fn validate(_: &EntityId) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }
    fn encode(value: &EntityId) -> Result<AspectValue, ApplicationValueEncodeDenial> {
        Ok(AspectValue::EntityRef(*value))
    }
}

impl ApplicationReadableScalarValueBinding for EntityRefApplicationValueBinding {
    fn decode(value: &AspectValue) -> Result<EntityId, ApplicationValueDecodeDenial> {
        match value {
            AspectValue::EntityRef(value) => Ok(*value),
            value => Err(ApplicationValueDecodeDenial::ScalarFamilyMismatch {
                binding_identity: Self::IDENTITY,
                expected: Self::SCALAR_FAMILY,
                observed: value.value_family(),
            }),
        }
    }
}
