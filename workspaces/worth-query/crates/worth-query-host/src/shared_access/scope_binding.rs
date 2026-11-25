//! Native scope-entity proof binding and query row-limit intersection.

use std::num::NonZeroUsize;

use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest, application_schema::ApplicationSchema,
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryAdmittedApplicationCapabilityAccess, WorthQueryApplicationEntityIdentity,
    WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryBinding;
use worth_relational::facade::identity::EntityId;

use super::binding::WorthQuerySharedAccessProgram;
use super::denial::{AuthorizationRefusalReason, BudgetRefusalReason, SharedAccessDenial};
use super::types::RemoteServerLimits;
use super::watch_drain::IntentBinding;

/// Exact native resolved scope entity identity retained across a watch's lifetime.
pub(super) struct BoundScopeIdentity {
    entity_id: EntityId,
}

impl BoundScopeIdentity {
    pub(super) fn from_resolved<Schema, Scope>(
        resolved: &WorthQueryApplicationEntityIdentity<Schema, Scope>,
    ) -> Self
    where
        Schema: ApplicationSchema,
    {
        Self {
            entity_id: resolved.entity_id(),
        }
    }

    pub(super) const fn entity_id(&self) -> EntityId {
        self.entity_id
    }
}

/// Verifies an admitted native capability proof's resolved resource `EntityId` matches the
/// exact bound scope entity, failing closed on any cross-row or alternate-selector shift.
pub(super) fn verify_admitted_scope_entity<Schema, Cap, Op, Input>(
    admitted: &WorthQueryAdmittedApplicationCapabilityAccess<Schema, Cap, Op, Input>,
    scope: &BoundScopeIdentity,
    reason: AuthorizationRefusalReason,
) -> Result<(), SharedAccessDenial>
where
    Schema: ApplicationSchema,
    Input: ApplicationCapabilityRequest<Schema, Cap>,
{
    if admitted.admitted_resource_entity_id() != scope.entity_id() {
        return Err(SharedAccessDenial::AuthorizationRefused(reason));
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
