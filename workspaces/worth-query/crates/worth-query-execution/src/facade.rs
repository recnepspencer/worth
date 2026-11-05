//! Public contract for the internal execution authority.

pub use crate::{__worth_query_complete_feature, worth_query_application_contribution};
#[doc(hidden)]
pub use worth_query_declaration::facade as declaration;
#[doc(hidden)]
pub use worth_query_declaration::worth_query_application_contribution as __worth_query_schema_contribution;
#[doc(hidden)]
pub use worth_query_declaration::worth_query_feature_spec;

/// Canonical diagnostic evidence protocol shared by the internal Query engine.
/// These identities describe evidence; they grant no execution or inspection authority.
#[doc(hidden)]
pub mod evidence_identity {
    pub use crate::domain_computation::evidence_identity::*;
}

/// Operational causal inspection owner; application entry still requires native receipts.
#[doc(hidden)]
pub mod causal_inspection {
    pub use crate::domain_computation::primary_graph::causal_inspection::*;
}

/// Fresh application inspection; only its supported result and request vocabulary.
pub mod application_inspection {
    pub use {crate::domain_computation::primary_graph::causal_inspection::CausalInspectionReason, crate::domain_computation::primary_graph::causal_inspection::CausalObservationMissingReferencePosture, crate::domain_computation::primary_graph::causal_inspection::DiagnosticComparisonDenial, crate::domain_computation::primary_graph::causal_inspection::FactlessCurrentness, crate::domain_computation::primary_graph::vocabulary::application_attempt::indexed_entity_selection::IndexedSelectionReobserveDenial, crate::domain_computation::primary_graph::causal_inspection::OutputFieldCaptureFailure, crate::domain_computation::primary_graph::causal_inspection::OwnEffectOnReads, crate::domain_computation::primary_graph::vocabulary::provider::diagnostic::RebaseVerificationReason, crate::domain_computation::primary_graph::causal_inspection::StoreDenial, crate::domain_computation::primary_graph::causal_inspection::WorthQueryApplicationDisclosureContractDenial, crate::domain_computation::primary_graph::causal_inspection::WorthQueryApplicationEvidenceFamily, crate::domain_computation::primary_graph::causal_inspection::WorthQueryApplicationInspectionCanonicalRefusal, crate::domain_computation::primary_graph::causal_inspection::WorthQueryApplicationInspectionDenial, crate::domain_computation::primary_graph::causal_inspection::WorthQueryApplicationInspectionOwnerRefusal, crate::domain_computation::primary_graph::causal_inspection::WorthQueryApplicationInspectionRequest, crate::domain_computation::primary_graph::causal_inspection::WorthQueryApplicationInspectionResourceRefusal, crate::domain_computation::primary_graph::causal_inspection::WorthQueryApplicationInspectionStage, crate::domain_computation::primary_graph::causal_inspection::WorthQueryApplicationInspectionStopKind, crate::domain_computation::primary_graph::causal_inspection::WorthQueryApplicationMaterializationPolicy, crate::domain_computation::primary_graph::causal_inspection::WorthQueryApplicationMaterializationRequest, crate::domain_computation::primary_graph::causal_inspection::WorthQueryApplicationProducerConnection, crate::domain_computation::primary_graph::causal_inspection::WorthQueryApplicationProducerEvaluation, crate::domain_computation::primary_graph::causal_inspection::WorthQueryApplicationQueryGovernanceDenialKind, crate::domain_computation::primary_graph::causal_inspection::WorthQueryApplicationQueryInstallationDenial, crate::domain_computation::primary_graph::causal_inspection::WorthQueryApplicationQueryInstallationDenialKind, crate::domain_computation::primary_graph::causal_inspection::WorthQueryApplicationQueryObservation, crate::domain_computation::primary_graph::causal_inspection::WorthQueryApplicationRichMaterialization, crate::domain_computation::primary_graph::causal_inspection::WorthQueryChangedInspectionRequest, crate::domain_computation::primary_graph::causal_inspection::WorthQueryChangedReferenceOnlyInspectionRequest, crate::domain_computation::primary_graph::causal_inspection::WorthQueryComparedApplicationInspection, crate::domain_computation::primary_graph::causal_inspection::WorthQueryConditionalHistoryAvailability, crate::domain_computation::primary_graph::causal_inspection::WorthQueryConditionalHistoryDecisionClass, crate::domain_computation::primary_graph::causal_inspection::WorthQueryConditionalHistoryOutcome, crate::domain_computation::primary_graph::causal_inspection::WorthQueryConnectedApplicationInspection, crate::domain_computation::primary_graph::causal_inspection::WorthQueryConnectedInspectionRequest, crate::domain_computation::primary_graph::causal_inspection::WorthQueryConnectedReferenceOnlyInspectionRequest, crate::domain_computation::primary_graph::causal_inspection::WorthQueryEvaluatedApplicationInspection, crate::domain_computation::primary_graph::causal_inspection::WorthQueryEvaluatedInspectionRequest, crate::domain_computation::primary_graph::causal_inspection::WorthQueryEvaluatedReferenceOnlyInspectionRequest, crate::domain_computation::primary_graph::causal_inspection::WorthQueryInspectionEvidenceAvailability, crate::domain_computation::primary_graph::causal_inspection::WorthQueryInspectionReadTerminalDenial, crate::domain_computation::primary_graph::causal_inspection::WorthQueryInspectionRetainedEvidence, crate::domain_computation::primary_graph::causal_inspection::WorthQueryInspectionSupplierEvidence, crate::domain_computation::primary_graph::causal_inspection::WorthQueryManagedGraphReadDenial, crate::domain_computation::primary_graph::causal_inspection::WorthQueryManagedGraphWorkSessionStartDenial, crate::domain_computation::primary_graph::causal_inspection::WorthQueryMaterializedApplicationInspection, crate::domain_computation::primary_graph::causal_inspection::WorthQueryMaterializedConditionalHistory, crate::domain_computation::primary_graph::causal_inspection::WorthQueryMaterializedConnectedInspectionRequest, crate::domain_computation::primary_graph::causal_inspection::WorthQueryMaterializedEvidenceReference, crate::domain_computation::primary_graph::causal_inspection::WorthQueryMaterializedFamilyOutcome, crate::domain_computation::primary_graph::causal_inspection::WorthQueryMaterializedFieldValue, crate::domain_computation::primary_graph::causal_inspection::WorthQueryMaterializedInspectionRequest, crate::domain_computation::primary_graph::causal_inspection::WorthQueryMaterializedPerformedSourceInspection, crate::domain_computation::primary_graph::causal_inspection::WorthQueryMaterializedQueryField, crate::domain_computation::primary_graph::causal_inspection::WorthQueryMaterializedQueryProjection, crate::domain_computation::primary_graph::causal_inspection::WorthQueryMaterializedQueryRelation, crate::domain_computation::primary_graph::causal_inspection::WorthQueryMaterializedQueryRow, crate::domain_computation::primary_graph::causal_inspection::WorthQueryMaterializedSourceEffects, crate::domain_computation::primary_graph::causal_inspection::WorthQueryMaterializedSourceField, crate::domain_computation::primary_graph::causal_inspection::WorthQueryMaterializedSourceFieldEffect, crate::domain_computation::primary_graph::causal_inspection::WorthQueryMaterializedSourceInspectionRequest, crate::domain_computation::primary_graph::causal_inspection::WorthQueryObservedApplicationInspection, crate::domain_computation::primary_graph::causal_inspection::WorthQueryObservedInspectionRequest, crate::domain_computation::primary_graph::causal_inspection::WorthQueryPerformedSourceInspection, crate::domain_computation::primary_graph::causal_inspection::WorthQueryPerformedSourceInspectionRequest, crate::domain_computation::primary_graph::causal_inspection::WorthQueryProducerConnectionDenial, crate::domain_computation::primary_graph::causal_inspection::WorthQueryProducerEvaluationDecision, crate::domain_computation::primary_graph::causal_inspection::WorthQueryProducerEvaluationDenial, crate::domain_computation::primary_graph::causal_inspection::WorthQueryProducerEvaluationIneligibility, crate::domain_computation::primary_graph::causal_inspection::WorthQueryProducerOutputConnectionDenial, crate::domain_computation::primary_graph::causal_inspection::WorthQueryProjectionMaterializationRefusal, crate::domain_computation::primary_graph::causal_inspection::WorthQueryReferenceOnlyInspectionRequest, crate::domain_computation::primary_graph::causal_inspection::WorthQuerySourceInspectionRequest, crate::domain_computation::primary_graph::causal_inspection::WorthQuerySourceReferenceOnlyInspectionRequest};
}

