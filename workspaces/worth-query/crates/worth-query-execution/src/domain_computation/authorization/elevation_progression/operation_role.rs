use worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity;
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationOperation,
};

use super::super::capability_registry::{
    WorthQueryElevationLifecycleOperationRole, WorthQueryInstalledCapabilityPlan,
};
use super::super::{
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

pub(super) fn installed_lifecycle_owner<'runtime, Schema, Operation, Input>(
    runtime: &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    command_capability_identity: [u8; 32],
    operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
    expected_role: WorthQueryElevationLifecycleOperationRole,
) -> Result<
    ([u8; 32], &'runtime WorthQueryInstalledCapabilityPlan),
    WorthQueryOperationAuthorizationDenial,
>
where
    Schema: ApplicationSchema,
    Operation: ApplicationOperationMarkerIdentity<Schema>,
{
    let Some((capability, command_capability, role)) = runtime
        .authorization
        .elevation_lifecycle_operation::<Schema, Operation>(
            operation.operation(),
            operation.input_type(),
        )
        .map_err(|()| stale_operation(operation.operation()))?
    else {
        return Err(role_mismatch(operation.operation()));
    };
    if role != expected_role || command_capability != command_capability_identity {
        return Err(role_mismatch(operation.operation()));
    }
    let installed = runtime
        .authorization
        .capability_plan_by_identity(&capability)
        .filter(|plan| plan.elevation().is_some())
        .ok_or_else(|| stale_operation(operation.operation()))?;
    Ok((capability, installed))
}

fn role_mismatch(subject: impl Into<String>) -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(
        WorthQueryOperationAuthorizationDenialKind::ElevationLifecycleRoleMismatch,
        subject,
    )
}

fn stale_operation(subject: impl Into<String>) -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(
        WorthQueryOperationAuthorizationDenialKind::StaleInstalledOperation,
        subject,
    )
}
