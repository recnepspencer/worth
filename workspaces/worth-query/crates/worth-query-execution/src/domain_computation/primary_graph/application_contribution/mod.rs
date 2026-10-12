mod composition;
pub(in crate::domain_computation::primary_graph) mod computation;
mod conditional;
mod contracts;
mod execution_denial;
mod partitioned_computation;
mod producer;
pub(in crate::domain_computation::primary_graph) use producer::OwnerPlan;
pub(in crate::domain_computation::primary_graph) use producer::{CurrentBasis, CurrentProof};
mod request_execution;
mod setup;

pub use composition::{
    WorthQueryApplicationContribution, WorthQueryApplicationContributionTuple,
    WorthQueryConfiguredApplicationContributions,
};
pub use computation::{
    WorthQueryCompletedManagedComputation, WorthQueryInstalledManagedComputation,
    WorthQueryManagedComputationCheckpoint, WorthQueryManagedComputationCheckpointDenial,
    WorthQueryManagedComputationDenial, WorthQueryManagedComputationExecution,
    WorthQueryManagedComputationInterruption, WorthQueryManagedComputationOwner,
    WorthQueryManagedComputationPrepared, WorthQueryManagedComputationResourceDenial,
    WorthQueryMemberExecution, WorthQueryMemoryLimitLevel, WorthQueryPreparedManagedComputation,
};
pub use conditional::{
    WorthQueryApplicationConditionalBinding, WorthQueryApplicationConditionalPackageContract,
    WorthQueryApplicationConditionalProducerAccess,
    WorthQueryInstalledApplicationConditionalRegistry, WorthQueryOutputReadinessContractBuilder,
    WorthQueryOutputReadinessContractDenial,
};
pub use contracts::{
    WorthQueryApplicationContractCatalog, WorthQueryApplicationContributionContracts,
};
pub use partitioned_computation::WorthQueryPartitionedComputationFullCause;
pub use partitioned_computation::{
    discarded_computation_retention_on_this_thread_for_test,
    full_partitioned_computation_preparations_on_this_thread_for_test,
    partitioned_computation_runs_on_this_thread_for_test,
    partitioned_computation_tree_work_on_this_thread_for_test,
    published_partitioned_computations_on_this_thread_for_test,
    WorthQueryPartitionedComputationRunForTest, WorthQueryPartitionedTreeMetrics,
    WorthQueryPartitionedTreeRebuildCause, WorthQueryPartitionedTreeRun,
    WorthQueryPublishedComputationStateForTest,
};
pub(in crate::domain_computation::primary_graph) use partitioned_computation::{
    Comparator, CompletedComputationRetention, ComputationDeposit, ComputationRetention,
};
pub(in crate::domain_computation) use partitioned_computation::{
    ComputationPrior, PriorAbsence, RetainedComputation, SealedComputationRetention,
    SealedComputationRun, Suppression,
};
pub use partitioned_computation::{
    WorthQueryCompletedPartitionedComputation, WorthQueryComputationInputDenial,
    WorthQueryComputationPartitionMembers, WorthQueryComputationPartitionPlan,
    WorthQueryComputationPartitionStop, WorthQueryComputationPartitionView,
    WorthQueryComputationReadDenial, WorthQueryComputationReader, WorthQueryDeterministicReducer,
    WorthQueryInstalledPartitionedComputation, WorthQueryPartitionedComputationDenial,
    WorthQueryPartitionedComputationOwner, WorthQueryPreparedPartitionedComputation,
    WorthQueryReductionInputDenial,
};
pub(in crate::domain_computation::primary_graph) use producer::{
    install_output_readiness_routes, InstalledProducerEdition, MatchedRequiredPredecessors,
    PendingOutputReadiness, TypedPendingOutputReadiness, WorthQueryInstalledOutputProducerRoutes,
    WorthQueryInstalledOutputReadinessRoutes, WorthQueryProducerCommitAuthority,
};
pub use producer::{
    WorthQueryAdmittedOutputDemand, WorthQueryApplicationOutputDemand,
    WorthQueryApplicationProducerBinding, WorthQueryApplicationProducerProvider,
    WorthQueryDecisionContextDependencies, WorthQueryInstalledApplicationProducerRegistry,
    WorthQueryOutputDemandAdvance, WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
    WorthQueryOutputDemandRecoveryPosture, WorthQueryProducerApplicability,
    WorthQueryProducerDemandResources, WorthQueryProducerInputReuseContract,
    WorthQueryProducerInvariantRequirement, WorthQueryProducerLifecyclePosture,
    WorthQueryProducerOutputFamily, WorthQuerySelectedApplicationProducer,
    WorthQueryWorkflowAssessmentOutputFamily, WorthQueryWorkflowAssessmentPosture,
};
pub use request_execution::query_test_execution_authority_for_test;
pub(in crate::domain_computation::primary_graph) use request_execution::QueryRequestExecution;
pub use request_execution::{
    bound_advancement_requests_on_this_thread_for_test,
    place_managed_computations_on_this_thread_for_test, test_execution_workers_for_test,
    WorthQueryExecutionPlacementForTest,
};
pub use request_execution::{
    WorthQueryAdvancementDenial, WorthQueryAdvancementPhase, WorthQueryForeignAdvancementPhase,
};
pub use setup::WorthQueryApplicationContributionSetup;

#[cfg(test)]
pub(in crate::domain_computation::primary_graph) use partitioned_computation::sealed_run_for_lineage_test;

pub(in crate::domain_computation) use partitioned_computation::observe_discarded;
pub(in crate::domain_computation::primary_graph) use partitioned_computation::observe_published;

pub(in crate::domain_computation::primary_graph) use request_execution::{
    with_serial_host_advancement, with_world_advancement,
};

pub use request_execution::{
    advancement_observations_on_this_thread_for_test, advancement_requests_on_this_thread_for_test,
    caller_pass_reports_on_this_thread_for_test, WorthQueryAdvancementEventForTest,
    WorthQueryAdvancementEventKindForTest, WorthQueryAdvancementIdentityForTest,
    WorthQueryAdvancementObservationForTest,
};

#[cfg(test)]
pub(crate) use request_execution::{
    drain_test_advancement_observations, with_test_advancement, TestHostRequest,
};

pub use request_execution::with_bootstrap_advancement;

pub use request_execution::WorthQueryBootstrapAdvancementPhase;

pub(in crate::domain_computation::primary_graph) use request_execution::advancement::observation::application::record_commit as record_advancement_commit;
