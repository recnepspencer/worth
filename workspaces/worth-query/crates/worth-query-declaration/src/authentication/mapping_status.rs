use worth_foundational::facade::{AspectValue, ScalarAspectType};

use crate::application_schema::{
    ApplicationIdentityScalarValueBinding, ApplicationReadableScalarValueBinding,
    ApplicationScalarValueBinding, ApplicationValueDecodeAvailable, ApplicationValueDecodeDenial,
    ApplicationValueEncodeDenial, ApplicationValueIsIdentity,
    ApplicationValueSignedAggregateUnavailable, ApplicationValueValidationDenial,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum WorthQueryPrincipalMappingStatus {
    Enabled,
    Disabled,
}
crate::worth_query_portable_type!(
    WorthQueryPrincipalMappingStatus => "worth.query.principal_mapping_status.v1"
);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthQueryPrincipalMappingStatusBinding;

impl ApplicationScalarValueBinding for WorthQueryPrincipalMappingStatusBinding {
    type Value = WorthQueryPrincipalMappingStatus;
    type Unit = ();
    type Decode = ApplicationValueDecodeAvailable;
    type Identity = ApplicationValueIsIdentity;
    type SignedAggregate = ApplicationValueSignedAggregateUnavailable;

    const IDENTITY_NAME: &'static str = "worth.query.principal_mapping_status.v1";
    const SCALAR_FAMILY: ScalarAspectType = ScalarAspectType::Bool;

    fn validate(_: &Self::Value) -> Result<(), ApplicationValueValidationDenial> {
        Ok(())
    }

    fn encode(value: &Self::Value) -> Result<AspectValue, ApplicationValueEncodeDenial> {
        Ok(AspectValue::Bool(matches!(value, Self::Value::Enabled)))
    }
}

impl ApplicationReadableScalarValueBinding for WorthQueryPrincipalMappingStatusBinding {
    fn decode(value: &AspectValue) -> Result<Self::Value, ApplicationValueDecodeDenial> {
        match value {
            AspectValue::Bool(true) => Ok(Self::Value::Enabled),
            AspectValue::Bool(false) => Ok(Self::Value::Disabled),
            value => Err(ApplicationValueDecodeDenial::ScalarFamilyMismatch {
                binding_identity: Self::IDENTITY,
                expected: Self::SCALAR_FAMILY,
                observed: value.value_family(),
            }),
        }
    }
}

impl ApplicationIdentityScalarValueBinding for WorthQueryPrincipalMappingStatusBinding {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binding_round_trips_both_statuses() {
        for status in [
            WorthQueryPrincipalMappingStatus::Enabled,
            WorthQueryPrincipalMappingStatus::Disabled,
        ] {
            let encoded = WorthQueryPrincipalMappingStatusBinding::encode(&status).unwrap();
            assert_eq!(
                WorthQueryPrincipalMappingStatusBinding::decode(&encoded).unwrap(),
                status
            );
        }
    }
}
