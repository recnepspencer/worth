use crate::authoring::WorthQueryGraphReadDomainOperationDeclaration;
use crate::authorized_projection::reconcile_authorized_declarative_projection;
use crate::basis::{BasisAuthorityFamily, ExecutionBasisIntent, SnapshotLineageClass};
use crate::canonicalization::CanonicalQueryBundle;
use crate::declarative_live::DeclarativeLiveQueryRequest;
use crate::domain_installation::WorthQueryInstalledGraphReadOperation;
use crate::domain_installation::WorthQueryInstalledGraphReadOperationBindingDenial;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::identity::{CanonicalQueryDigest, CanonicalResultShapeDigest, SchemaBasisDigest};
use crate::planning::{
    plan_validated_bundle, plan_validated_bundle_for_count_aggregate,
    plan_validated_bundle_for_count_aggregate_with_policy_authority,
    plan_validated_bundle_with_policy_authority, planning_request_context_for_direct,
};
use crate::policy_narrowing::NarrowedPolicyQueryArtifact;
use crate::policy_plan::lower_policy_aware_current_plan;
use crate::relationship_proof::RelationshipProofAdmission;
use crate::runtime::{
    WorthQueryReadBuiltInOperator, WorthQueryReadDenial, WorthQueryReadDenialKind,
    WorthQueryReadGraph, WorthQueryReadGraphFamily, WorthQueryReadScopeClass,
};
use crate::schema_view::QuerySchemaView;
use crate::validation::ValidatedQueryBundle;

/// Query-owned read meaning after canonicalization and validation but before
/// authority admission or planning.
///
/// This type is present in the inferred builder signature only. Consumers
/// cannot inspect or manufacture its canonical, validated, or schema-bearing
/// internals, and only Query's context handoff may plan it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryDeclaredReadIntent {
    digest: String,
    family: WorthQueryReadGraphFamily,
    scope_class: WorthQueryReadScopeClass,
    schema_basis: SchemaBasisDigest,
    built_in_operators: Vec<WorthQueryReadBuiltInOperator>,
    domain_graph_operations: Vec<WorthQueryGraphReadDomainOperationDeclaration>,
    installed_operation_bindings: Vec<WorthQueryInstalledGraphReadOperation>,
    declared_traversal_clause_count: usize,
    declared_traversal_depth_limit: usize,
    declarative_request: DeclarativeLiveQueryRequest,
    schema_view: QuerySchemaView,
    canonical: CanonicalQueryBundle,
    validated: ValidatedQueryBundle,
}

pub(crate) struct WorthQueryDeclaredReadMeaning {
    pub(crate) family: WorthQueryReadGraphFamily,
    pub(crate) scope_class: WorthQueryReadScopeClass,
    pub(crate) schema_basis: SchemaBasisDigest,
}

pub(crate) struct WorthQueryDeclaredReadOperations {
    pub(crate) built_in: Vec<WorthQueryReadBuiltInOperator>,
    pub(crate) domain: Vec<WorthQueryGraphReadDomainOperationDeclaration>,
}

pub(crate) struct WorthQueryDeclaredTraversalContract {
    pub(crate) clause_count: usize,
    pub(crate) depth_limit: usize,
}

pub(crate) struct WorthQueryDeclaredReadArtifacts {
    pub(crate) request: DeclarativeLiveQueryRequest,
    pub(crate) schema_view: QuerySchemaView,
    pub(crate) canonical: CanonicalQueryBundle,
    pub(crate) validated: ValidatedQueryBundle,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorthQueryReadPlanningAuthority {
    Canonical {
        relationship_proof: Option<RelationshipProofAdmission>,
    },
    PolicyNarrowed(Box<NarrowedPolicyQueryArtifact>),
}

impl WorthQueryReadPlanningAuthority {
    pub(crate) fn canonical(relationship_proof: Option<RelationshipProofAdmission>) -> Self {
        Self::Canonical { relationship_proof }
    }

