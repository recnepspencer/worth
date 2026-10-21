mod access_admission;
mod access_authority_context;
mod access_requirements;
mod access_shape;
mod basis_binding;
mod boolean_expression;
mod cost_model;
mod ephemeral_index_provisioning;
mod explanation_api;
mod graph_index_inventory;
mod installed_explanation;
mod live_maintenance;
mod operation_resolution;
mod persistent_index_requirement;
mod schema_reference_admission;
mod schema_reference_evidence;
mod selectivity_normalization;
mod selectivity_shape;
mod selectivity_vocabulary;
mod shape_derivation;
mod shape_explanation;
mod vocabulary;

pub(crate) use access_admission::admit_graph_read_access_for_family_in_authority_with_inventory_and_lookup;
#[cfg(test)]
pub(crate) use access_admission::{
    admit_graph_read_access_for_family, plan_admitted_graph_read_access_for_family,
};
#[cfg(test)]
pub(crate) use access_authority_context::admit_graph_read_access_authority_from_policy_tenant_request;
pub(crate) use boolean_expression::admit_boolean_predicate_expression_for_read_graph;
pub(crate) use installed_explanation::{
    explain_boolean_selectivity_shape_for_family_in_authority_with_lookup,
    explain_graph_read_access_requirements_for_family_in_authority_with_lookup,
    explain_graph_read_access_shape_for_family_in_authority_with_lookup,
};
pub(crate) use operation_resolution::resolve_graph_read_operations_for_read_graph;
pub(crate) use operation_resolution::WorthQueryGraphReadOperationLookup;
pub(crate) use schema_reference_admission::admit_query_schema_references_for_read_graph;
pub(crate) use selectivity_normalization::normalize_boolean_selectivity_for_access_shape;
pub(crate) use shape_derivation::derive_graph_read_access_shape;

