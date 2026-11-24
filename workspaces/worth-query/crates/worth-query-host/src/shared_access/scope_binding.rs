//! Scope-proof binding and native query row-limit intersection.

use std::num::NonZeroUsize;

use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest, application_schema::ApplicationSchema,
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationEntityIdentity, WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryBinding;
use worth_relational::facade::identity::EntityId;

use super::binding::WorthQuerySharedAccessProgram;
use super::denial::{AuthorizationRefusalReason, BudgetRefusalReason, SharedAccessDenial};
use super::types::RemoteServerLimits;
use super::watch_drain::IntentBinding;

/// Exact resolved scope entity and selector coordinates retained across a watch's lifetime.
pub(super) struct BoundScopeIdentity {
    entity_id: EntityId,
    entity: &'static str,
    aspect: &'static str,
    field: &'static str,
    value: AspectValue,
}

impl BoundScopeIdentity {
    pub(super) fn from_resolved_and_read<Schema, Cap, Input, Scope>(
        resolved: &WorthQueryApplicationEntityIdentity<Schema, Scope>,
        read_input: &Input,
    ) -> Result<Self, SharedAccessDenial>
    where
        Schema: ApplicationSchema,
        Input: ApplicationCapabilityRequest<Schema, Cap, Scope = Scope>,
    {
        let projection = read_input.capability_request().map_err(|_| {
            SharedAccessDenial::AuthorizationRefused(AuthorizationRefusalReason::CapabilityDenied)
        })?;
        let resource = projection.resource();
        if resource.entity() != resolved.entity_name()
            || resource.value() != resolved.identity_value()
        {
            return Err(SharedAccessDenial::AuthorizationRefused(
                AuthorizationRefusalReason::CapabilityDenied,
            ));
        }
        Ok(Self {
            entity_id: resolved.entity_id(),
            entity: resource.entity(),
            aspect: resource.aspect(),
            field: resource.field(),
            value: resource.value().clone(),
        })
    }

    pub(super) const fn entity_id(&self) -> EntityId {
        self.entity_id
    }
}

/// Ensures a capability input's resource selector targets the exact bound scope entity.
pub(super) fn verify_capability_matches_scope<Schema, Cap, Input, Scope>(
    input: &Input,
    scope: &BoundScopeIdentity,
) -> Result<(), SharedAccessDenial>
where
    Schema: ApplicationSchema,
    Input: ApplicationCapabilityRequest<Schema, Cap, Scope = Scope>,
{
    let projection = input.capability_request().map_err(|_| {
        SharedAccessDenial::AuthorizationRefused(AuthorizationRefusalReason::CapabilityDenied)
    })?;
    let resource = projection.resource();
    if resource.entity() != scope.entity
        || resource.aspect() != scope.aspect
        || resource.field() != scope.field
        || resource.value() != &scope.value
    {
        return Err(SharedAccessDenial::AuthorizationRefused(
            AuthorizationRefusalReason::CapabilityDenied,
        ));
    }
    Ok(())
}

/// Intersects remote server row limit with BOTH graph cardinality AND installed binding cap.
pub(super) fn clamped_query_row_limit<Schema, Program>(
    app: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    limits: RemoteServerLimits,
) -> Result<NonZeroUsize, SharedAccessDenial>
where
    Schema: ApplicationSchema + 'static,
    Program: WorthQuerySharedAccessProgram<Schema>,
{
    let binding = app
        .installed_schema()
        .installed_query_binding::<IntentBinding<Schema, Program>>()
        .map_err(|_| SharedAccessDenial::HostUnavailable)?;
    clamped_row_limit_for_binding::<Schema, Program>(&binding, limits)
}

pub(super) fn clamped_row_limit_for_binding<Schema, Program>(
    binding: &WorthQueryInstalledApplicationQueryBinding<Schema, IntentBinding<Schema, Program>>,
    limits: RemoteServerLimits,
) -> Result<NonZeroUsize, SharedAccessDenial>
where
    Schema: ApplicationSchema + 'static,
    Program: WorthQuerySharedAccessProgram<Schema>,
{
    let graph_max = binding.query().read_graph().maximum_result_count().max(1);
    let binding_max = binding.limits().maximum_results().get();
    let clamped = (limits.max_rows_per_response() as usize)
        .min(graph_max)
        .min(binding_max);
    NonZeroUsize::new(clamped).ok_or(SharedAccessDenial::BudgetRefused(
        BudgetRefusalReason::RequestLimitExceeded,
    ))
}
