pub mod computation;
pub mod conditional {
    pub mod output_readiness_contract;
    pub mod package_contract;
    pub use output_readiness_contract::{
        WorthQueryOutputReadinessContract, WorthQueryOutputReadinessContractBuilder,
        WorthQueryOutputReadinessContractDenial, WorthQueryReadinessFieldUse,
    };
    pub use package_contract::WorthQueryApplicationConditionalPackageContract;
    pub use worth_query_declaration::facade::application_schema::ApplicationSchema;
    pub use worth_query_installation::facade::{
        WorthQueryConditionalNodeLocation,
        WorthQueryPortableApplicationConditionalOperationBinding,
        WorthQueryPortableDomainOperationDefinition,
    };
}
pub mod execution_denial;
pub mod native_field_contract;
pub mod partitioned_computation;
pub mod producer {
    pub mod demand {
        pub mod bridge_denial;
        pub mod bridge_execution_denial;
        pub mod denial;
        pub mod denial_diagnostic;
        pub mod denial_posture;
        pub mod progression {
            pub mod required_wave {
                pub mod performed {
                    pub mod certification {
                        pub mod current_basis;
                        pub use current_basis::CurrentBasis;
                    }
                    pub use certification::CurrentBasis;
                }
                pub mod resolved {
                    pub mod matched_predecessors;
                    pub use matched_predecessors::*;
                }
                pub use performed::CurrentBasis;
                pub use resolved::*;
            }
            pub use required_wave::*;
        }
        pub mod read_close;
        pub use denial::*;
        pub use progression::{CurrentBasis, MatchedRequiredPredecessors};
        pub use read_close::{WorthQueryOutputReadCleanup, WorthQueryOutputReadCloseDenial};
    }
    pub mod denial_value;
    pub mod input_reuse_contract;
    pub mod observation_reads;
    pub mod output_equivalence_contract;
    pub mod output_evaluation;
    pub mod registry {
        pub mod semantic_edition;
        pub use semantic_edition::*;
    }
    pub mod source_qualification;

    pub use demand::{
        CurrentBasis, MatchedRequiredPredecessors, WorthQueryOutputDemandDenial,
        WorthQueryOutputDemandDenialKind, WorthQueryOutputDemandRecoveryPosture,
        WorthQueryOutputReadCleanup, WorthQueryOutputReadCloseDenial,
    };
    pub use worth_query_declaration::facade::application_program::{
        WorthQueryApplicationOutputDemand, WorthQueryProducerApplicability,
        WorthQueryProducerLifecyclePosture, WorthQueryProducerOutputFamily,
    };

    #[derive(Clone, Eq, PartialEq)]
    pub enum WorthQueryProducerCommitAuthority {
        Ordinary,
        ProgramOutput,
        SelectedProgram {
            identity:
                worth_query_declaration::facade::application_program::ApplicationProgramIdentity,
            revision:
                worth_query_declaration::facade::application_program::ApplicationProgramRevision,
        },
    }

    #[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
    pub struct WorthQueryProducerInvariantRequirement {
        identifier: &'static str,
        major: u16,
        minor: u16,
        execution_point:
            worth_query_declaration::facade::application_schema::ApplicationInvariantExecutionPoint,
    }

    impl WorthQueryProducerInvariantRequirement {
        pub const fn new(
            identifier: &'static str,
            major: u16,
            minor: u16,
            execution_point:
                worth_query_declaration::facade::application_schema::ApplicationInvariantExecutionPoint,
        ) -> Self {
            Self {
                identifier,
                major,
                minor,
                execution_point,
            }
        }

        pub const fn identifier(self) -> &'static str {
            self.identifier
        }

        pub const fn major(self) -> u16 {
            self.major
        }

        pub const fn minor(self) -> u16 {
            self.minor
        }

        pub const fn execution_point(
            self,
        ) -> worth_query_declaration::facade::application_schema::ApplicationInvariantExecutionPoint
        {
            self.execution_point
        }
    }
    pub use denial_value::{
        WorthQueryProducerDenialDeclaration, WorthQueryProducerDenialDecodeDenial,
        WorthQueryProducerDenialEncodingRefusal, WorthQueryProducerDenialProjection,
        WorthQueryProducerDenialValue, WorthQueryProjectedProducerDenial,
        WorthQueryRecordedProducerDenialEncodingRefusal, WorthQueryRecordedProducerDenialValue,
        WorthQueryUndeclaredProducerDenial,
    };
    pub use input_reuse_contract::{
        WorthQueryDecisionContextDependencies, WorthQueryProducerInputReuseContract,
    };
    pub use observation_reads::{
        WorthQueryProducerObservationReadContract, WorthQueryProducerObservationReadDenial,
        WorthQueryProducerObservationReadOperation,
    };
    pub use output_evaluation::{
        ComparedProducerOutput, ProducerOutputEvaluation, ProducerOutputEvaluationState,
    };
    pub use registry::InstalledProducerEdition;
    pub use source_qualification::{OutputSourceAddress, QualifiedOutputSource};
}
pub use worth_query_execution_runtime::facade::request_execution;

