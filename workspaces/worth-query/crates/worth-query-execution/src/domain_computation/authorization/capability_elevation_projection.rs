//! Exact request-shape validation for installed elevation meaning.

use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use worth_query_declaration::facade::application_capability::{
    ApplicationCapabilityElevationRule, ApplicationCapabilityFieldBinding,
    ApplicationCapabilityRequestProjection, ErasedApplicationCapabilityContract,
    ErasedApplicationCapabilityEntitySelector,
};

use super::{WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind};

pub(in crate::domain_computation) fn validate_elevation_projection<Schema, Scope, Context>(
    contract: &ErasedApplicationCapabilityContract,
    projection: &ApplicationCapabilityRequestProjection<Schema, Scope, Context>,
) -> Result<(), WorthQueryOperationAuthorizationDenial> {
    match (contract.elevation(), projection.elevation_selector()) {
        (ApplicationCapabilityElevationRule::NotApplicable, None) => Ok(()),
        (ApplicationCapabilityElevationRule::NotApplicable, Some(_)) => Err(denial(
            WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::ElevationNotApplicable,
            ),
            contract.name(),
        )),
        (ApplicationCapabilityElevationRule::Governed(_), None) => Err(denial(
            WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::ElevationRequired,
            ),
            contract.name(),
        )),
        (ApplicationCapabilityElevationRule::Governed(definition), Some(selector))
            if selector_matches(definition.identity(), selector) =>
        {
            Ok(())
        }
        (ApplicationCapabilityElevationRule::Governed(_), Some(_)) => Err(denial(
            WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::ElevationProjectionRejected,
            ),
            contract.name(),
        )),
    }
}

fn selector_matches(
    expected: &ApplicationCapabilityFieldBinding,
    actual: &ErasedApplicationCapabilityEntitySelector,
) -> bool {
    expected.entity() == actual.entity()
        && expected.aspect() == actual.aspect()
        && expected.field() == actual.field()
        && expected.scalar_family() == actual.scalar_family()
        && expected.value_type() == actual.value_type()
}

fn denial(
    kind: WorthQueryOperationAuthorizationDenialKind,
    subject: impl Into<String>,
) -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(kind, subject)
}