/// Directional integration for retained legacy request occurrences.
#[doc(hidden)]
pub mod resource_custody {
    pub use crate::domain_computation::primary_graph::application_resource::legacy_bridge::{
        WorthQueryInstalledOwnedAsyncSource, WorthQueryOwnedAsyncRequestCustody,
        WorthQueryOwnedAsyncRuntimeCause, WorthQueryOwnedAsyncRuntimeDenial,
        WorthQueryOwnedAsyncRuntimeServices, WorthQueryOwnedAsyncRuntimeStop,
        WorthQueryOwnedAsyncRuntimeTopology,
    };
}

pub mod domain_computation {
    pub use super::primary_graph::*;
    pub use crate::domain_computation::artifact_owner::*;
    pub use crate::domain_computation::convergence_epoch::*;
    pub use crate::domain_computation::execution_runtime::*;
    pub use crate::domain_computation::managed_run::*;
    pub use crate::domain_computation::operation_binding::*;
    pub use crate::domain_computation::provider_session::*;
    pub use crate::domain_computation::vocabulary::execution_runtime::runtime_identity::WorthQueryRuntimeAuthorityIdentity;
    pub use crate::domain_computation::vocabulary::managed_run::counters::WorthQueryManagedRunCounters;
    pub use crate::domain_computation::vocabulary::managed_run::evidence::WorthQueryManagedProviderSessionDisposition;
    pub use crate::domain_computation::vocabulary::managed_run::evidence::WorthQueryManagedProviderWorkEvidence;
    pub use crate::domain_computation::vocabulary::managed_run::inspection::WorthQueryDirectRunCleanupInspection;
    pub use crate::domain_computation::vocabulary::managed_run::inspection::WorthQueryDirectRunCleanupReceipt;
    pub use crate::domain_computation::vocabulary::managed_run::safe_point_observation::WorthQueryManagedSafePointObservation;
    pub use crate::domain_computation::vocabulary::managed_run::terminal::WorthQueryManagedRunCleanupDisposition;
    pub use crate::domain_computation::vocabulary::managed_run::terminal::WorthQueryManagedRunCleanupFailureKind;
    pub use crate::domain_computation::vocabulary::managed_run::terminal::WorthQueryManagedRunTerminalKind;
    pub use crate::domain_computation::vocabulary::operation_binding::installed_domain_authority::WorthQueryInstalledDomainExecutionAuthority;
    pub use crate::domain_computation::vocabulary::primary_graph::control::WorthQueryManagedIndexControl;
    pub use crate::domain_computation::vocabulary::primary_graph::invocation::WorthQueryManagedIndexWorkDenial;
    pub use crate::domain_computation::vocabulary::primary_graph::stop::WorthQueryManagedIndexStop;
    pub use crate::domain_computation::vocabulary::provider_session::call_kind::WorthQueryGraphProviderCallKind;
    pub use crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind;
    pub use crate::domain_computation::vocabulary::provider_session::denial::WorthQueryGraphProviderStepDenial;
    pub use crate::domain_computation::vocabulary::provider_session::denial::WorthQueryGraphProviderStepDenialKind;
    pub use crate::domain_computation::vocabulary::provider_session::direct_attempt::WorthQueryDirectExecutionAttemptReleaseReceipt;
    pub use crate::domain_computation::vocabulary::provider_session::failure::WorthQueryGraphProviderFailure;
    pub use crate::domain_computation::vocabulary::provider_session::protocol_denial::WorthQueryProviderSessionDenialKind;
    pub use crate::domain_computation::vocabulary::provider_session::provider_execution_release::WorthQueryProviderExecutionDestructorDisposition;
    pub use crate::domain_computation::vocabulary::provider_session::provider_execution_release::WorthQueryProviderExecutionDisposalDisposition;
    pub use crate::domain_computation::vocabulary::provider_session::session_identity::WorthQueryGraphWorkManagedRunIdentity;
    pub use crate::domain_computation::vocabulary::provider_session::session_identity::WorthQueryGraphWorkSessionIdentity;
    pub use crate::domain_computation::vocabulary::provider_session::step_failure::WorthQueryGraphProviderStepInvocationDisposition;
    pub use crate::domain_computation::{
        canonical_indexed_operation_material, canonical_operation_material,
        WorthQueryConvergenceDomainEvidenceBindingDenial,
    };
}