pub use computation::{
    WorthQueryCompletedManagedComputation, WorthQueryInstalledManagedComputation,
    WorthQueryManagedComputationCheckpoint, WorthQueryManagedComputationCheckpointDenial,
    WorthQueryManagedComputationDenial, WorthQueryManagedComputationExecution,
    WorthQueryManagedComputationInterruption, WorthQueryManagedComputationOwner,
    WorthQueryManagedComputationResourceDenial, WorthQueryMemberExecution,
    WorthQueryMemoryLimitLevel, WorthQueryPreparedManagedComputation,
};
pub use conditional::{
    WorthQueryOutputReadinessContract, WorthQueryOutputReadinessContractBuilder,
    WorthQueryOutputReadinessContractDenial, WorthQueryReadinessFieldUse,
};
pub use execution_denial::publication_execution_kind;
pub use partitioned_computation::{
    discarded_computation_retention_on_this_thread_for_test,
    full_partitioned_computation_preparations_on_this_thread_for_test, observe_discarded,
    observe_published, partitioned_computation_runs_on_this_thread_for_test,
    partitioned_computation_tree_work_on_this_thread_for_test,
    published_partitioned_computations_on_this_thread_for_test, Comparator,
    CompletedComputationRetention, ComputationDeposit, ComputationPrior, NativeComputationReads,
    PriorAbsence, RetainedComputation, SealedComputationRetention, SealedComputationRun,
    Suppression, WorthQueryComputationInputDenial, WorthQueryComputationPartitionMembers,
    WorthQueryComputationPartitionPlan, WorthQueryComputationPartitionStop,
    WorthQueryComputationPartitionView, WorthQueryComputationReadDenial,
    WorthQueryDeterministicReducer, WorthQueryPartitionedComputationDenial,
    WorthQueryPartitionedComputationFullCause, WorthQueryPartitionedComputationRunForTest,
    WorthQueryPartitionedTreeAccounting, WorthQueryPartitionedTreeMetrics,
    WorthQueryPartitionedTreeRebuildCause, WorthQueryPartitionedTreeRun,
    WorthQueryPublishedComputationStateForTest, WorthQueryReductionInputDenial,
};
pub use producer::{
    ComparedProducerOutput, CurrentBasis, InstalledProducerEdition, MatchedRequiredPredecessors,
    OutputSourceAddress, ProducerOutputEvaluation, ProducerOutputEvaluationState,
    QualifiedOutputSource, WorthQueryApplicationOutputDemand,
    WorthQueryDecisionContextDependencies, WorthQueryOutputDemandDenial,
    WorthQueryOutputDemandDenialKind, WorthQueryOutputDemandRecoveryPosture,
    WorthQueryOutputReadCleanup, WorthQueryOutputReadCloseDenial, WorthQueryProducerApplicability,
    WorthQueryProducerCommitAuthority, WorthQueryProducerDenialDeclaration,
    WorthQueryProducerDenialDecodeDenial, WorthQueryProducerDenialEncodingRefusal,
    WorthQueryProducerDenialProjection, WorthQueryProducerDenialValue,
    WorthQueryProducerInputReuseContract, WorthQueryProducerInvariantRequirement,
    WorthQueryProducerLifecyclePosture, WorthQueryProducerObservationReadContract,
    WorthQueryProducerObservationReadDenial, WorthQueryProducerObservationReadOperation,
    WorthQueryProducerOutputFamily, WorthQueryProjectedProducerDenial,
    WorthQueryRecordedProducerDenialEncodingRefusal, WorthQueryRecordedProducerDenialValue,
    WorthQueryUndeclaredProducerDenial,
};
pub use request_execution::{
    advancement_denial, advancement_observations_on_this_thread_for_test,
    advancement_requests_on_this_thread_for_test,
    bound_advancement_requests_on_this_thread_for_test,
    place_managed_computations_on_this_thread_for_test, query_test_execution_authority_for_test,
    test_execution_workers_for_test, with_bootstrap_advancement,
    with_bootstrap_advancement_from_request, QueryMemoryReservation, QueryRequestExecution,
    WorthQueryAdvancementDenial, WorthQueryAdvancementEventForTest,
    WorthQueryAdvancementEventKindForTest, WorthQueryAdvancementIdentityForTest,
    WorthQueryAdvancementObservationForTest, WorthQueryAdvancementPhase,
    WorthQueryBootstrapAdvancementPhase, WorthQueryExecutionPlacementForTest,
    WorthQueryForeignAdvancementPhase,
};
