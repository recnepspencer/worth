//! Contribution-owned application handler and invariant configuration.
pub use crate::domain_computation::primary_graph::{
    WorthQueryDerivedRelationCountOwner, WorthQueryFeatureConfiguration,
    WorthQueryFeatureDefinition, WorthQueryFeaturePairConfiguration, WorthQueryFeatureRoleRuntime,
    WorthQueryFeatureRuntime, WorthQueryRequiredFeatureContract, WorthQueryRequiredFeaturePair,
};
pub use worth_foundational::facade::ProducerPublicationOwnership;

pub use {
    crate::domain_computation::request_execution::WorthQueryAdvancementDenial,
    crate::domain_computation::request_execution::WorthQueryAdvancementPhase,
    crate::domain_computation::request_execution::WorthQueryForeignAdvancementPhase,
};

pub use crate::domain_computation::primary_graph::computation_retention::full_cause::WorthQueryPartitionedComputationFullCause;
/// How a partitioned computation's run ran, for the test observer only.
pub use crate::domain_computation::primary_graph::{
    discarded_computation_retention_on_this_thread_for_test,
    published_partitioned_computations_on_this_thread_for_test,
    WorthQueryPartitionedComputationRunForTest, WorthQueryPartitionedTreeAccounting,
    WorthQueryPartitionedTreeMetrics, WorthQueryPartitionedTreeRebuildCause,
    WorthQueryPartitionedTreeRun, WorthQueryPublishedComputationStateForTest,
};
/// What a partitioned owner's results and denials declare to execution, the
/// identity of a planned item, and execution's refusal before dispatch.
pub use worth_execution::{CanonicalBits, ChargedBytes, PartitionItemId};
pub use worth_query_declaration::facade::application_schema::ApplicationSchemaComposition;
pub use {
    crate::domain_computation::primary_graph::vocabulary::application_contribution::checkpoint::WorthQueryManagedComputationCheckpoint,
    crate::domain_computation::primary_graph::vocabulary::application_contribution::checkpoint::WorthQueryManagedComputationExecution,
    crate::domain_computation::primary_graph::vocabulary::application_contribution::producer::WorthQueryProducerApplicability,
    crate::domain_computation::primary_graph::vocabulary::application_contribution::producer::WorthQueryProducerInvariantRequirement,
    crate::domain_computation::primary_graph::vocabulary::application_contribution::producer::WorthQueryProducerLifecyclePosture,
    crate::domain_computation::primary_graph::vocabulary::application_contribution::producer::WorthQueryWorkflowAssessmentPosture,
    crate::domain_computation::primary_graph::WorthQueryApplicationConditionalBinding,
    crate::domain_computation::primary_graph::WorthQueryApplicationConditionalPackageContract,
    crate::domain_computation::primary_graph::WorthQueryApplicationConditionalProducerAccess,
    crate::domain_computation::primary_graph::WorthQueryApplicationContractCatalog,
    crate::domain_computation::primary_graph::WorthQueryApplicationContribution,
    crate::domain_computation::primary_graph::WorthQueryApplicationContributionContracts,
    crate::domain_computation::primary_graph::WorthQueryApplicationContributionSetup,
    crate::domain_computation::primary_graph::WorthQueryApplicationContributionTuple,
    crate::domain_computation::primary_graph::WorthQueryApplicationOutputDemand,
    crate::domain_computation::primary_graph::WorthQueryApplicationProducerBinding,
    crate::domain_computation::primary_graph::WorthQueryApplicationProducerProvider,
    crate::domain_computation::primary_graph::WorthQueryCompletedManagedComputation,
    crate::domain_computation::primary_graph::WorthQueryCompletedPartitionedComputation,
    crate::domain_computation::primary_graph::WorthQueryComputationInputDenial,
    crate::domain_computation::primary_graph::WorthQueryComputationPartitionMembers,
    crate::domain_computation::primary_graph::WorthQueryComputationPartitionPlan,
    crate::domain_computation::primary_graph::WorthQueryComputationPartitionStop,
    crate::domain_computation::primary_graph::WorthQueryComputationPartitionView,
    crate::domain_computation::primary_graph::WorthQueryComputationReadDenial,
    crate::domain_computation::primary_graph::WorthQueryComputationReader,
    crate::domain_computation::primary_graph::WorthQueryConfiguredApplicationContributions,
    crate::domain_computation::primary_graph::WorthQueryDeterministicReducer,
    crate::domain_computation::primary_graph::WorthQueryInstalledApplicationConditionalRegistry,
    crate::domain_computation::primary_graph::WorthQueryInstalledApplicationProducerRegistry,
    crate::domain_computation::primary_graph::WorthQueryInstalledManagedComputation,
    crate::domain_computation::primary_graph::WorthQueryInstalledPartitionedComputation,
    crate::domain_computation::primary_graph::WorthQueryManagedComputationDenial,
    crate::domain_computation::primary_graph::WorthQueryManagedComputationOwner,
    crate::domain_computation::primary_graph::WorthQueryManagedIndexProviderRegistration,
    crate::domain_computation::primary_graph::WorthQueryOutputReadinessContract,
    crate::domain_computation::primary_graph::WorthQueryOutputReadinessContractBuilder,
    crate::domain_computation::primary_graph::WorthQueryPartitionedComputationDenial,
    crate::domain_computation::primary_graph::WorthQueryPartitionedComputationOwner,
    crate::domain_computation::primary_graph::WorthQueryPreparedManagedComputation,
    crate::domain_computation::primary_graph::WorthQueryPreparedPartitionedComputation,
    crate::domain_computation::primary_graph::WorthQueryProducerDenialDeclaration,
    crate::domain_computation::primary_graph::WorthQueryProducerDenialProjection,
    crate::domain_computation::primary_graph::WorthQueryProducerOutputEquivalence,
    crate::domain_computation::primary_graph::WorthQueryProducerOutputFamily,
    crate::domain_computation::primary_graph::WorthQueryProjectedProducerDenial,
    crate::domain_computation::primary_graph::WorthQueryReadinessFieldUse,
    crate::domain_computation::primary_graph::WorthQueryReductionInputDenial,
    crate::domain_computation::primary_graph::WorthQueryUndeclaredProducerDenial,
    crate::domain_computation::primary_graph::WorthQueryWorkflowAssessmentOutputFamily,
    crate::domain_computation::request_execution::execution_refusal::WorthQueryManagedComputationCheckpointDenial,
    crate::domain_computation::request_execution::execution_refusal::WorthQueryManagedComputationInterruption,
    crate::domain_computation::vocabulary::primary_graph::input_reuse_contract::WorthQueryDecisionContextDependencies,
    crate::domain_computation::vocabulary::primary_graph::input_reuse_contract::WorthQueryProducerInputReuseContract,
    crate::domain_computation::vocabulary::primary_graph::output_readiness_contract::WorthQueryOutputReadinessContractDenial,
    crate::domain_computation::vocabulary::request_execution::execution_refusal::WorthQueryManagedComputationResourceDenial,
    crate::domain_computation::vocabulary::request_execution::execution_refusal::WorthQueryMemoryLimitLevel,
};

pub use {
    crate::domain_computation::request_execution::with_bootstrap_advancement,
    crate::domain_computation::request_execution::with_bootstrap_advancement_from_request,
};

pub use crate::domain_computation::request_execution::WorthQueryBootstrapAdvancementPhase;

pub use crate::domain_computation::primary_graph::vocabulary::application_contribution::member_execution::WorthQueryMemberExecution;
