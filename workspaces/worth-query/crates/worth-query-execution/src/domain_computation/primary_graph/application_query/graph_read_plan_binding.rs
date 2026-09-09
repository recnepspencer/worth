use super::WorthQueryApplicationBasisIdentity;
use worth_foundational::facade::CanonicalDigestId;
use worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters;
use worth_query_installation::facade::{
    WorthQueryCanonicalWorkPhases, WorthQueryInstalledApplicationQuery,
    WorthQueryInstalledApplicationQueryIdentity,
};
use worth_relational::facade::indexes::DerivedIndexId;
use worth_relational::facade::indexes::{DerivedIndexGenerationId, RelatedEntityOrderingBoundary};

use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;

use super::super::{WorthQueryApplicationEntityIdentity, WorthQueryAuthenticatedPrincipal};
use super::{
    basis::WorthQueryApplicationQueryBasisCustody, WorthQueryAdmittedApplicationQueryControls,
};

/// Sealed execution authority for one exact installed query admission.
///
/// Descriptive query identities, support inventories, and graph-plan reviews
/// cannot construct this value.
pub struct WorthQueryAdmittedApplicationQueryPlan<
    'a,
    Schema,
    Query,
    Parameters,
    QueryResult,
    Principal,
    PrincipalIdentity,
    Scope,
> {
    pub(super) runtime_authority: WorthQueryRuntimeAuthorityIdentity,
    pub(super) graph_authority_identity: String,
    pub(super) provider_identity: String,
    pub(super) query:
        &'a WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
    pub(super) principal:
        &'a WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity>,
    pub(super) scope: &'a WorthQueryApplicationEntityIdentity<Schema, Scope>,
    pub(super) parameters: WorthQueryAdmittedApplicationQueryParameters,
    pub(super) controls: WorthQueryAdmittedApplicationQueryControls<'a>,
    pub(super) canonical_work: WorthQueryCanonicalWorkPhases,
    pub(super) continuation_index_id: Option<DerivedIndexId>,
    pub(super) continuation_state: Option<WorthQueryAdmittedContinuationState>,
    pub(super) basis: WorthQueryApplicationQueryBasisCustody,
    pub(super) graph_work:
        crate::domain_computation::provider_session::WorthQueryManagedGraphWorkSession,
    pub(super) authorization:
        crate::domain_computation::authorization::WorthQueryRetainedAuthorizationDecisionFacts,
    pub(super) authorization_work: super::WorthQueryApplicationAuthorizationWorkEvidence,
    pub(super) governance: super::disclosure::WorthQueryApplicationQueryGovernance,
}

pub(super) struct WorthQueryAdmittedContinuationState {
    pub(super) expected_generation: DerivedIndexGenerationId,
    pub(super) boundary: RelatedEntityOrderingBoundary,
    pub(super) page_ordinal: u64,
}

impl<'a, Schema, Query, Parameters, QueryResult, Principal, PrincipalIdentity, Scope>
    WorthQueryAdmittedApplicationQueryPlan<
        'a,
        Schema,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >
{
    pub fn query_identity(&self) -> &WorthQueryInstalledApplicationQueryIdentity {
        self.query.identity()
    }

    pub fn parameter_binding_identity(&self) -> &CanonicalDigestId {
        self.parameters.identity()
    }

    pub fn controls(&self) -> &WorthQueryAdmittedApplicationQueryControls<'a> {
        &self.controls
    }

    pub fn graph_read_plan(
        &self,
    ) -> &worth_query_admission::facade::graph_read_access::WorthQueryGraphReadPlanReview {
        self.graph_work.graph_read_review()
    }

    pub const fn canonical_work(&self) -> WorthQueryCanonicalWorkPhases {
        self.canonical_work
    }

    pub fn continuation_index_id(&self) -> Option<DerivedIndexId> {
        self.continuation_index_id
    }

    pub fn basis_identity(&self) -> &WorthQueryApplicationBasisIdentity {
        self.graph_work
            .query_basis()
            .expect("an application-query graph-work session retains its exact basis")
    }

    pub fn graph_work_session_identity(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryGraphWorkSessionIdentity {
        self.graph_work.identity()
    }

    pub fn graph_work_managed_run_identity(
        &self,
    ) -> crate::domain_computation::provider_session::WorthQueryGraphWorkManagedRunIdentity {
        self.graph_work.managed_run_identity()
    }

    pub fn graph_work_branch(&self) -> &worth_relational::facade::history::BranchId {
        self.graph_work.branch().relational()
    }

    pub fn graph_work_decision_fact_count(&self) -> usize {
        self.graph_work.retained_decision_facts()
    }

    pub fn graph_work_runtime_ordinal(&self) -> u64 {
        self.graph_work.runtime_ordinal()
    }

    pub fn graph_work_principal_entity_id(&self) -> worth_relational::facade::identity::EntityId {
        self.graph_work.principal()
    }

    pub fn graph_work_scope_entity_id(
        &self,
    ) -> Option<worth_relational::facade::identity::EntityId> {
        self.graph_work.entity_access_context()
    }

    pub fn graph_work_capability_identity(&self) -> Option<[u8; 32]> {
        self.graph_work.capability_access_context()
    }

    pub fn graph_work_provider(&self) -> &str {
        self.graph_work.provider()
    }

    pub fn principal(
        &self,
    ) -> &WorthQueryAuthenticatedPrincipal<Schema, Principal, PrincipalIdentity> {
        self.principal
    }

    pub fn scope(&self) -> &WorthQueryApplicationEntityIdentity<Schema, Scope> {
        self.scope
    }

    pub fn runtime_authority_identity(&self) -> u64 {
        self.runtime_authority.as_u64()
    }

    pub fn authorization_decision_fact_count(&self) -> usize {
        self.authorization.exact_fact_count()
    }

    pub const fn authorization_work(
        &self,
    ) -> super::WorthQueryApplicationAuthorizationWorkEvidence {
        self.authorization_work
    }

    pub(super) fn take_governance(
        &mut self,
    ) -> super::disclosure::WorthQueryApplicationQueryGovernance {
        std::mem::replace(
            &mut self.governance,
            super::disclosure::WorthQueryApplicationQueryGovernance::Public,
        )
    }
}
