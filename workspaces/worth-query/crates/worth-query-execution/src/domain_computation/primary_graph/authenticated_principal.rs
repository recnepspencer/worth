use std::marker::PhantomData;
use std::time::Instant;

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryPrincipalAttribute,
};
use worth_query_installation::facade::{
    ApplicationSchemaBindingIdentity, WorthQueryExternalPrincipalIdentity,
};
use worth_relational::facade::identity::{EntityId, RelationId};

use super::freshness::WorthQueryPrincipalFreshnessEvidence;
use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;

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
    external_identity: WorthQueryExternalPrincipalIdentity,
    attributes: Vec<WorthQueryPrincipalAttribute>,
    valid_until: Instant,
    application_principal:
        WorthQueryApplicationPrincipalIdentity<Schema, Principal, PrincipalIdentity>,
    binding_identity: ApplicationSchemaBindingIdentity,
    binding: String,
    mapping_entity_id: EntityId,
    target_relation_id: RelationId,
    freshness: WorthQueryPrincipalFreshnessEvidence,
    examined_candidate_count: usize,
}

pub(super) struct WorthQueryResolvedPrincipalEvidence<PrincipalIdentity> {
    pub(super) principal_entity_id: EntityId,
    pub(super) principal_identity: PrincipalIdentity,
    pub(super) runtime_authority: WorthQueryRuntimeAuthorityIdentity,
    pub(super) binding_identity: ApplicationSchemaBindingIdentity,
    pub(super) binding: String,
    pub(super) mapping_entity_id: EntityId,
    pub(super) target_relation_id: RelationId,
    pub(super) freshness: WorthQueryPrincipalFreshnessEvidence,
    pub(super) examined_candidate_count: usize,
}

impl<Schema, Principal, PrincipalIdentity>
    WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>
{
    pub(super) fn mint(
        external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        evidence: WorthQueryResolvedPrincipalEvidence<PrincipalIdentity>,
    ) -> Self {
        Self {
            external_identity: external.identity().clone(),
            attributes: external.attributes().to_vec(),
            valid_until: external.valid_until(),
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

    pub fn application_principal(
        &self,
    ) -> &WorthQueryApplicationPrincipalIdentity<Schema, Principal, PrincipalIdentity> {
        &self.application_principal
    }

    pub fn principal_identity(&self) -> &PrincipalIdentity {
        &self.application_principal.identity
    }

    pub fn external_identity(&self) -> &WorthQueryExternalPrincipalIdentity {
        &self.external_identity
    }

    pub fn attributes(&self) -> &[WorthQueryPrincipalAttribute] {
        &self.attributes
    }

    pub fn binding_identity(&self) -> &ApplicationSchemaBindingIdentity {
        &self.binding_identity
    }

    pub fn binding(&self) -> &str {
        &self.binding
    }

    pub fn valid_until(&self) -> Instant {
        self.valid_until
    }

    pub fn is_expired(&self) -> bool {
        Instant::now() >= self.valid_until
    }

    pub const fn examined_candidate_count(&self) -> usize {
        self.examined_candidate_count
    }

    pub(crate) const fn principal_entity_id(&self) -> EntityId {
        self.application_principal.entity_id
    }

    pub(crate) const fn runtime_authority(&self) -> WorthQueryRuntimeAuthorityIdentity {
        self.application_principal.runtime_authority
    }

    pub(crate) const fn mapping_entity_id(&self) -> EntityId {
        self.mapping_entity_id
    }

    pub(crate) const fn target_relation_id(&self) -> RelationId {
        self.target_relation_id
    }

    pub(in crate::domain_computation) const fn freshness(
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
