use crate::authoring::WorthQueryGraphReadDomainOperationDeclaration;
use crate::authorized_projection::AuthorizedProjectionArtifact;
use crate::basis::QuerySchemaBasisAuthority;
use crate::canonicalization::CanonicalQueryBundle;
use crate::declarative_live::DeclarativeLiveQueryRequest;
use crate::domain_installation::{
    WorthQueryInstalledDomainAuthorityWitness, WorthQueryInstalledGraphReadOperation,
    WorthQueryInstalledGraphReadOperationBindingDenial,
};
use crate::evidence_identity::WorthQueryEvidenceIdentity;
use crate::identity::SchemaBasisDigest;
use crate::intent_admission::WorthQueryIntentDecisionTraceEnvelope;
use crate::memory_workspace::WorthQuerySnapshotIdentity;
use crate::planning::ExecutionPlanBundle;
use crate::policy_plan::PolicyAwareCurrentPlan;
use crate::projection_consumption::ProjectionMaterializedFactPosture;
use crate::relationship_proof::{RelationshipProofAdmission, RelationshipProofSupportProfile};
use crate::runtime::WorthQueryIntentExecutionProvenance;
use crate::schema_view::QuerySchemaView;
use crate::validation::ValidatedQueryBundle;
use worth_query_installation::facade::WorthQueryApplicationCanonicalArtifact;

use super::read_graph_identity::derive_read_graph_identity;
use super::read_graph_planning_contract::{
    prepare_mature_read_graph_planning_basis, WorthQueryReadGraphPlanningPreparation,
};
use super::{WorthQueryReadBreadth, WorthQueryReadBuiltInOperator, WorthQueryReadOperatorFamily};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryReadScopeClass {
    LocalNeighborhood,
    AnchoredExpansion,
    ExplicitBroadSearch,
}

