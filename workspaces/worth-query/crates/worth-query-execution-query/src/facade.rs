//! Crate facade for primary-graph query evaluation, managed indexes, derived
//! views, handlers, invariants, resources, bootstrap, installation, conditional
//! operations, causal inspection, correction sources and
//! producer computation.

pub mod domain_computation {
    pub use crate::domain_computation::*;
}

pub mod ports {
    pub use crate::domain_computation::execution_runtime::product_world::internal_port::QueryProductRuntimeInternalPort;
    pub use crate::domain_computation::execution_runtime::QueryExecutionRuntimeInternalPort;
    pub use crate::domain_computation::primary_graph::application_contribution::conditional::producer_access::ConditionalProducerAccessInternalPort;
    pub use crate::domain_computation::primary_graph::application_contribution::derived_relation::DerivedRelationCountOwnerInternalPort;
    pub use crate::domain_computation::primary_graph::application_contribution::partitioned_computation::installed::collection::InstalledPartitionedComputationInternalPort;
    pub use crate::domain_computation::primary_graph::application_contribution::producer::demand::internal_port::AdmittedOutputDemandInternalPort;
    pub use crate::domain_computation::primary_graph::application_output_demand::completed_outputs::CompletedRequiredOutputsInternalPort;
    pub use crate::domain_computation::primary_graph::application_output_demand::original_outputs::CompletedOriginalReadOutputsInternalPort;
    pub use crate::domain_computation::primary_graph::application_query::batch::planning::BatchAdmissionInternalPort;
    pub use crate::domain_computation::primary_graph::application_query::controls::internal_port::AdmittedApplicationQueryControlsInternalPort;
    pub use crate::domain_computation::primary_graph::application_resource::closed::ClosedResourceInternalPort;
    pub use crate::domain_computation::primary_graph::application_resource::stop::{
        ResourceActionDenialInternalPort, ResourceRecoveryCloseStopInternalPort,
    };
    pub use crate::domain_computation::primary_graph::application_runtime::runtime_core::internal_port::ApplicationRuntimeCoreInternalPort;
    pub use crate::domain_computation::primary_graph::causal_inspection::application_observation::declared_query::ApplicationQueryObservationInternalPort;
    pub use crate::domain_computation::primary_graph::conditional_operation::installation::clock_handle::ConditionalClockHandleInternalPort;
    pub use crate::domain_computation::primary_graph::conditional_operation::installation::denial::ConditionalRuntimeInstallationDenialInternalPort;
    pub use crate::domain_computation::primary_graph::product_operation::entry_internal_port::ProductEntryInternalPort;
    pub use crate::domain_computation::primary_graph::product_operation::query_controls::ProductQueryControlsInternalPort;
    pub use crate::ports::*;
}

pub mod runtime {
    pub use crate::domain_computation::execution_runtime::product_world::{
        WorthQueryPerformedRelationalProductChange,
        WorthQueryPerformedRelationalProductChangeDeliveryDenial,
        WorthQueryPerformedRelationalProductChangeDeliveryDenialKind,
        WorthQueryPerformedRelationalProductChangeDeliveryOutcome,
        WorthQueryProductBranchCreationDenial, WorthQueryProductWorldClock,
        WorthQueryProductWorldResources,
    };
    pub use crate::domain_computation::execution_runtime::*;
    pub use crate::domain_computation::{
        WorthQueryAdmittedDirectRun, WorthQueryAdmittedWorkflowRun,
        WorthQueryDirectGraphStepOutcome, WorthQueryDirectRunCleanupFailure,
        WorthQueryDirectRunCleanupReceipt, WorthQueryDirectRunTerminal,
        WorthQueryExecutionBoundOperationAuthority, WorthQueryExecutionOperationBindingDenial,
        WorthQueryInstalledDomainExecutionAuthority, WorthQueryManagedDirectRunAdmissionFailure,
        WorthQueryManagedGraphCallRequest, WorthQueryManagedRunAdmission,
        WorthQueryManagedTruthReadRequest, WorthQueryManagedWorkflowRunAdmissionFailure,
        WorthQueryRunningDirectRun, WorthQueryRunningWorkflowRun,
        WorthQueryWorkflowGraphStepOutcome, WorthQueryWorkflowRunCleanupOutcome,
        WorthQueryWorkflowRunCleanupReceipt, WorthQueryWorkflowRunTerminal,
    };
    pub use worth_execution::{
        build_ordered_tree, build_sorted_ordered_tree, insert_ordered_tree, remove_ordered_tree,
        CancellationSource, CancellationToken, ConstructionDenial, ExecutionAllocationDenial,
        ExecutionAllocationDenialClass, ExecutionAllocationPolicy, ExecutionAuthority,
        ExecutionAuthorityConfig, ExecutionMemoryReservation, ExecutionOrderedMapDenial,
        ExecutionOrderedTreeBounds, ExecutionOrderedTreeNode, ExecutionOrderedTreeStorage,
        ExecutionPhysicalAllocationDenialKind, ExecutionRequestArrayDenial, ExecutionResourceLease,
        LeaseDenial, LeaseRequest, MapKernelStop, MemoryLimitDenial, MemoryLimitLevel,
        WorkCeilingDenial,
    };
    pub use worth_foundational::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
    };
    pub use worth_runtime_world::facade::{
        CompositeComponentChangePosture, CompositeSignalPublicationIdentity, NoEffectCause,
        ProductUnpublishedCause, RuntimeWorldBranchAdmissionDenial,
        RuntimeWorldBranchBudgetInstallation, RuntimeWorldBudgetDenial,
        RuntimeWorldBudgetInstallation, RuntimeWorldBudgets, RuntimeWorldCancellationSource,
        RuntimeWorldCancellationToken, RuntimeWorldCustodyBudgetInstallation,
        RuntimeWorldHistoryBudgetInstallation, RuntimeWorldObservationBudgetInstallation,
        RuntimeWorldPublicationBudgetInstallation, RuntimeWorldRecoveryBudgetInstallation,
        RuntimeWorldRetentionBudgetInstallation, RuntimeWorldServiceDenial,
    };
    pub use worth_signal::facade::branch::{
        validate_signal_branch_name, ValidatedSignalBranchName,
    };
}

pub mod primary_graph {
    pub use crate::domain_computation::primary_graph::*;
}

pub mod product {
    pub use crate::basis::{
        WorthQueryProductBranch, WorthQueryProductBranchAdmissionDenial,
        WorthQueryProductBranchAdmissionRefusalKind, WorthQueryProductBranchLease,
        WorthQueryProductBranchReadIdentity, WorthQueryProductObservationLease,
    };
    pub use crate::domain_computation::primary_graph::{
        WorthQueryProductEntry, WorthQuerySelectedProductOperation,
    };
    pub use crate::domain_computation::{
        WorthQueryProductStaleApplication, WorthQueryProductUnpublishedApplication,
        WorthQueryProductUnpublishedRecovery,
    };
}

pub mod correction_material {
    pub use crate::domain_computation::application_aftermath::correction_material::{
        QueryCorrectionMaterialDenial, QueryCorrectionMaterialFrame,
        QueryCorrectionMaterialPublicationOutcome, QueryCorrectionsMedium,
        QUERY_CORRECTION_JOURNAL_PROTOCOL, QUERY_CORRECTION_JOURNAL_VERSION,
    };
}

/// Thin forwarding surface for installed execution persistent naming.
pub mod application_naming {
    pub use worth_query_execution_naming::facade::*;
}
