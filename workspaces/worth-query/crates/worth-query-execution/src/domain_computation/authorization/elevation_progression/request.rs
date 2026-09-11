use std::collections::BTreeSet;

use worth_query_declaration::facade::application_capability::{
    ApplicationCapabilityCardinalityDimension, ApplicationCapabilityElevationRequest,
    ApplicationCapabilityElevationRequestProjection, ApplicationCapabilityRelationDimension,
    ApplicationCapabilityRequest,
};
use worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity;
use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;
use worth_query_installation::facade::{
    ApplicationSchema, WorthQueryInstalledApplicationOperation,
};

use super::super::capability_registry::{
    WorthQueryElevationLifecycleOperationRole, WorthQueryInstalledCapabilityPlan,
};
use super::super::operation_progression::{
    progress_capability_operation, WorthQueryCapabilityOperationProgression,
};
use super::super::{
    WorthQueryAdmittedApplicationCapabilityAccess, WorthQueryAdmittedApplicationOperation,
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

use super::request_binding::bind_request;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn authorize_elevation_request<Capability, Operation, Input>(
        &self,
        mut access: WorthQueryAdmittedApplicationCapabilityAccess<
            Schema,
            Capability,
            Operation,
            Input,
        >,
        operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
        preconditions: TypedMutationPreconditions<
            Schema,
            Operation,
            <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
        >,
    ) -> Result<
        WorthQueryAdmittedApplicationOperation<
            Schema,
            Operation,
            Input,
            <Input as ApplicationCapabilityRequest<Schema, Capability>>::Scope,
        >,
        WorthQueryOperationAuthorizationDenial,
    >
    where
        Operation: ApplicationOperationMarkerIdentity<Schema>,
        Input: ApplicationCapabilityRequest<Schema, Capability>
            + ApplicationCapabilityElevationRequest<Schema, Operation>
            + 'static,
    {
        let proposed = access
            .capability_input()
            .elevation_request()
            .map_err(|rejection| {
                denial(
                    WorthQueryOperationAuthorizationDenialKind::ElevationRequestRejected,
                    rejection.subject(),
                )
            })?;
        let (capability_identity, installed) =
            installed_request_lifecycle(self, &access, operation)?;
        validate_request_projection::<Schema, Operation, Input>(installed, &proposed)?;
        let binding = bind_request(self, capability_identity, installed, &access, &proposed)?;
        access
            .retain_observed_support(binding.supporting().retained_for_operation())
            .map_err(|()| {
                denial(
                    WorthQueryOperationAuthorizationDenialKind::InconsistentDecision,
                    installed.contract().name(),
                )
            })?;
        progress_capability_operation(
            self,
            access,
            operation,
            preconditions,
            WorthQueryCapabilityOperationProgression::ElevationLifecycle,
        )?
        .bind_elevation_request(binding)
    }
}

fn installed_request_lifecycle<'runtime, Schema, Capability, Operation, Input>(
    runtime: &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    access: &WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
    operation: &WorthQueryInstalledApplicationOperation<Schema, Operation, Input>,
) -> Result<
    ([u8; 32], &'runtime WorthQueryInstalledCapabilityPlan),
    WorthQueryOperationAuthorizationDenial,
>
where
    Schema: ApplicationSchema,
    Operation: ApplicationOperationMarkerIdentity<Schema>,
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    let Some((capability, command_capability, role)) = runtime
        .authorization
        .elevation_lifecycle_operation::<Schema, Operation>(
            operation.operation(),
            operation.input_type(),
        )
        .map_err(|()| stale_operation(operation.operation()))?
    else {
        return Err(denial(
            WorthQueryOperationAuthorizationDenialKind::ElevationLifecycleRoleMismatch,
            operation.operation(),
        ));
    };
    if role != WorthQueryElevationLifecycleOperationRole::Request
        || access.installed_capability_identity() != command_capability
    {
        return Err(denial(
            WorthQueryOperationAuthorizationDenialKind::ElevationLifecycleRoleMismatch,
            operation.operation(),
        ));
    }
    let installed = runtime
        .authorization
        .capability_plan_by_identity(&capability)
        .filter(|plan| plan.elevation().is_some())
        .ok_or_else(|| stale_operation(operation.operation()))?;
    Ok((capability, installed))
}

