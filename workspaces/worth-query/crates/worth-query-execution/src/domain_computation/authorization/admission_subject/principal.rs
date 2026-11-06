//! The authenticated principal as admission compares it, and its two graph reads.
use std::time::Instant;

use worth_foundational::facade::AspectValue;
use worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock;
use worth_query_installation::facade::{
    ApplicationSchemaBindingIdentity, WorthQueryExternalPrincipalIdentity,
};
use worth_relational::facade::identity::EntityId;

use crate::domain_computation::authorization::WorthQueryPrincipalCurrentnessDependency;
use crate::domain_computation::vocabulary::execution_runtime::runtime_identity::WorthQueryRuntimeAuthorityIdentity;
use crate::domain_computation::vocabulary::primary_graph::principal_binding::WorthQueryPrimaryPrincipalBindingLayout;
use crate::domain_computation::vocabulary::primary_graph::resolution_denial::WorthQueryPrincipalResolutionDenial;
use crate::domain_computation::vocabulary::provider_session::session_identity::WorthQueryGraphWorkSessionIdentity;

/// Identity authority already proven by the primary graph, read as authorization admits it.
///
/// The type parameters are the principal's own (schema, principal declaration, decoded
/// identity): the admitted subject stays typed, and every admission phase keeps inferring its
/// parameters from the subject it is handed. Nothing here mints or rebuilds the principal.
pub(in crate::domain_computation) trait WorthQueryAdmissionPrincipal<
    Schema,
    Principal,
    PrincipalIdentity,
>
{
    fn binding(&self) -> &str;
    fn binding_identity(&self) -> &ApplicationSchemaBindingIdentity;
    fn external_identity(&self) -> &WorthQueryExternalPrincipalIdentity;
    fn clock(&self) -> &WorthQueryAdmissionClock;
    fn valid_until(&self) -> Instant;
    fn is_expired(&self) -> bool;
    fn principal_entity_id(&self) -> EntityId;
    fn runtime_authority(&self) -> WorthQueryRuntimeAuthorityIdentity;

    /// The currentness dependency of this principal within one graph-work session, captured by
    /// the dependency's own seal from this exact principal.
    fn capture_currentness(
        &self,
        session_identity: WorthQueryGraphWorkSessionIdentity,
        layout: &WorthQueryPrimaryPrincipalBindingLayout,
    ) -> WorthQueryPrincipalCurrentnessDependency;

    /// Re-observes the principal's mapping and target at `snapshot` and refuses a stale proof.
    fn validate_freshness_at_snapshot(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        relational: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        layout: &WorthQueryPrimaryPrincipalBindingLayout,
        expected_identity: &AspectValue,
    ) -> Result<(), WorthQueryPrincipalResolutionDenial>;
}