/// The execution runtime: product world and branch setup, performed-change
/// delivery, direct and workflow runs with their cleanup, and Runtime World
/// budgets and cancellation.
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
    pub use crate::domain_computation::vocabulary::execution_runtime::runtime_identity::WorthQueryRuntimeAuthorityIdentity;
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
        ProductUnpublishedCause, RuntimeWorldBranchBudgetInstallation, RuntimeWorldBudgetDenial,
        RuntimeWorldBudgetInstallation, RuntimeWorldBudgets, RuntimeWorldCancellationSource,
        RuntimeWorldCancellationToken, RuntimeWorldCustodyBudgetInstallation,
        RuntimeWorldHistoryBudgetInstallation, RuntimeWorldObservationBudgetInstallation,
        RuntimeWorldPublicationBudgetInstallation, RuntimeWorldRecoveryBudgetInstallation,
        RuntimeWorldRetentionBudgetInstallation,
    };
    pub use worth_signal::facade::branch::{
        validate_signal_branch_name, ValidatedSignalBranchName,
    };
    pub use {
        crate::domain_computation::vocabulary::managed_run::inspection::WorthQueryDirectRunCleanupReceipt,
        crate::domain_computation::vocabulary::operation_binding::installed_domain_authority::WorthQueryInstalledDomainExecutionAuthority,
        crate::domain_computation::WorthQueryAdmittedDirectRun,
        crate::domain_computation::WorthQueryAdmittedWorkflowRun,
        crate::domain_computation::WorthQueryDirectGraphStepOutcome,
        crate::domain_computation::WorthQueryDirectRunCleanupFailure,
        crate::domain_computation::WorthQueryDirectRunTerminal,
        crate::domain_computation::WorthQueryExecutionBoundOperationAuthority,
        crate::domain_computation::WorthQueryExecutionOperationBindingDenial,
        crate::domain_computation::WorthQueryManagedDirectRunAdmissionFailure,
        crate::domain_computation::WorthQueryManagedGraphCallRequest,
        crate::domain_computation::WorthQueryManagedRunAdmission,
        crate::domain_computation::WorthQueryManagedTruthReadRequest,
        crate::domain_computation::WorthQueryManagedWorkflowRunAdmissionFailure,
        crate::domain_computation::WorthQueryRunningDirectRun,
        crate::domain_computation::WorthQueryRunningWorkflowRun,
        crate::domain_computation::WorthQueryWorkflowGraphStepOutcome,
        crate::domain_computation::WorthQueryWorkflowRunCleanupOutcome,
        crate::domain_computation::WorthQueryWorkflowRunCleanupReceipt,
        crate::domain_computation::WorthQueryWorkflowRunTerminal,
    };
}