impl WorthQueryReadScopeClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::LocalNeighborhood => "local_neighborhood",
            Self::AnchoredExpansion => "anchored_expansion",
            Self::ExplicitBroadSearch => "explicit_broad_search",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryReadGraphFamily {
    Detail,
    Collection,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryReadExecutionEngine {
    QueryRuntimeCurrent,
    QueryRuntimeBranch,
    QueryRuntimeHistorical,
    QueryRuntimePreviewDerived,
}

impl WorthQueryReadExecutionEngine {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::QueryRuntimeCurrent => "query-runtime-current",
            Self::QueryRuntimeBranch => "query-runtime-branch",
            Self::QueryRuntimeHistorical => "query-runtime-historical",
            Self::QueryRuntimePreviewDerived => "query-runtime-preview-derived",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryReadFallbackClass {
    None,
    SnapshotIndexedDebt,
    WholeViewDebt,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryReadRelationshipProofPosture {
    NotRequired,
    DescriptorAdmittedSyntheticRuntime,
}

impl WorthQueryReadRelationshipProofPosture {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NotRequired => "not_required",
            Self::DescriptorAdmittedSyntheticRuntime => "descriptor_admitted_synthetic_runtime",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryReadGraph {
    identity: WorthQueryEvidenceIdentity,
    canonical_planning: WorthQueryApplicationCanonicalArtifact,
    family: WorthQueryReadGraphFamily,
    scope_class: WorthQueryReadScopeClass,
    schema_basis: SchemaBasisDigest,
    built_in_operators: Vec<WorthQueryReadBuiltInOperator>,
    domain_graph_operations: Vec<WorthQueryGraphReadDomainOperationDeclaration>,
    installed_operation_bindings: Vec<WorthQueryInstalledGraphReadOperation>,
    declared_traversal_clause_count: usize,
    declared_traversal_depth_limit: usize,
    relationship_proof_admission: Option<RelationshipProofAdmission>,
    policy_aware_plan: Option<PolicyAwareCurrentPlan>,
    authorized_projection: Option<AuthorizedProjectionArtifact>,
    canonical: CanonicalQueryBundle,
    validated: ValidatedQueryBundle,
    declarative_request: DeclarativeLiveQueryRequest,
    schema_view: QuerySchemaView,
    execution_plan: ExecutionPlanBundle,
}

impl WorthQueryReadGraph {
    pub fn digest(&self) -> &str {
        self.identity.as_str()
    }

    pub fn evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.identity
    }

    pub fn canonical_planning_basis(&self) -> &WorthQueryApplicationCanonicalArtifact {
        &self.canonical_planning
    }

    pub fn family(&self) -> &WorthQueryReadGraphFamily {
        &self.family
    }

    pub fn scope_class(&self) -> &WorthQueryReadScopeClass {
        &self.scope_class
    }

    pub fn schema_basis(&self) -> &SchemaBasisDigest {
        &self.schema_basis
    }

    pub fn schema_basis_authority(&self) -> QuerySchemaBasisAuthority {
        self.schema_view.basis_authority()
    }

    pub fn built_in_operators(&self) -> &[WorthQueryReadBuiltInOperator] {
        &self.built_in_operators
    }

    pub fn domain_graph_operations(&self) -> &[WorthQueryGraphReadDomainOperationDeclaration] {
        &self.domain_graph_operations
    }

    pub(crate) fn installed_operation_authority(
        &self,
        declaration: &WorthQueryGraphReadDomainOperationDeclaration,
    ) -> Option<&WorthQueryInstalledDomainAuthorityWitness> {
        self.installed_operation_bindings
            .iter()
            .find(|binding| binding.declaration() == declaration)
            .map(WorthQueryInstalledGraphReadOperation::authority)
    }

    pub(crate) fn bind_installed_operation(
        mut self,
        binding: WorthQueryInstalledGraphReadOperation,
    ) -> Result<Self, WorthQueryInstalledGraphReadOperationBindingDenial> {
        if !self
            .domain_graph_operations
            .iter()
            .any(|declaration| declaration == binding.declaration())
        {
            return Err(
                WorthQueryInstalledGraphReadOperationBindingDenial::DeclarationMissingFromReadGraph,
            );
        }
        if let Some(existing) = self
            .installed_operation_bindings
            .iter()
            .find(|existing| existing.declaration() == binding.declaration())
        {
            return if existing == &binding {
                Ok(self)
            } else {
                Err(WorthQueryInstalledGraphReadOperationBindingDenial::
                    ConflictingInstalledAuthority)
            };
        }
        self.installed_operation_bindings.push(binding);
        self.installed_operation_bindings
            .sort_by_key(|binding| binding.declaration().digest_part());
        self.identity = derive_read_graph_identity(
            &self.family,
            &self.scope_class,
            &self.built_in_operators,
            &self.domain_graph_operations,
            &self.installed_operation_bindings,
            self.declared_traversal_clause_count,
            self.declared_traversal_depth_limit,
            self.relationship_proof_admission.as_ref(),
            self.policy_aware_plan.as_ref(),
            &self.execution_plan,
        );
        Ok(self)
    }

    pub fn query_digest(&self) -> &str {
        self.execution_plan
            .query()
            .validated_query_digest()
            .as_str()
    }

    pub fn declared_traversal_clause_count(&self) -> usize {
        self.declared_traversal_clause_count
    }

    pub fn declared_traversal_depth_limit(&self) -> usize {
        self.declared_traversal_depth_limit
    }

    pub fn execution_plan(&self) -> &ExecutionPlanBundle {
        &self.execution_plan
    }

    pub(crate) fn canonical(&self) -> &CanonicalQueryBundle {
        &self.canonical
    }

    pub(crate) fn validated(&self) -> &ValidatedQueryBundle {
        &self.validated
    }

    pub fn declarative_request(&self) -> &DeclarativeLiveQueryRequest {
        &self.declarative_request
    }

    pub fn relationship_proof_admission(&self) -> Option<&RelationshipProofAdmission> {
        self.relationship_proof_admission.as_ref()
    }

    pub(crate) fn policy_aware_plan(&self) -> Option<&PolicyAwareCurrentPlan> {
        self.policy_aware_plan.as_ref()
    }

    pub(crate) fn authorized_projection(&self) -> Option<&AuthorizedProjectionArtifact> {
        self.authorized_projection.as_ref()
    }

    pub fn schema_view(&self) -> &QuerySchemaView {
        &self.schema_view
    }

    pub fn operator_families(&self) -> Vec<WorthQueryReadOperatorFamily> {
        let query = self.execution_plan.query();
        let mut families = Vec::new();
        if query.projection_count() > 0 {
            families.push(WorthQueryReadOperatorFamily::Projection);
        }
        if query.traversal_count() > 0 {
            families.push(WorthQueryReadOperatorFamily::Traversal);
        }
        if query.predicate_count() > 0 {
            families.push(WorthQueryReadOperatorFamily::Predicate);
        }
        if query.ordering_count() > 0 {
            families.push(WorthQueryReadOperatorFamily::Ordering);
        }
        families
    }

    pub(crate) fn new(
        family: WorthQueryReadGraphFamily,
        scope_class: WorthQueryReadScopeClass,
        schema_basis: SchemaBasisDigest,
        built_in_operators: Vec<WorthQueryReadBuiltInOperator>,
        domain_graph_operations: Vec<WorthQueryGraphReadDomainOperationDeclaration>,
        installed_operation_bindings: Vec<WorthQueryInstalledGraphReadOperation>,
        declared_traversal_clause_count: usize,
        declared_traversal_depth_limit: usize,
        relationship_proof_admission: Option<RelationshipProofAdmission>,
        policy_aware_plan: Option<PolicyAwareCurrentPlan>,
        authorized_projection: Option<AuthorizedProjectionArtifact>,
        canonical: CanonicalQueryBundle,
        validated: ValidatedQueryBundle,
        declarative_request: DeclarativeLiveQueryRequest,
        schema_view: QuerySchemaView,
        execution_plan: ExecutionPlanBundle,
    ) -> Self {
        let identity = derive_read_graph_identity(
            &family,
            &scope_class,
            &built_in_operators,
            &domain_graph_operations,
            &installed_operation_bindings,
            declared_traversal_clause_count,
            declared_traversal_depth_limit,
            relationship_proof_admission.as_ref(),
            policy_aware_plan.as_ref(),
            &execution_plan,
        );
        let canonical_planning =
            prepare_mature_read_graph_planning_basis(&WorthQueryReadGraphPlanningPreparation {
                schema_basis: &schema_basis,
                built_in_operators: &built_in_operators,
                declared_traversal_depth_limit,
                canonical: &canonical,
                validated: &validated,
            });
        Self {
            identity,
            canonical_planning,
            family,
            scope_class,
            schema_basis,
            built_in_operators,
            domain_graph_operations,
            installed_operation_bindings,
            declared_traversal_clause_count,
            declared_traversal_depth_limit,
            relationship_proof_admission,
            policy_aware_plan,
            authorized_projection,
            canonical,
            validated,
            declarative_request,
            schema_view,
            execution_plan,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryReadReceipt {
    pub(super) read_graph_identity: WorthQueryEvidenceIdentity,
    pub(super) graph_family: WorthQueryReadGraphFamily,
    pub(super) collection_result_family: Option<crate::collection::CollectionResultFamily>,
    pub(super) execution_plan_evidence_identity: WorthQueryEvidenceIdentity,
    pub(super) canonical_query_digest: String,
    pub(super) query_digest: String,
    pub(super) basis_digest: String,
    pub(super) result_digest: String,
    pub(super) snapshot_identity: WorthQuerySnapshotIdentity,
    pub(super) scope_class: WorthQueryReadScopeClass,
    pub(super) execution_engine: WorthQueryReadExecutionEngine,
    pub(super) fallback_class: WorthQueryReadFallbackClass,
    pub(super) fallback_count: usize,
    pub(super) operator_families: Vec<WorthQueryReadOperatorFamily>,
    pub(super) built_in_operator_coverage: Vec<WorthQueryReadBuiltInOperator>,
    pub(super) relationship_proof_posture: WorthQueryReadRelationshipProofPosture,
    pub(super) relationship_proof_admission: Option<RelationshipProofAdmission>,
    pub(super) relationship_proof_support_profile: Option<RelationshipProofSupportProfile>,
    pub(super) policy_narrowing_digest: Option<String>,
    pub(super) policy_aware_plan_digest: Option<String>,
    pub(super) policy_execution_seam_identity: Option<String>,
    pub(super) policy_executor_semantic_rediscovery_count: usize,
    pub(super) breadth: WorthQueryReadBreadth,
    pub(super) materialized_fact_posture: Option<ProjectionMaterializedFactPosture>,
    pub(super) graph_read_access_plan:
        Option<crate::runtime::WorthQueryAdmittedGraphReadAccessPlan>,
    pub(super) graph_read_access_plan_consumption:
        Option<crate::runtime::WorthQueryGraphReadAccessPlanConsumption>,
    pub(super) graph_read_streaming_receipt:
        Option<crate::runtime::WorthQueryGraphReadStreamingReceipt>,
    pub(super) graph_read_access_summary:
        Option<crate::runtime::WorthQueryGraphReadAccessReceiptSummary>,
    pub(super) graph_read_access_complexity_counters:
        Option<crate::runtime::WorthQueryGraphReadAccessComplexityCounters>,
    pub(super) decision_trace_envelope: Option<WorthQueryIntentDecisionTraceEnvelope>,
    pub(super) execution_provenance: Option<WorthQueryIntentExecutionProvenance>,
}