    pub(crate) fn policy_narrowed(artifact: NarrowedPolicyQueryArtifact) -> Self {
        Self::PolicyNarrowed(Box::new(artifact))
    }
}

impl WorthQueryDeclaredReadIntent {
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
        self.installed_operation_bindings.dedup();
        Ok(self)
    }
    pub(crate) fn digest(&self) -> &str {
        &self.digest
    }

    pub(crate) fn canonical_query_digest(&self) -> &CanonicalQueryDigest {
        self.canonical.query().digest()
    }

    pub(crate) fn canonical_result_shape_digest(&self) -> &CanonicalResultShapeDigest {
        self.canonical.result_shape().digest()
    }

    pub(crate) fn declarative_request(&self) -> &DeclarativeLiveQueryRequest {
        &self.declarative_request
    }

    pub(crate) fn family(&self) -> &WorthQueryReadGraphFamily {
        &self.family
    }

    pub(crate) fn schema_basis(&self) -> &SchemaBasisDigest {
        &self.schema_basis
    }

    pub(crate) fn canonical(&self) -> &CanonicalQueryBundle {
        &self.canonical
    }

    pub(crate) fn validated(&self) -> &ValidatedQueryBundle {
        &self.validated
    }

    pub(crate) fn built_in_operators(&self) -> &[WorthQueryReadBuiltInOperator] {
        &self.built_in_operators
    }

    pub(crate) fn requires_relationship_proof(&self) -> bool {
        self.declared_traversal_clause_count > 0
    }

    pub(crate) fn plan(
        self,
        authority: WorthQueryReadPlanningAuthority,
    ) -> Result<WorthQueryReadGraph, WorthQueryReadDenial> {
        self.plan_result_family(
            authority,
            WorthQueryDeclaredReadResultFamily::Rows,
            ExecutionBasisIntent::new(
                BasisAuthorityFamily::Runtime,
                SnapshotLineageClass::CurrentHead,
                false,
            ),
        )
    }

    pub(crate) fn plan_runtime_historical(
        self,
        authority: WorthQueryReadPlanningAuthority,
    ) -> Result<WorthQueryReadGraph, WorthQueryReadDenial> {
        self.plan_result_family(
            authority,
            WorthQueryDeclaredReadResultFamily::Rows,
            ExecutionBasisIntent::new(
                BasisAuthorityFamily::Runtime,
                SnapshotLineageClass::ReplayEquivalent,
                false,
            ),
        )
    }

    pub(crate) fn plan_count(
        self,
        authority: WorthQueryReadPlanningAuthority,
    ) -> Result<WorthQueryReadGraph, WorthQueryReadDenial> {
        self.plan_result_family(
            authority,
            WorthQueryDeclaredReadResultFamily::CountRows,
            ExecutionBasisIntent::new(
                BasisAuthorityFamily::Runtime,
                SnapshotLineageClass::CurrentHead,
                false,
            ),
        )
    }

