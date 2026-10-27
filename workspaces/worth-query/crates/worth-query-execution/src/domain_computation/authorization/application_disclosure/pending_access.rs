//! One actual capability-to-disclosure handoff for issued application scopes.
use super::WorthQueryPendingApplicationQueryGovernance;
use crate::domain_computation::{
    authorization::WorthQueryAdmittedApplicationCapabilityAccess,
    primary_graph::WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
use worth_query_installation::facade::{ApplicationSchema, ApplicationSchemaBindingIdentity};
use worth_relational::facade::identity::EntityId;

pub(in crate::domain_computation) fn prepare_pending_disclosure<
    Schema,
    Scope,
    Capability,
    Operation,
    Input,
    E,
>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    binding: &ApplicationSchemaBindingIdentity,
    principal: EntityId,
    scope: EntityId,
    capability: WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
    validate_current_request: impl FnOnce() -> Result<(), E>,
    mismatch: impl Fn() -> E,
) -> Result<WorthQueryPendingApplicationQueryGovernance, E>
where
    Schema: ApplicationSchema,
    Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope>,
{
    // Each caller first validates the capability's original request. Preserve
    // the existing order: affinity, receiving request, then retained projection.
    if capability.runtime_authority() != runtime.runtime.authority_identity()
        || capability.binding_identity() != binding
        || capability.principal_entity_id() != principal
        || capability.resource_entity_id() != scope
        || runtime.authentication_is_expired(capability.authentication_valid_until())
    {
        return Err(mismatch());
    }
    validate_current_request()?;
    let capability_name = capability.capability_name().to_string();
    let capability_type = capability.capability_type().to_string();
    let authorization_canonical_work = capability.admission_canonical_work();
    let disclosure_value = capability
        .disclosure_value()
        .cloned()
        .ok_or_else(&mismatch)?;
    Ok(WorthQueryPendingApplicationQueryGovernance::new(
        capability_name,
        capability_type,
        disclosure_value,
        authorization_canonical_work,
        capability.into_query_authorization(),
    ))
}
