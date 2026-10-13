//! Exact request-projection validation for capability observation.

use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use std::collections::BTreeSet;

use worth_query_declaration::facade::application_capability::{
    ApplicationCapabilityCardinalityDimension, ApplicationCapabilityRelationDimension,
};

use super::super::capability_registry::WorthQueryInstalledCapabilityPlan;
use super::super::retained_capability_request::WorthQueryRetainedCapabilityRequest;
use super::super::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
};
use super::path_preparation;

pub(super) fn validate_projection_shape(
    installed: &WorthQueryInstalledCapabilityPlan,
    projection: &WorthQueryRetainedCapabilityRequest,
    path_count: usize,
    elevation_required: bool,
) -> Result<(), WorthQueryOperationAuthorizationDenial> {
    let request = &installed.request();
    validate_purpose(installed, projection)?;
    validate_scope(installed, projection)?;
    validate_operation_shape(installed, projection, elevation_required)?;
    validate_relation(installed, projection)?;
    let expected_context = installed
        .paths()
        .iter()
        .take(path_count)
        .flat_map(|path| {
            path.context_anchors
                .iter()
                .map(path_preparation::context_key)
        })
        .collect::<BTreeSet<_>>();
    if !expected_context
        .iter()
        .all(|key| projection.context().contains_key(key))
    {
        return Err(scope_mismatch(installed));
    }
    debug_assert_eq!(&request.purpose, projection.purpose());
    Ok(())
}

fn validate_purpose(
    installed: &WorthQueryInstalledCapabilityPlan,
    projection: &WorthQueryRetainedCapabilityRequest,
) -> Result<(), WorthQueryOperationAuthorizationDenial> {
    if projection.purpose() != &installed.request().purpose {
        return Err(WorthQueryOperationAuthorizationDenial::new(
            WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::PurposeMismatch,
            ),
            installed.contract().name(),
        ));
    }
    Ok(())
}

fn validate_scope(
    installed: &WorthQueryInstalledCapabilityPlan,
    projection: &WorthQueryRetainedCapabilityRequest,
) -> Result<(), WorthQueryOperationAuthorizationDenial> {
    let request = &installed.request();
    if projection.resource_entity() != request.resource_entity
        || projection.context_name() != request.context
        || projection.context_type() != request.context_type
        || !cardinality_admitted(request.cardinality, projection.cardinality())
        || projection.magnitude().is_some() != request.magnitude.is_some()
    {
        return Err(scope_mismatch(installed));
    }
    Ok(())
}

fn validate_operation_shape(
    installed: &WorthQueryInstalledCapabilityPlan,
    projection: &WorthQueryRetainedCapabilityRequest,
    elevation_required: bool,
) -> Result<(), WorthQueryOperationAuthorizationDenial> {
    let request = &installed.request();
    if projection.action() != &request.action
        || projection.field().is_some() != request.field.is_some()
        || projection.elevation().is_some() != elevation_required
    {
        return Err(WorthQueryOperationAuthorizationDenial::new(
            WorthQueryOperationAuthorizationDenialKind::Refused(
                AuthorizationRefusal::CapabilityProjectionRejected,
            ),
            installed.contract().name(),
        ));
    }
    Ok(())
}

fn validate_relation(
    installed: &WorthQueryInstalledCapabilityPlan,
    projection: &WorthQueryRetainedCapabilityRequest,
) -> Result<(), WorthQueryOperationAuthorizationDenial> {
    let matches = match (
        installed.contract().target().relation(),
        projection.related_relation(),
    ) {
        (ApplicationCapabilityRelationDimension::NotApplicable, None) => true,
        (ApplicationCapabilityRelationDimension::Bound(expected), Some(actual)) => {
            expected == actual
        }
        _ => false,
    };
    if !matches {
        return Err(scope_mismatch(installed));
    }
    Ok(())
}

const fn cardinality_admitted(
    installed: ApplicationCapabilityCardinalityDimension,
    requested: u32,
) -> bool {
    match installed {
        ApplicationCapabilityCardinalityDimension::One => requested == 1,
        ApplicationCapabilityCardinalityDimension::Many => requested > 0,
        ApplicationCapabilityCardinalityDimension::Bounded(maximum) => {
            requested > 0 && requested <= maximum
        }
    }
}

fn scope_mismatch(
    installed: &WorthQueryInstalledCapabilityPlan,
) -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(
        WorthQueryOperationAuthorizationDenialKind::Refused(AuthorizationRefusal::ScopeMismatch),
        installed.contract().name(),
    )
}
