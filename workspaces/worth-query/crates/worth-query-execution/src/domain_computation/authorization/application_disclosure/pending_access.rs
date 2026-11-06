//! One actual capability-to-disclosure handoff for issued application scopes.
use super::WorthQueryPendingApplicationQueryGovernance;
use crate::domain_computation::{
    authorization::WorthQueryAdmittedApplicationCapabilityAccess,
    primary_graph::WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
use worth_query_installation::facade::{ApplicationSchema, ApplicationSchemaBindingIdentity};
use worth_relational::facade::identity::EntityId;

pub(crate) mod funded;
pub(in crate::domain_computation) use funded::prepare_funded_pending_disclosure;

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
    validate_original_request: impl FnOnce(
        &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<(), E>,
    validate_current_request: impl FnOnce() -> Result<(), E>,
    mismatch: impl Fn() -> E,
    incarnation_refusal: impl FnOnce(
        crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial,
    ) -> E,
) -> Result<WorthQueryPendingApplicationQueryGovernance, E>
where
    Schema: ApplicationSchema,
    Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope>,
{
    prepare_with_capture(
        runtime,
        binding,
        principal,
        scope,
        capability,
        validate_original_request,
        validate_current_request,
        mismatch,
        incarnation_refusal,
        |name, kind, value| Ok((name.to_owned(), kind.to_owned(), value.clone(), ())),
    )
    .map(|(pending, ())| pending)
}

fn prepare_with_capture<Schema, Scope, Capability, Operation, Input, E, Capture>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    binding: &ApplicationSchemaBindingIdentity,
    principal: EntityId,
    scope: EntityId,
    capability: WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
    validate_original_request: impl FnOnce(
        &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> Result<(), E>,
    validate_current_request: impl FnOnce() -> Result<(), E>,
    mismatch: impl Fn() -> E,
    incarnation_refusal: impl FnOnce(
        crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial,
    ) -> E,
    capture: impl FnOnce(
        &str,
        &str,
        &worth_foundational::facade::AspectValue,
    ) -> Result<
        (
            String,
            String,
            worth_foundational::facade::AspectValue,
            Capture,
        ),
        E,
    >,
) -> Result<(WorthQueryPendingApplicationQueryGovernance, Capture), E>
where
    Schema: ApplicationSchema,
    Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope>,
{
    // Each caller first validates the capability's original request. Preserve
    // the existing order: affinity, receiving request, then retained projection.
    let issuing_runtime = match capability.runtime_authority_for(runtime.runtime.product_runtime.owner.owner_identity()) {
        Ok(authority) => Some(authority),
        Err(worth_proof::HomeIncarnationDenial::ForeignHome) => None,
        Err(worth_proof::HomeIncarnationDenial::StaleIncarnation) => return Err(incarnation_refusal(
            crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial::new(
                crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::Refused(
                    crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind::StaleIncarnation), "capability disclosure"))),
    };
    validate_original_request(capability.request_scope())?;
    if issuing_runtime != Some(runtime.runtime.authority_identity())
        || capability.binding_identity() != binding
        || capability.principal_entity_id() != principal
        || capability.resource_entity_id() != scope
        || runtime.authentication_is_expired(capability.authentication_valid_until())
    {
        return Err(mismatch());
    }
    validate_current_request()?;
    let disclosure_value = capability.disclosure_value().ok_or_else(&mismatch)?;
    let (capability_name, capability_type, disclosure_value, capture) = capture(
        capability.capability_name(),
        capability.capability_type(),
        disclosure_value,
    )?;
    let authorization_canonical_work = capability.admission_canonical_work();
    let pending = WorthQueryPendingApplicationQueryGovernance::new(
        capability_name,
        capability_type,
        disclosure_value,
        authorization_canonical_work,
        capability.into_query_authorization(),
    );
    Ok((pending, capture))
}
