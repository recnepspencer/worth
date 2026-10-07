use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::identity::EntityId;

use super::super::capability_registry::WorthQueryInstalledCapabilityPlan;
use super::super::WorthQueryAdmittedApplicationCapabilityAccess;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

pub(super) fn selected_elevation_entity<Schema, Capability, Operation, Input>(
    access: &WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
    installed: &WorthQueryInstalledCapabilityPlan,
) -> Option<EntityId>
where
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    let lifecycle = installed.contract().elevation().definition()?.lifecycle();
    access.resolved_context_entity(lifecycle.elevation_slot())
}

pub(super) fn selected_review_entity<Schema, Capability, Operation, Input>(
    access: &WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
    installed: &WorthQueryInstalledCapabilityPlan,
) -> Option<EntityId>
where
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    let lifecycle = installed.contract().elevation().definition()?.lifecycle();
    access.resolved_context_entity(lifecycle.review_slot())
}

pub(super) fn resolve_elevation_identity<Schema, Capability, Operation, Input>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    access: &WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
    capability_identity: [u8; 32],
    installed: &WorthQueryInstalledCapabilityPlan,
    value: &AspectValue,
) -> Result<Option<EntityId>, crate::facade::primary_graph::WorthQueryHandleDenial>
where
    Schema: ApplicationSchema,
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    access
        .with_exact_observation(runtime, |observation| {
            observation.resolve_elevation_identity(capability_identity, installed, value.clone())
        })
        .map(|result| result.and_then(Result::ok))
}

pub(super) fn resolve_review_identity<Schema, Capability, Operation, Input>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    access: &WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
    capability_identity: [u8; 32],
    installed: &WorthQueryInstalledCapabilityPlan,
    value: &AspectValue,
) -> Result<Option<EntityId>, crate::facade::primary_graph::WorthQueryHandleDenial>
where
    Schema: ApplicationSchema,
    Input: ApplicationCapabilityRequest<Schema, Capability>,
{
    access
        .with_exact_observation(runtime, |observation| {
            observation.resolve_review_identity(capability_identity, installed, value.clone())
        })
        .map(|result| result.and_then(Result::ok))
}