    fn plan_result_family(
        self,
        authority: WorthQueryReadPlanningAuthority,
        result_family: WorthQueryDeclaredReadResultFamily,
        basis_intent: ExecutionBasisIntent,
    ) -> Result<WorthQueryReadGraph, WorthQueryReadDenial> {
        let mut declarative_request = self.declarative_request;
        let (relationship_proof_admission, policy_aware_plan, authorized_projection) =
            match authority {
                WorthQueryReadPlanningAuthority::Canonical { relationship_proof } => {
                    (relationship_proof, None, None)
                }
                WorthQueryReadPlanningAuthority::PolicyNarrowed(artifact) => {
                    let authorized_request_projection =
                        reconcile_authorized_declarative_projection(
                            &declarative_request,
                            artifact.authorized_projection(),
                        )
                        .map_err(planning_denial)?;
                    declarative_request = declarative_request
                        .with_authorized_query_projection(authorized_request_projection);
                    let relationship_proof = Some(artifact.relationship_proof().clone());
                    let plan = Some(lower_policy_aware_current_plan(&artifact));
                    let authorized_projection = Some(artifact.authorized_projection().clone());
                    (relationship_proof, plan, authorized_projection)
                }
            };
        let request_context = planning_request_context_for_direct(&self.validated, basis_intent)
            .map_err(planning_denial)?;
        let execution_plan = match (result_family, policy_aware_plan.as_ref()) {
            (WorthQueryDeclaredReadResultFamily::Rows, Some(policy_plan)) => {
                plan_validated_bundle_with_policy_authority(
                    &self.validated,
                    request_context,
                    policy_plan,
                )
            }
            (WorthQueryDeclaredReadResultFamily::Rows, None) => {
                plan_validated_bundle(&self.validated, request_context)
            }
            (WorthQueryDeclaredReadResultFamily::CountRows, Some(policy_plan)) => {
                plan_validated_bundle_for_count_aggregate_with_policy_authority(
                    &self.validated,
                    request_context,
                    policy_plan,
                )
            }
            (WorthQueryDeclaredReadResultFamily::CountRows, None) => {
                plan_validated_bundle_for_count_aggregate(&self.validated, request_context)
            }
        }
        .map_err(planning_denial)?;
        Ok(WorthQueryReadGraph::new(
            self.family,
            self.scope_class,
            self.schema_basis,
            self.built_in_operators,
            self.domain_graph_operations,
            self.installed_operation_bindings,
            self.declared_traversal_clause_count,
            self.declared_traversal_depth_limit,
            relationship_proof_admission,
            policy_aware_plan,
            authorized_projection,
            self.canonical,
            self.validated,
            declarative_request,
            self.schema_view,
            execution_plan,
        ))
    }

    pub(crate) fn new(
        meaning: WorthQueryDeclaredReadMeaning,
        operations: WorthQueryDeclaredReadOperations,
        traversal: WorthQueryDeclaredTraversalContract,
        artifacts: WorthQueryDeclaredReadArtifacts,
    ) -> Result<Self, IdentityDerivationDenial> {
        let digest = worth_query_evidence_identity(WorthQueryEvidenceScope::ReadGraphDigest)
            .field_shape(WorthQueryEvidenceTag::new("stage"), "declared_read_intent")
            .field_value(
                WorthQueryEvidenceTag::new("canonical_query"),
                artifacts.canonical.query().digest().as_str(),
            )
            .field_value(
                WorthQueryEvidenceTag::new("canonical_result_shape"),
                artifacts.canonical.result_shape().digest().as_str(),
            )
            .field_shape(
                WorthQueryEvidenceTag::new("scope"),
                meaning.scope_class.as_str(),
            )
            .field_value_sequence(
                WorthQueryEvidenceTag::new("built_in_operator"),
                operations
                    .built_in
                    .iter()
                    .map(WorthQueryReadBuiltInOperator::as_str),
            )
            .seal()?
            .as_str()
            .to_string();
        Ok(Self {
            digest,
            family: meaning.family,
            scope_class: meaning.scope_class,
            schema_basis: meaning.schema_basis,
            built_in_operators: operations.built_in,
            domain_graph_operations: operations.domain,
            installed_operation_bindings: Vec::new(),
            declared_traversal_clause_count: traversal.clause_count,
            declared_traversal_depth_limit: traversal.depth_limit,
            declarative_request: artifacts.request,
            schema_view: artifacts.schema_view,
            canonical: artifacts.canonical,
            validated: artifacts.validated,
        })
    }
}

#[derive(Clone, Copy)]
enum WorthQueryDeclaredReadResultFamily {
    Rows,
    CountRows,
}

fn planning_denial(error: impl std::fmt::Debug) -> WorthQueryReadDenial {
    WorthQueryReadDenial::new(
        WorthQueryReadDenialKind::PlanningDenied,
        format!("{error:?}"),
    )
}