/// Read-only owner observations for cert-band consumers.
pub mod certification {
    pub use {crate::domain_computation::primary_graph::vocabulary::application_runtime::certification_cost::WorthQueryCertificationApplicationWork, crate::domain_computation::primary_graph::vocabulary::application_runtime::certification_cost::WorthQueryCertificationCostObservation, crate::domain_computation::primary_graph::WorthQueryCertificationCostRuntimeExt, crate::domain_computation::primary_graph::vocabulary::application_runtime::certification_cost::WorthQueryCertificationCostScope, crate::domain_computation::primary_graph::vocabulary::application_runtime::certification_cost::WorthQueryCertificationWorldHistory, crate::domain_computation::primary_graph::vocabulary::application_runtime::certification_cost::WorthQueryCertificationWorldRetention};
}

pub mod application_contribution;
pub mod application_discovery;
pub mod application_installation;
pub mod application_invariants;
pub mod product;

pub mod provider_session {
    pub use crate::domain_computation::provider_session::*;
    pub use crate::domain_computation::vocabulary::provider_session::call_kind::WorthQueryGraphProviderCallKind;
    pub use crate::domain_computation::vocabulary::provider_session::control_stopped::WorthQueryProviderSessionControlStopKind;
    pub use crate::domain_computation::vocabulary::provider_session::denial::WorthQueryGraphProviderStepDenial;
    pub use crate::domain_computation::vocabulary::provider_session::denial::WorthQueryGraphProviderStepDenialKind;
    pub use crate::domain_computation::vocabulary::provider_session::direct_attempt::WorthQueryDirectExecutionAttemptReleaseReceipt;
    pub use crate::domain_computation::vocabulary::provider_session::failure::WorthQueryGraphProviderFailure;
    pub use crate::domain_computation::vocabulary::provider_session::protocol_denial::WorthQueryProviderSessionDenialKind;
    pub use crate::domain_computation::vocabulary::provider_session::session_identity::WorthQueryGraphWorkManagedRunIdentity;
    pub use crate::domain_computation::vocabulary::provider_session::session_identity::WorthQueryGraphWorkSessionIdentity;
}

