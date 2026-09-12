//! Public contract for the internal execution authority.

pub mod domain_computation {
    pub use super::primary_graph::*;
    pub use crate::domain_computation::artifact_owner::*;
    pub use crate::domain_computation::convergence_epoch::*;
    pub use crate::domain_computation::execution_runtime::*;
    pub use crate::domain_computation::managed_run::*;
    pub use crate::domain_computation::operation_binding::*;
    pub use crate::domain_computation::provider_session::*;
    pub use crate::domain_computation::{
        canonical_indexed_operation_material, canonical_operation_material,
        WorthQueryConvergenceDomainEvidenceBindingDenial,
    };
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
    pub use worth_runtime_world::facade::{
        CompositeComponentChangePosture, CompositeSignalPublicationIdentity,
        RuntimeWorldBranchBudgetInstallation, RuntimeWorldBudgetDenial,
        RuntimeWorldBudgetInstallation, RuntimeWorldBudgets, RuntimeWorldCancellationSource,
        RuntimeWorldCancellationToken, RuntimeWorldCustodyBudgetInstallation,
        RuntimeWorldHistoryBudgetInstallation, RuntimeWorldObservationBudgetInstallation,
        RuntimeWorldPublicationBudgetInstallation, RuntimeWorldRecoveryBudgetInstallation,
        RuntimeWorldRetentionBudgetInstallation,
    };
    pub use worth_signal::facade::branch::{
        validate_signal_branch_name, ValidatedSignalBranchName,
    };
}

pub mod application_contribution;
pub mod application_discovery;
pub mod application_installation;
pub mod application_invariants;
pub mod product;

pub mod provider_session {
    pub use crate::domain_computation::provider_session::*;
}

pub mod primary_graph;

/// Compatibility surface for the current undo/redo experiment.
///
/// These types remain compiled but are not accepted Phase 8 product contracts.
pub mod provisional_aftermath {
    pub use crate::domain_computation::application_aftermath::{
        admit_undo, consume_redo_progression, consume_unresolved_undo_progression,
        deny_irreversible_undo_attempt, map_ordinary_commit_conflict_to_redo,
        progress_admitted_reconciliation, progress_admitted_redo, progress_admitted_undo,
        WorthQueryAftermathCausalRole, WorthQueryCommittedAftermathCausality, WorthQueryProvedUndo,
        WorthQueryRedoAdmission, WorthQueryRedoDenial, WorthQueryRedoDenialKind,
        WorthQueryRedoIntent, WorthQueryRedoIntentIdentity, WorthQueryRedoProgressionHandoff,
        WorthQueryRedoRecovery, WorthQueryRetainedPreImage, WorthQueryUndoAdmission,
        WorthQueryUndoDenial, WorthQueryUndoDenialKind, WorthQueryUndoDerivedRequest,
        WorthQueryUndoIntentIdentity, WorthQueryUndoProgressionHandoff,
    };
}

pub mod convergence_epoch {
    pub use crate::domain_computation::convergence_epoch::*;
}

pub mod installed {
    pub use super::{convergence_epoch, domain_computation, provider_session, runtime};
}

#[doc(hidden)]
pub mod integration {
    pub use crate::domain_computation::execution_runtime::product_world::{
        WorthQueryProductRelationalInstallation, WorthQueryProductRuntime,
        WorthQueryProductRuntimeInstallationDenial, WorthQueryProductSharedRoot,
        WorthQueryProductWorldClock, WorthQueryProductWorldResources,
        WorthQueryRelationalSourceOwner,
    };
    use worth_query_installation::facade::{
        ApplicationSchema, WorthQueryInstalledApplicationSchema,
    };
    use worth_relational::facade::runtime::RelationalRuntime;
    pub use worth_runtime_world::facade::{
        RuntimeWorldBranchBudgetInstallation, RuntimeWorldBudgetDenial,
        RuntimeWorldBudgetInstallation, RuntimeWorldBudgets, RuntimeWorldCustodyBudgetInstallation,
        RuntimeWorldHistoryBudgetInstallation, RuntimeWorldObservationBudgetInstallation,
        RuntimeWorldOwnedAsyncRequestAdmissionDenial, RuntimeWorldOwnedAsyncRevalidationDenial,
        RuntimeWorldPublicationBudgetInstallation, RuntimeWorldRecoveryBudgetInstallation,
        RuntimeWorldRetentionBudgetInstallation,
    };

    use crate::domain_computation::execution_runtime::{
        WorthQueryExecutionInstallationAuthority, WorthQueryExecutionRuntime,
    };
    use crate::domain_computation::primary_graph::{
        WorthQueryPrimaryGraphBootstrap, WorthQueryPrimaryGraphInstallationDenial,
        WorthQueryPrimaryGraphPublication,
    };

    pub use crate::domain_computation::artifact_owner::{
        WorthQueryArtifactAccessAuthority, WorthQueryArtifactProductionAuthority,
        WorthQueryArtifactTransferAdmission, WorthQueryWorkflowArtifactAuthority,
        WorthQueryWorkflowArtifactRegistry,
    };
    pub use crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor;
    pub use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphIntegrationHandle;
    pub use crate::domain_computation::primary_graph::{
        WorthQueryPrimaryGraphIndexRefreshDenial,
        WorthQueryPrimaryGraphIndexRefreshDenialKind,
        WorthQueryApplicationInvariantFactories,
        WorthQueryApplicationInvariantSchemaResolver,
    };