pub(crate) use access_admission::WorthQueryGraphReadAccessExecutionRecorder;
pub use access_admission::{
    WorthQueryAdmittedGraphReadAccessPlan, WorthQueryGraphReadAccessAdmission,
    WorthQueryGraphReadAccessAdmissionPosture, WorthQueryGraphReadAccessCase,
    WorthQueryGraphReadAccessCaseRegistry, WorthQueryGraphReadAccessDenial,
    WorthQueryGraphReadAccessDenialKind, WorthQueryGraphReadAccessExecutionCounters,
    WorthQueryGraphReadAccessInventoryMatch, WorthQueryGraphReadAccessPlanConsumption,
    WorthQueryGraphReadAccessPlanExplanation, WorthQueryGraphReadPersistentArtifactAudit,
    WorthQueryGraphReadRequiredCapabilityOwner,
};
pub use access_authority_context::{
    admit_graph_read_access_authority, WorthQueryGraphReadAccessAuthorityContext,
    WorthQueryGraphReadAccessAuthorityCounters, WorthQueryGraphReadAccessAuthorityDenial,
    WorthQueryGraphReadAccessAuthorityDenialKind, WorthQueryGraphReadAccessAuthorityReceipt,
    WorthQueryGraphReadAccessAuthorityRequest, WorthQueryGraphReadAccessBasisScope,
    WorthQueryGraphReadAccessBasisScopeKind, WorthQueryGraphReadPolicyTenantAuthorityRequest,
};
pub use access_requirements::{
    WorthQueryGraphReadAccessComplexityContract, WorthQueryGraphReadAccessInvalidationBasis,
    WorthQueryGraphReadAccessMemoryEstimateBasis, WorthQueryGraphReadAccessRebuildBasis,
    WorthQueryGraphReadAccessRequirementCounters,
    WorthQueryGraphReadAccessRequirementDerivationError,
    WorthQueryGraphReadAccessRequirementExplanationOutcome,
    WorthQueryGraphReadAccessRequirementKind, WorthQueryGraphReadAccessRequirementRow,
    WorthQueryGraphReadAccessRequirementSet, WorthQueryGraphReadAccessRequirementSetDigest,
    WorthQueryGraphReadOrderingFieldAuthority, WorthQueryGraphReadPredicateFieldAuthority,
    WorthQueryGraphReadRelationAuthority,
};
pub use access_shape::WorthQueryGraphReadAccessShape;
pub use basis_binding::{
    WorthQueryGraphReadBasisBinding, WorthQueryGraphReadBasisPosture,
    WorthQueryGraphReadPolicyTenantPosture, WorthQueryGraphReadPolicyTenantProofBinding,
    WorthQueryGraphReadRelationshipProofBindingPosture,
};
pub use boolean_expression::{
    WorthQueryAdmittedBooleanExpressionBranch, WorthQueryAdmittedBooleanExpressionBranchKind,
    WorthQueryAdmittedBooleanExpressionCounters, WorthQueryAdmittedBooleanExpressionTopology,
    WorthQueryAdmittedBooleanPredicateExpression, WorthQueryAdmittedBooleanPredicateLeaf,
    WorthQueryBooleanExpressionAdmissionError, WorthQueryBooleanExpressionAdmissionErrorKind,
};
pub use cost_model::{
    derive_graph_read_cost_evidence, estimate_graph_read_access_cost,
    estimate_graph_read_access_cost_with_planning_observation,
    WorthQueryGraphReadAccessCostEstimate, WorthQueryGraphReadAccessCostEstimateDigest,
    WorthQueryGraphReadComplexityContract, WorthQueryGraphReadComplexityContractKind,
    WorthQueryGraphReadCostAttributionRow, WorthQueryGraphReadCostEstimateCounters,
    WorthQueryGraphReadCostEstimateStatus, WorthQueryGraphReadCostEstimateStatusKind,
    WorthQueryGraphReadCostEvidence, WorthQueryGraphReadIntrinsicCostContribution,
    WorthQueryGraphReadIntrinsicCostEstimate, WorthQueryGraphReadMemoryByteEstimate,
    WorthQueryGraphReadObservedCostEstimate, WorthQueryGraphReadPlanningObservation,
    WorthQueryGraphReadSupportedCostContribution, WorthQueryGraphReadSupportedCostEstimate,
};
pub use ephemeral_index_provisioning::{
    WorthQueryEphemeralGraphIndexAllocationRow, WorthQueryEphemeralGraphIndexPlan,
};
pub use explanation_api::{
    derive_graph_read_access_requirements, try_derive_graph_read_access_requirements,
    WorthQueryGraphReadAccessShapeExplanationError,
};
#[cfg(test)]
pub(crate) use explanation_api::{
    explain_boolean_selectivity_shape_for_family,
    explain_graph_read_access_requirement_outcome_for_family_with_operation_lookup,
    explain_graph_read_access_requirements_for_family,
    explain_graph_read_access_requirements_for_family_with_operation_lookup,
    explain_graph_read_access_shape_for_family,
    explain_graph_read_access_shape_for_family_with_operation_lookup,
    resolve_graph_read_operations_for_family_with_operation_lookup,
};
pub use graph_index_inventory::{
    match_current_graph_index_inventory_for_requirements, worth_query_graph_index_inventory,
    WorthQueryGraphIndexInventory, WorthQueryGraphIndexInventoryCounters,
    WorthQueryGraphIndexInventoryMatch, WorthQueryGraphIndexInventoryMatchOutcome,
    WorthQueryGraphIndexInventoryMatchReport, WorthQueryGraphIndexLifecycleClass,
    WorthQueryGraphIndexLifecycleOwner, WorthQueryGraphIndexPosture,
    WorthQueryGraphIndexSupportRow, WorthQueryGraphIndexSupportState,
};
pub use live_maintenance::{
    WorthQueryLiveGraphReadAccessDenial, WorthQueryLiveGraphReadAccessPlan,
    WorthQueryLiveGraphReadAccessPosture, WorthQueryLiveGraphReadAccessReceipt,
    WorthQueryLiveGraphReadMaintenanceBudget, WorthQueryLiveGraphReadMaintenanceCounters,
    WorthQueryLiveGraphReadMaintenanceReceipt, WorthQueryLiveGraphReadMutationDeltaScope,
};
pub use operation_resolution::{
    WorthQueryBuiltInGraphReadOperation, WorthQueryDomainRegisteredGraphReadOperation,
    WorthQueryGraphReadOperationCapabilityRequirement,
    WorthQueryGraphReadOperationCapabilityRequirementDeclaration,
    WorthQueryGraphReadOperationCapabilityRequirementKind, WorthQueryGraphReadOperationOutcome,
    WorthQueryGraphReadOperationResolution, WorthQueryGraphReadOperationUnsupportedDenial,
    WorthQueryGraphReadOperationUnsupportedDenialKind,
    WorthQueryGraphReadOperationUnsupportedShapeDeclaration, WorthQueryGraphReadResolvedOperation,
    WorthQueryGraphReadResolvedOperationFamily, WorthQueryGraphReadResolvedOperationKind,
};
pub(crate) use operation_resolution::{
    WorthQueryGraphReadOperationRegistration, WorthQueryGraphReadOperationRegistry,
    WorthQueryGraphReadRegistryAdmissionError,
};
pub use persistent_index_requirement::{
    WorthQueryGraphReadFamilyIndexContract, WorthQueryPersistentGraphIndexRequirementCounters,
    WorthQueryPersistentGraphIndexRequirementDeclaration,
    WorthQueryPersistentGraphIndexRequirementReceipt, WorthQueryPersistentGraphIndexRequirementRow,
};
pub use schema_reference_evidence::{
    WorthQueryAdmittedGraphReadOrderingField, WorthQueryAdmittedGraphReadPredicateField,
    WorthQueryAdmittedGraphReadProjectionField, WorthQueryAdmittedGraphReadRelation,
    WorthQueryAdmittedGraphReadRelationDirection, WorthQueryAdmittedNativeFieldFamily,
    WorthQueryAdmittedQuerySchemaReferences, WorthQueryGraphReadSchemaReferenceAdmissionError,
    WorthQueryGraphReadSchemaReferenceAdmissionErrorKind,
};
pub use selectivity_shape::{
    WorthQueryBooleanPredicateSelectivityRow, WorthQueryBooleanSelectivityBranch,
    WorthQueryBooleanSelectivityCounters, WorthQueryBooleanSelectivityShape,
};
pub use selectivity_vocabulary::{
    WorthQueryBooleanPredicateTopology, WorthQueryBooleanSelectivityAdmissionPosture,
    WorthQueryBooleanSelectivityBranchKind, WorthQueryBooleanSelectivityShapeDigest,
    WorthQueryPredicateAnchorPosture, WorthQueryPredicateOperandOperator,
    WorthQueryPredicateSelectivityClass, WorthQueryTraversalPredicateOrderingPosture,
};
pub use shape_explanation::{
    WorthQueryGraphReadAccessShapeDerivationCounters, WorthQueryGraphReadAccessShapeExplanation,
};
pub use vocabulary::{
    WorthQueryGraphReadAccessShapeDigest, WorthQueryGraphReadFanoutPosture,
    WorthQueryGraphReadLifecycleClass, WorthQueryGraphReadOrderingPosture,
    WorthQueryGraphReadPredicateFamily, WorthQueryGraphReadResultPressure,
    WorthQueryGraphReadRootPosture, WorthQueryGraphReadTraversalOperator,
};