pub mod primary_graph;

/// Convergence epochs: bounded iteration of a run toward a stable result, with
/// domain providers that compare, measure progress, and detect repeated
/// states, plus terminal reports.
pub mod convergence_epoch {
    pub use crate::domain_computation::convergence_epoch::*;
}

/// The installed-runtime surface in one place: `runtime`, `domain_computation`,
/// `provider_session`, and `convergence_epoch`.
pub mod installed {
    pub use super::{convergence_epoch, domain_computation, provider_session, runtime};
}

#[doc(hidden)]
pub mod integration {
    pub use crate::domain_computation::execution_runtime::product_world::{
        WorthQueryProductRelationalInstallation, WorthQueryProductRuntime,
        WorthQueryProductRuntimeInstallationDenial, WorthQueryProductSharedRoot,
        WorthQueryProductWorldClock, WorthQueryProductWorldResources,
        WorthQueryRelationalSourceInstallationDenial, WorthQueryRelationalSourceOwner,
    };
    pub use crate::domain_computation::execution_runtime::WorthQueryInvalidationResources;
    pub use worth_execution::{ExecutionAuthority, ExecutionAuthorityConfig};
    pub use worth_foundational::{
        DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
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
    use {
        crate::domain_computation::primary_graph::vocabulary::bootstrap::publication::WorthQueryPrimaryGraphPublication,
        crate::domain_computation::primary_graph::WorthQueryPrimaryGraphBootstrap,
        crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenial,
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
        WorthQueryMergeUniqueValueDenial,
        WorthQueryMergeUniqueValueDenialKind,
        WorthQueryApplicationInvariantFactories,
        WorthQueryApplicationInvariantSchemaResolver,
    };

    pub fn prepare_primary_graph_with_relational_runtime<Schema>(
        phase: &crate::domain_computation::request_execution::WorthQueryBootstrapAdvancementPhase<
            '_,
        >,
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
            phase,
            runtime,
            installed_schema,
            relational_runtime,
            product_world_resources,
        )
    }

    pub fn prepare_primary_graph_with_relational_runtime_and_invariants<Schema>(
        phase: &crate::domain_computation::request_execution::WorthQueryBootstrapAdvancementPhase<
            '_,
        >,
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
            phase,
            runtime,
            installed_schema,
            relational_runtime,
            product_world_resources,
            invariant_factories,
        )
    }
    #[doc(hidden)]
    pub fn product_world_resources_for_test(
        retained_composite_commits: u64,
        clock: worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock,
    ) -> WorthQueryProductWorldResources {
        crate::domain_computation::execution_runtime::product_world::test_product_world_resources_with_history_limit_for_test(
            retained_composite_commits,
            clock,
        )
    }
    #[doc(hidden)]
    pub fn prepare_primary_graph_with_active_snapshot_limit_for_test<Schema>(
        phase: &crate::domain_computation::request_execution::WorthQueryBootstrapAdvancementPhase<
            '_,
        >,
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
                max_patch_records_per_commit: None,
                max_published_snapshot_handles: Some(256),
                max_active_snapshot_handles: Some(maximum_active_snapshots),
                max_transaction_savepoints: 4_096,
                max_prepared_candidates: Some(1_024),
                candidate_max_lifetime_millis: 30_000,
            })
            .build()?;
        prepare_primary_graph_with_relational_runtime(
            phase,
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
        phase: &crate::domain_computation::request_execution::WorthQueryBootstrapAdvancementPhase<
            '_,
        >,
        bootstrap: WorthQueryPrimaryGraphBootstrap<Schema>,
        runtime: &mut WorthQueryExecutionRuntime,
        authority: &WorthQueryExecutionInstallationAuthority,
    ) -> Result<WorthQueryPrimaryGraphPublication, WorthQueryPrimaryGraphInstallationDenial>
    where
        Schema: ApplicationSchema,
    {
        bootstrap.publish(phase, runtime, authority)
    }

    #[doc(hidden)]
    pub fn classify_conditional_signal_for_certification(
        evidence: &worth_runtime_bridge::facade::BridgeConditionalDecisionEvidence,
    ) -> crate::domain_computation::primary_graph::vocabulary::conditional_operation::classification::WorthQueryConditionalSignalDecision{
        crate::domain_computation::primary_graph::classify_bridge_signal(evidence)
    }
}

pub use crate::domain_computation::primary_graph::application_query::WorthQueryDerivedPairReadPlans;
pub use worth_execution::ExecutionRequest;