fn validate_request_projection<Schema, Operation, Input>(
    installed: &WorthQueryInstalledCapabilityPlan,
    proposed: &ApplicationCapabilityElevationRequestProjection<
        Schema,
        <Input as ApplicationCapabilityElevationRequest<Schema, Operation>>::Scope,
        <Input as ApplicationCapabilityElevationRequest<Schema, Operation>>::Context,
    >,
) -> Result<(), WorthQueryOperationAuthorizationDenial>
where
    Input: ApplicationCapabilityElevationRequest<Schema, Operation>,
{
    let target = proposed.target();
    let request = &installed.request();
    let elevation = installed
        .contract()
        .elevation()
        .definition()
        .ok_or_else(|| projection_denial(installed.contract().name()))?;
    if target.elevation_selector().is_some()
        || target.action() != &request.action
        || target.purpose() != &request.purpose
        || target.resource().entity() != request.resource_entity
        || target.context_value().context() != request.context
        || target.context_value().context_type() != request.context_type
        || !cardinality_admitted(request.cardinality, target.cardinality_value())
        || target.field_value().is_some() != request.field.is_some()
        || target.magnitude_value().is_some() != request.magnitude.is_some()
        || !relation_matches(installed, target)
        || !context_matches(installed, target)
        || proposed.grant().entity() != installed.contract().grant_entity()
        || proposed.elevation_identity().field() != elevation.identity()
        || proposed.review_identity().field() != elevation.review().identity()
        || proposed.reason().field() != elevation.reason()
    {
        return Err(projection_denial(installed.contract().name()));
    }
    let duration = proposed.duration();
    let maximum_duration = installed
        .elevation()
        .as_ref()
        .ok_or_else(|| projection_denial(installed.contract().name()))?
        .lifecycle
        .maximum_duration;
    if duration.is_zero() || duration > maximum_duration {
        return Err(denial(
            WorthQueryOperationAuthorizationDenialKind::ElevationDurationExceeded,
            installed.contract().name(),
        ));
    }
    Ok(())
}

fn relation_matches<Schema, Scope, Context>(
    installed: &WorthQueryInstalledCapabilityPlan,
    projection: &worth_query_declaration::facade::application_capability::ApplicationCapabilityRequestProjection<
        Schema,
        Scope,
        Context,
    >,
) -> bool {
    match (
        installed.contract().target().relation(),
        projection.related(),
    ) {
        (ApplicationCapabilityRelationDimension::NotApplicable, None) => true,
        (ApplicationCapabilityRelationDimension::Bound(expected), Some(actual)) => {
            expected == actual.relation()
        }
        _ => false,
    }
}

fn context_matches<Schema, Scope, Context>(
    installed: &WorthQueryInstalledCapabilityPlan,
    projection: &worth_query_declaration::facade::application_capability::ApplicationCapabilityRequestProjection<
        Schema,
        Scope,
        Context,
    >,
) -> bool {
    let expected = installed
        .paths()
        .iter()
        .flat_map(|path| path.context_anchors.iter())
        .map(|anchor| {
            (
                anchor.context.as_str(),
                anchor.context_type.as_str(),
                anchor.slot.as_str(),
                anchor.slot_type.as_str(),
                anchor.entity.as_str(),
            )
        })
        .collect::<BTreeSet<_>>();
    let actual = projection
        .context_value()
        .entities()
        .iter()
        .map(|selected| {
            let slot = selected.slot();
            (
                slot.context(),
                slot.context_identity_ref().as_str(),
                slot.slot(),
                slot.slot_identity_ref().as_str(),
                slot.entity(),
            )
        })
        .collect::<BTreeSet<_>>();
    expected == actual
}

const fn cardinality_admitted(
    expected: ApplicationCapabilityCardinalityDimension,
    actual: u32,
) -> bool {
    match expected {
        ApplicationCapabilityCardinalityDimension::One => actual == 1,
        ApplicationCapabilityCardinalityDimension::Many => actual > 0,
        ApplicationCapabilityCardinalityDimension::Bounded(limit) => actual > 0 && actual <= limit,
    }
}

pub(super) fn projection_denial(
    subject: impl Into<String>,
) -> WorthQueryOperationAuthorizationDenial {
    denial(
        WorthQueryOperationAuthorizationDenialKind::ElevationRequestRejected,
        subject,
    )
}

fn stale_operation(subject: impl Into<String>) -> WorthQueryOperationAuthorizationDenial {
    denial(
        WorthQueryOperationAuthorizationDenialKind::StaleInstalledOperation,
        subject,
    )
}

pub(super) fn denial(
    kind: WorthQueryOperationAuthorizationDenialKind,
    subject: impl Into<String>,
) -> WorthQueryOperationAuthorizationDenial {
    WorthQueryOperationAuthorizationDenial::new(kind, subject)
}
