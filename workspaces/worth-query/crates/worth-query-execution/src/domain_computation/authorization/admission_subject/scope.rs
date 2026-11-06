//! The resolved scope entity as admission compares it, and the graph reads it needs.
use std::sync::Arc;

use worth_query_installation::facade::ApplicationSchemaBindingIdentity;
use worth_relational::facade::identity::{EntityId, KindId};

use crate::domain_computation::authorization::host::AuthorizationHost;
use crate::domain_computation::vocabulary::execution_runtime::runtime_identity::WorthQueryRuntimeAuthorityIdentity;
use crate::domain_computation::vocabulary::primary_graph::entity_resolution_denial::WorthQueryEntityResolutionDenialKind;
use crate::domain_computation::vocabulary::primary_graph::retained_selector::WorthQueryObservedScopeSelector;
use crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind;
use crate::domain_computation::vocabulary::provider_session::protocol_denial::WorthQueryProviderSessionDenialKind;

/// Descriptive scope identity resolved by the primary graph, read as authorization admits it.
///
/// The type parameters are the identity's own (schema, scope entity): the admitted scope stays
/// typed, and every admission phase keeps inferring them from the scope it is handed.
pub(in crate::domain_computation) trait WorthQueryAdmissionScope<Schema, Scope> {
    fn entity_id(&self) -> EntityId;
    fn entity_kind(&self) -> KindId;
    fn entity_name(&self) -> &str;
    fn runtime_authority(&self) -> WorthQueryRuntimeAuthorityIdentity;
    fn binding_identity(&self) -> &ApplicationSchemaBindingIdentity;

    /// The scope selector retained beside an admitted operation, never read authority.
    fn retain_scope_selector(&self) -> Arc<WorthQueryObservedScopeSelector>;

    /// The same selector with its backing kept funded after the request returns.
    fn retain_scope_selector_funded(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        Arc<WorthQueryObservedScopeSelector>,
        Result<WorthQueryProviderSessionDenialKind, WorthQueryProviderSessionControlStopKind>,
    >;

    /// Re-validates the identity against the runtime's installed graph at `snapshot`; the kind
    /// of a refusal or stop is returned for authorization to map.
    fn validate_freshness_at_snapshot(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &dyn AuthorizationHost<Schema>,
        relational: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    ) -> Result<(), WorthQueryEntityResolutionDenialKind>;
}
