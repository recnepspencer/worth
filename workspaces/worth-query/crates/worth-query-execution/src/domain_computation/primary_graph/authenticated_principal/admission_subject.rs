//! The authenticated principal through authorization's admission contract.
//!
//! The primary graph implements the contract beside the value it mints: authorization reads
//! identity, binding and validity, and asks this owner for the graph reads on its evidence.
//! The mint and the resolved evidence stay sealed to the primary graph.
use super::*;
use crate::domain_computation::authorization::{
    WorthQueryAdmissionPrincipal, WorthQueryPrincipalCurrentnessDependency,
};
use crate::domain_computation::vocabulary::primary_graph::principal_binding::WorthQueryPrimaryPrincipalBindingLayout;
use crate::domain_computation::vocabulary::primary_graph::resolution_denial::WorthQueryPrincipalResolutionDenial;
use crate::domain_computation::vocabulary::provider_session::session_identity::WorthQueryGraphWorkSessionIdentity;

impl<Schema, Principal, PrincipalIdentity>
    WorthQueryAdmissionPrincipal<Schema, Principal, PrincipalIdentity>
    for WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>
{
    fn binding(&self) -> &str {
        Self::binding(self)
    }

    fn binding_identity(&self) -> &ApplicationSchemaBindingIdentity {
        Self::binding_identity(self)
    }

    fn external_identity(&self) -> &WorthQueryExternalPrincipalIdentity {
        Self::external_identity(self)
    }

    fn clock(
        &self,
    ) -> &worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock {
        Self::clock(self)
    }

    fn valid_until(&self) -> Instant {
        Self::valid_until(self)
    }

    fn is_expired(&self) -> bool {
        Self::is_expired(self)
    }

    fn principal_entity_id(&self) -> EntityId {
        Self::principal_entity_id(self)
    }

    fn runtime_authority(&self) -> WorthQueryRuntimeAuthorityIdentity {
        Self::runtime_authority(self)
    }

    fn capture_currentness(
        &self,
        session_identity: WorthQueryGraphWorkSessionIdentity,
        layout: &WorthQueryPrimaryPrincipalBindingLayout,
    ) -> WorthQueryPrincipalCurrentnessDependency {
        WorthQueryPrincipalCurrentnessDependency::capture(session_identity, self, layout)
    }

    fn validate_freshness_at_snapshot(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        relational: &worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
        layout: &WorthQueryPrimaryPrincipalBindingLayout,
        expected_identity: &worth_foundational::facade::AspectValue,
    ) -> Result<(), WorthQueryPrincipalResolutionDenial> {
        super::super::freshness::validate_freshness_at_snapshot(
            execution,
            relational,
            snapshot,
            self,
            layout,
            expected_identity,
        )
    }
}
