use std::marker::PhantomData;
use std::time::Instant;

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryAuthenticationAdapterIdentity,
    WorthQueryPrincipalAttribute,
};
use worth_query_installation::facade::{
    ApplicationSchemaBindingIdentity, WorthQueryExternalPrincipalIdentity,
};
use worth_relational::facade::identity::{EntityId, RelationId};

use super::freshness::WorthQueryPrincipalFreshnessEvidence;
use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;

pub fn validate_freshness_at_snapshot<
    Schema,
    Principal,
    PrincipalIdentity,
>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
    layout: &super::WorthQueryPrimaryPrincipalBindingLayout,
    expected_identity: &worth_foundational::facade::AspectValue,
) -> Result<(), super::WorthQueryPrincipalResolutionDenial> {
    super::freshness::validate_freshness_fields_at_snapshot(
        execution,
        runtime,
        snapshot,
        principal.mapping_entity_id(),
        principal.target_relation_id(),
        principal.principal_entity_id(),
        principal.binding(),
        principal.freshness(),
        layout,
        expected_identity,
    )
}


/// Opaque identity of one application principal in an installed primary graph.
///
/// This value is descriptive and cannot authorize an operation by itself.
pub struct WorthQueryApplicationPrincipalIdentity<Schema, Principal, PrincipalIdentity> {
    entity_id: EntityId,
    runtime_authority: WorthQueryRuntimeAuthorityIdentity,
    identity: PrincipalIdentity,
    _marker: PhantomData<fn() -> (Schema, Principal)>,
}

impl<Schema, Principal, PrincipalIdentity> std::fmt::Debug
    for WorthQueryApplicationPrincipalIdentity<Schema, Principal, PrincipalIdentity>
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryApplicationPrincipalIdentity")
            .field("runtime_authority", &self.runtime_authority)
            .finish_non_exhaustive()
    }
}

/// Sealed proof that a time-bounded external authentication resolved through
/// the current installed primary graph to one enabled application principal.
///
/// This is identity authority only. It grants no operation, permission,
/// touched-graph, provider, or commit authority.
///
/// Direct field construction is not a consumer authority path:
///
/// ```compile_fail
/// use worth_query_execution::facade::primary_graph::WorthQueryAuthenticatedPrincipal;
///
/// let _ = WorthQueryAuthenticatedPrincipal::<(), (), u64> {
///     external_identity: todo!(),
///     attributes: Vec::new(),
///     valid_until: std::time::Instant::now(),
///     application_principal: todo!(),
///     binding_identity: todo!(),
///     binding: String::new(),
///     mapping_entity_id: todo!(),
///     target_relation_id: todo!(),
///     freshness: todo!(),
///     examined_candidate_count: 0,
/// };
/// ```
///
/// Serialized data cannot mint the proof:
///
/// ```compile_fail
/// use worth_query_execution::facade::primary_graph::WorthQueryAuthenticatedPrincipal;
///
/// let _: WorthQueryAuthenticatedPrincipal<(), (), u64> =
///     serde_json::from_str("{}").unwrap();
/// ```
pub struct WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity> {
    pub external: std::sync::Arc<WorthQueryAuthenticatedExternalPrincipal<Schema>>,
    pub application_principal:
        WorthQueryApplicationPrincipalIdentity<Schema, Principal, PrincipalIdentity>,
    pub binding_identity: ApplicationSchemaBindingIdentity,
    pub binding: String,
    pub mapping_entity_id: EntityId,
    pub target_relation_id: RelationId,
    pub freshness: WorthQueryPrincipalFreshnessEvidence,
    pub examined_candidate_count: usize,
}

pub struct WorthQueryResolvedPrincipalEvidence<PrincipalIdentity> {
    pub principal_entity_id: EntityId,
    pub principal_identity: PrincipalIdentity,
    pub runtime_authority: WorthQueryRuntimeAuthorityIdentity,
    pub binding_identity: ApplicationSchemaBindingIdentity,
    pub binding: String,
    pub mapping_entity_id: EntityId,
    pub target_relation_id: RelationId,
    pub freshness: WorthQueryPrincipalFreshnessEvidence,
    pub examined_candidate_count: usize,
}

impl<Schema, Principal, PrincipalIdentity>
    WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>
{
    pub fn mint(
        external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        evidence: WorthQueryResolvedPrincipalEvidence<PrincipalIdentity>,
    ) -> Self {
        Self {
            external: std::sync::Arc::new(external.clone()),
            application_principal: WorthQueryApplicationPrincipalIdentity {
                entity_id: evidence.principal_entity_id,
                runtime_authority: evidence.runtime_authority,
                identity: evidence.principal_identity,
                _marker: PhantomData,
            },
            binding_identity: evidence.binding_identity,
            binding: evidence.binding,
            mapping_entity_id: evidence.mapping_entity_id,
            target_relation_id: evidence.target_relation_id,
            freshness: evidence.freshness,
            examined_candidate_count: evidence.examined_candidate_count,
        }
    }

    pub fn retain_authentication(
        &self,
    ) -> std::sync::Arc<dyn std::any::Any + Send + Sync>
    where
        Schema: 'static,
    {
        self.external.clone()
    }

    pub fn application_principal(
        &self,
    ) -> &WorthQueryApplicationPrincipalIdentity<Schema, Principal, PrincipalIdentity> {
        &self.application_principal
    }

    pub fn principal_identity(&self) -> &PrincipalIdentity {
        &self.application_principal.identity
    }

    pub fn external_identity(&self) -> &WorthQueryExternalPrincipalIdentity {
        self.external.identity()
    }

    pub fn adapter_identity(&self) -> &WorthQueryAuthenticationAdapterIdentity {
        self.external.adapter_identity()
    }

    pub fn attributes(&self) -> &[WorthQueryPrincipalAttribute] {
        self.external.attributes()
    }

    pub fn binding_identity(&self) -> &ApplicationSchemaBindingIdentity {
        &self.binding_identity
    }

    pub fn binding(&self) -> &str {
        &self.binding
    }

    pub fn valid_until(&self) -> Instant {
        self.external.valid_until()
    }

    pub fn clock(
        &self,
    ) -> &worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock {
        self.external.clock()
    }

    pub fn is_expired(&self) -> bool {
        self.external.clock().now() >= self.external.valid_until()
    }

    pub const fn examined_candidate_count(&self) -> usize {
        self.examined_candidate_count
    }

    pub const fn principal_entity_id(&self) -> EntityId {
        self.application_principal.entity_id
    }

    pub const fn runtime_authority(&self) -> WorthQueryRuntimeAuthorityIdentity {
        self.application_principal.runtime_authority
    }

    pub const fn mapping_entity_id(&self) -> EntityId {
        self.mapping_entity_id
    }

    pub const fn target_relation_id(&self) -> RelationId {
        self.target_relation_id
    }

    pub const fn freshness(
        &self,
    ) -> &WorthQueryPrincipalFreshnessEvidence {
        &self.freshness
    }
}

impl<Schema, Principal, PrincipalIdentity> std::fmt::Debug
    for WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryAuthenticatedPrincipal")
            .field("application_principal", &self.application_principal)
            .field("binding_identity", &self.binding_identity)
            .field("binding", &self.binding)
            .field("valid_until", &self.valid_until())
            .field("examined_candidate_count", &self.examined_candidate_count)
            .finish_non_exhaustive()
    }
}

mod court_doors_for_test;

pub mod command_pin;
pub use command_pin::*;