    pub fn prepare_primary_graph_with_relational_runtime<Schema>(
        authority: &WorthQueryExecutionInstallationAuthority,
        runtime: &WorthQueryExecutionRuntime,
        installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
        relational_runtime: RelationalRuntime,
        product_world_resources: WorthQueryProductWorldResources,
    ) -> Result<WorthQueryPrimaryGraphBootstrap<Schema>, WorthQueryPrimaryGraphInstallationDenial>
    where
        Schema: ApplicationSchema,
    {
        authority.prepare_primary_graph_with_relational_runtime(
            runtime,
            installed_schema,
            relational_runtime,
            product_world_resources,
        )
    }

    pub fn prepare_primary_graph_with_relational_runtime_and_invariants<Schema>(
        authority: &WorthQueryExecutionInstallationAuthority,
        runtime: &WorthQueryExecutionRuntime,
        installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
        relational_runtime: RelationalRuntime,
        product_world_resources: WorthQueryProductWorldResources,
        invariant_factories: WorthQueryApplicationInvariantFactories<Schema>,
    ) -> Result<WorthQueryPrimaryGraphBootstrap<Schema>, WorthQueryPrimaryGraphInstallationDenial>
    where
        Schema: ApplicationSchema,
    {
        authority.prepare_primary_graph_with_relational_runtime_and_invariants(
            runtime,
            installed_schema,
            relational_runtime,
            product_world_resources,
            invariant_factories,
        )
    }
    #[doc(hidden)]
    #[cfg(feature = "test-primary-graph-faults")]
    pub fn product_world_resources_for_test(
        retained_composite_commits: u64,
    ) -> WorthQueryProductWorldResources {
        crate::domain_computation::execution_runtime::product_world::test_product_world_resources_with_history_limit(
            retained_composite_commits,
        )
    }
    #[doc(hidden)]
    #[cfg(feature = "test-primary-graph-faults")]
    pub fn prepare_primary_graph_with_active_snapshot_limit_for_test<Schema>(
        authority: &WorthQueryExecutionInstallationAuthority,
        runtime: &WorthQueryExecutionRuntime,
        installed_schema: &WorthQueryInstalledApplicationSchema<Schema>,
        maximum_active_snapshots: usize,
        product_world_resources: WorthQueryProductWorldResources,
    ) -> Result<WorthQueryPrimaryGraphBootstrap<Schema>, WorthQueryPrimaryGraphInstallationDenial>
    where
        Schema: ApplicationSchema,
    {
        let relational = worth_relational::facade::runtime::RelationalRuntimeApi::builder()
            .profile(worth_relational::facade::config::RelationalRuntimeProfile::AiWorkflow)
            .publication(worth_relational::facade::config::PublicationConfig {
                coherent_publication_required: true,
                max_patch_records_per_commit: 4_096,
                max_published_snapshot_handles: 256,
                max_active_snapshot_handles: maximum_active_snapshots,
                max_transaction_overlay_bytes: 268_435_456,
                max_transaction_footprint_loci: 262_144,
                max_transaction_savepoints: 4_096,
                max_prepared_candidates: 1_024,
                candidate_max_lifetime_millis: 30_000,
                max_prepared_root_bytes: 268_435_456,
            })
            .build();
        prepare_primary_graph_with_relational_runtime(
            authority,
            runtime,
            installed_schema,
            relational,
            product_world_resources,
        )
    }

    pub fn retain_primary_graph_integration_handle(
        runtime: &WorthQueryExecutionRuntime,
    ) -> Option<WorthQueryPrimaryGraphIntegrationHandle> {
        runtime.retain_primary_graph_integration_handle()
    }

    /// Derives Product World's bridge from the exact published primary graph.
    ///
    /// The returned bridge carries the graph's source authority and schema
    /// correspondence. It grants no primary-provider authority.
    #[doc(hidden)]
    pub fn prepare_primary_graph_product_bridge<Schema>(
        installed_schema: &worth_query_installation::facade::WorthQueryInstalledApplicationSchema<
            Schema,
        >,
        integration: &WorthQueryPrimaryGraphIntegrationHandle,
    ) -> Result<worth_runtime_bridge::facade::RuntimeBridge, WorthQueryPrimaryGraphInstallationDenial>
    where
        Schema: ApplicationSchema,
    {
        crate::domain_computation::primary_graph::build_primary_graph_product_bridge(
            installed_schema,
            integration,
        )
    }

    pub fn publish_primary_graph<Schema>(
        bootstrap: WorthQueryPrimaryGraphBootstrap<Schema>,
        runtime: &mut WorthQueryExecutionRuntime,
        authority: &WorthQueryExecutionInstallationAuthority,
    ) -> Result<WorthQueryPrimaryGraphPublication, WorthQueryPrimaryGraphInstallationDenial>
    where
        Schema: ApplicationSchema,
    {
        bootstrap.publish(runtime, authority)
    }

    #[doc(hidden)]
    pub fn classify_conditional_signal_for_certification(
        evidence: &worth_runtime_bridge::facade::BridgeConditionalDecisionEvidence,
    ) -> crate::domain_computation::primary_graph::WorthQueryConditionalSignalDecision {
        crate::domain_computation::primary_graph::classify_bridge_signal(evidence)
    }
}
