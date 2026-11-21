pub(crate) use worth_query_execution_lineage::facade::primary_graph::{
    aggregate_identity, aggregate_observation, aggregate_projection,
    component_selection, content, denial, derived_source, entity_key, entity_resolution_denial, fact_verification_observation, fragment_view, freshness,
    home_original_progression, index_currency, index_refresh, initial_schema_denial,
    merge_unique_values, observations, output_lineage, read_observation, resolution_denial, root, root_path_selection,
    source_facts, source_read_equivalence, source_read_verification, structural_walk,
};
pub(in crate::domain_computation) use content::retention_checkpoint;
pub use content::{
    AdmittedContent, ContentReadDenial, ContentWriteDenial, RelationalContentDenial,
    RelationalContentRejection,
};
mod access_strategy;
pub(crate) mod entity_resolution;
pub(crate) mod invariant_projection;
pub(crate) mod output_reuse;
pub(crate) mod source_disclosure;
mod facade;
pub use facade::*;
pub(crate) use worth_query_execution_foundation::facade::primary_graph::{
    admitted_capture, application_branch, exact_basis_access, index_maintenance_budget,
    output_binding_identity, output_family_identity, principal_key, projection_read_origin,
    publication_request_denial, request_allocation_control, request_local, value_decode,
};
pub(crate) mod application_attempt;
pub(crate) mod application_checkpoint;
pub(crate) mod application_contribution;
pub(in crate::domain_computation) use application_contribution::ComputationPrior;
pub(in crate::domain_computation) use application_contribution::SealedComputationRetention;
pub(crate) mod application_discovery;
mod application_entry;
pub(crate) mod application_installation;
pub(crate) mod application_invariant;
mod application_invariant_preparation;
pub(crate) mod application_output_demand;
pub(in crate::domain_computation::primary_graph) use application_output_demand::PreparedPrerequisiteClaims;
pub(in crate::domain_computation) use application_output_demand::RequiredOutputDemandContext;
pub(in crate::domain_computation) use application_output_demand::RequiredOutputExecution;
pub use application_output_demand::WorthQueryCompletedReadOutputs;
pub use application_output_demand::WorthQueryCompletedRequiredOutputs;
pub use application_output_demand::WorthQueryFulfilledRequiredRoot;
pub use application_output_demand::WorthQueryRecoveredProducerPublication;
pub use application_output_demand::{
    WorthQueryCompletedOriginalReadOutputs, WorthQueryOriginalReadOutputDenial,
};
pub use application_output_demand::{WorthQueryOutputSourceEvidence, WorthQueryOutputSourceReplay};
mod application_program;
pub(crate) mod application_query;
pub(crate) mod application_resource;
pub(crate) mod application_runtime;
pub(in crate::domain_computation) use application_runtime::WorthQueryApplicationRuntimeCore;
pub(crate) mod authenticated_principal;
mod bootstrap;
pub(crate) mod bootstrap_publication;
pub(crate) mod conditional_operation;
pub(crate) use conditional_operation::classify_bridge_signal;
pub use conditional_operation::conditional_intent_read_is_active_for_test;
mod expression;
mod granular_invalidation;
pub(crate) mod handler;
#[doc(hidden)]
pub use fragment_view::Readable as FragmentReadable;
pub use fragment_view::{
    CandidateEntity, FragmentDenial, ObservedEntity, ObservedTarget, Reading, View, Writing,
};
pub(in crate::domain_computation) use handler::DecisionContextUse;
mod invariant_installation;
pub(crate) mod live_delivery;
pub(crate) mod managed_bridge;
pub(in crate::domain_computation) use source_facts::{
    validate_source_principal_facts, SourceFactsAllocation,
};
pub(crate) use managed_bridge::build_primary_graph_product_bridge;
mod operation_body;
mod ordinary_read;
pub(in crate::domain_computation) use output_lineage::SourceInvalidationOwner;
mod principal_currentness_capture;
pub(in crate::domain_computation) use principal_currentness_capture::capture_principal_currentness_admitted;
pub(in crate::domain_computation) use principal_currentness_capture::PrincipalCurrentnessCaptureStop;
pub(crate) mod product_activation;
pub(crate) mod product_operation;
pub use application_query::{
    WorthQueryHistoryOutputIntent, WorthQueryHistoryOutputNavigationDenial,
};
pub use product_operation::document_history::{
    WorthQueryAdmittedHistoryProgramOutput, WorthQueryCompletedHistoryOutputs,
    WorthQueryCompletedRecordingOutputs, WorthQueryDocumentHistory,
    WorthQueryDocumentHistoryAdoptionFailure, WorthQueryDocumentHistoryAttachmentFailure,
    WorthQueryDocumentHistoryCandidateDenial, WorthQueryDocumentHistoryCapturePauseForTest,
    WorthQueryDocumentHistoryCloseFailure, WorthQueryDocumentHistoryCloseReceipt,
    WorthQueryDocumentHistoryClosing, WorthQueryDocumentHistoryClosingDenial,
    WorthQueryDocumentHistoryDenial, WorthQueryDocumentHistoryGroupAborting,
    WorthQueryDocumentHistoryGroupAdoptionFailure, WorthQueryDocumentHistoryGroupPreparation,
    WorthQueryDocumentHistoryGroupReservation, WorthQueryDocumentHistoryIdentity,
    WorthQueryDocumentHistoryNavigation, WorthQueryDocumentHistoryOperation,
    WorthQueryDocumentHistoryOperationConstraint, WorthQueryDocumentHistoryRecordPreparation,
    WorthQueryDocumentHistoryReservation, WorthQueryHistoryCommandContextDenial,
    WorthQueryHistoryContextRefreshDenial, WorthQueryHistoryContextRefreshLimits,
    WorthQueryHistoryContextRefreshPauseForTest, WorthQueryHistoryDependentOutputRead,
    WorthQueryHistoryDescriptionContext, WorthQueryHistoryEnrollmentObservationForTest,
    WorthQueryHistoryEnrollmentReportForTest, WorthQueryHistoryGroupCandidateCreation,
    WorthQueryHistoryGroupMemberAdmission, WorthQueryHistoryNavigationCompletion,
    WorthQueryHistoryNavigationDenial, WorthQueryHistoryNavigationEligibilityDenial,
    WorthQueryHistoryNavigationIntent, WorthQueryHistoryNavigationPauseForTest,
    WorthQueryHistoryNavigationPosture, WorthQueryHistoryNodeId,
    WorthQueryHistoryOutputAdvanceDenial, WorthQueryHistoryOutputAuthority,
    WorthQueryHistoryOutputDiscovery, WorthQueryHistoryOutputNavigationAdvance,
    WorthQueryHistoryOutputProfile, WorthQueryHistoryOutputQualificationDenial,
    WorthQueryHistoryOutputSource, WorthQueryHistoryPage,
    WorthQueryHistoryPageAssemblyPauseForTest, WorthQueryHistoryPageCursor,
    WorthQueryHistoryPageDenial, WorthQueryHistoryPageEntry, WorthQueryHistoryPageLimits,
    WorthQueryHistoryPageRow, WorthQueryHistoryParentBoundary, WorthQueryHistoryPreview,
    WorthQueryHistoryProgramOutputAdvance, WorthQueryHistoryRecordingOutputOrigin,
    WorthQueryHistoryRedoChoice, WorthQueryHistoryRedoChoices, WorthQueryHistoryReleaseOutcome,
    WorthQueryHistoryRetentionChangePauseForTest, WorthQueryHistoryRetentionStatus,
    WorthQueryHistoryRootOutputAdmissionDenial, WorthQueryHistorySelection, WorthQueryHistoryStep,
    WorthQueryRecordingOutputQualificationDenial, WorthQueryRetainedHistoryOutputTree,
    WorthQueryRetainedRecordingOutputTree, WorthQuerySettledHistoryProgramOutput,
    WorthQueryVerifiedHistoryParent,
};
pub(in crate::domain_computation) use product_operation::SharedSelectedProductOperation;
pub use product_operation::WorthQueryHistoryOutputControlDenial;
pub use product_operation::WorthQueryHistoryOutputReadDenial;
pub use product_operation::WorthQueryHistoryRootOutputRead;
pub(crate) mod program_occurrence;
pub(crate) mod query_obligations;
pub(in crate::domain_computation) use program_occurrence::OriginalProgramMeaning;
pub(crate) mod provider;
pub(crate) mod retained_correction;
pub(in crate::domain_computation) use application_attempt::retained_decision_facts::StorageControl;
pub(in crate::domain_computation::primary_graph) use provider::OutstandingDispatchInFlightLease;
pub(in crate::domain_computation) use provider::PrimaryDecisionReadSetComparison;
pub(in crate::domain_computation) use provider::QueryPreparingApplicationCommit;
pub(in crate::domain_computation) use provider::RegisteredPrimaryRead;
pub use retained_correction::WorthQueryCorrectionRetentionDenial;
pub(crate) mod resolution;
pub(crate) use worth_query_execution_layout::facade::schema_layout;
mod typed_bootstrap;
pub(crate) mod workflow;
pub use workflow::captured_input::{
    PreparedWorkflowCapturedInput, WorkflowCapturedInputContract, WorkflowCapturedInputDenial,
    WorkflowCapturedInputSource, WorkflowCapturedInputs,
};

pub(in crate::domain_computation) use application_attempt::application_resource_request;
pub(in crate::domain_computation) use application_attempt::precondition_binding::bind_empty_mutation_preconditions_admitted;
pub(in crate::domain_computation) use application_attempt::precondition_binding::bind_mutation_preconditions;
pub(in crate::domain_computation) use application_attempt::precondition_binding::EmptyMutationPreconditionAdmissionStop;
pub(in crate::domain_computation) use application_attempt::precondition_binding::WorthQueryBoundMutationPreconditions;
pub(in crate::domain_computation) use application_attempt::progression_denied;
pub(in crate::domain_computation) use application_attempt::WorthQueryApplicationAttemptAffinity;
pub(in crate::domain_computation) use application_attempt::WorthQueryApplicationAttemptBasis;
pub(in crate::domain_computation) use application_attempt::WorthQueryApplicationObservedFact;
pub(in crate::domain_computation::primary_graph) use application_attempt::WorthQueryApplicationOutputCorrespondence;
pub(in crate::domain_computation) use application_attempt::WorthQueryApplicationSnapshotLease;
pub(in crate::domain_computation) use application_attempt::WorthQueryApplicationSnapshotLeaseDenial;
pub(in crate::domain_computation) use application_attempt::WorthQueryApplicationSnapshotRelease;
pub(crate) use application_attempt::WorthQueryPerformedExternalRedispatchSeal;
pub(in crate::domain_computation) use application_attempt::WorthQueryPreparedApplicationProviderAttempt;
pub(in crate::domain_computation) use application_attempt::WorthQueryProviderAttemptRegistrationContext;
pub(in crate::domain_computation) use application_attempt::WorthQueryProviderProgressionOutcome;
pub(in crate::domain_computation) use application_attempt::WorthQueryRegisteredProviderAttempt;
pub(crate) use application_attempt::WorthQueryRetainedGovernedInput;
pub(in crate::domain_computation) use application_branch::primary_relational_branch_id;
pub(in crate::domain_computation) use application_branch::primary_truth_branch_identity;
pub(in crate::domain_computation) use application_query::observed_source::WorthQuerySourceExpectationKind;
pub(in crate::domain_computation) use exact_basis_access::WorthQueryExactBasisSnapshotDenial;
pub(in crate::domain_computation) use provider::ApplicationFactTable;
pub(in crate::domain_computation) use provider::CompletedApplicationFacts;
pub(in crate::domain_computation) use provider::WorthQueryPrimaryGraphApplicationDecisionFact;
pub(crate) use provider::WorthQueryPrimaryGraphProvider;
pub(crate) use provider::WorthQueryPrimaryLogicalGraph;
pub(in crate::domain_computation) use provider::WorthQueryUnpublishedIdempotencyDisposition;
pub(crate) use provider::{WorthQueryRetainedPreImageSeal, WorthQueryRetainedRelationPreImageSeal};

pub(crate) use application_query::WorthQueryApplicationQueryControls;
pub(in crate::domain_computation) use crate::domain_computation::application_aftermath::external_effect::WorthQueryAdmittedExternalDispatchAttempt;
pub(in crate::domain_computation) use application_runtime::WorthQueryExternalDispatchAttemptOrdinal;
pub use application_runtime::WorthQueryPerformedInboundCompletion;
pub use application_runtime::WorthQueryRecordedCompletionDenial;
pub use application_runtime::WorthQueryUnpublishedInboundCompletion;
pub(in crate::domain_computation::primary_graph) use application_runtime::InstalledTransportCompletion;
pub(in crate::domain_computation::primary_graph) use application_runtime::InstalledTransportResumeOutcome;
pub use application_runtime::PerformedInstalledTransportCompletion;
pub(in crate::domain_computation) use application_runtime::WorthQueryInstalledTransportCompletionBinding;
pub(in crate::domain_computation) use entity_resolution::WorthQueryEntityResolutionTruth;
pub(in crate::domain_computation) use entity_resolution::WorthQueryObservedScopeSelector;
pub(in crate::domain_computation) use entity_resolution::WorthQueryInstalledEntityResolutionContext;
pub(in crate::domain_computation) use entity_resolution::WorthQueryResolvedEntity;
pub(in crate::domain_computation) use entity_resolution::WorthQueryIssuedSelectedScope;
pub(in crate::domain_computation) use resolution::WorthQueryIssuedSelectedPrincipal;
pub(in crate::domain_computation) use authenticated_principal::validate_freshness_at_snapshot;
pub(in crate::domain_computation) use freshness::WorthQueryDurablePrincipalCurrentness;
pub(in crate::domain_computation) use freshness::WorthQueryPrincipalFreshnessEvidence;

pub(in crate::domain_computation) use provider::WorthQueryApplicationBranchCommitCoordination;
pub(in crate::domain_computation) use provider::WorthQueryCommittedDispatchOutboxBinding;

pub(in crate::domain_computation) use schema_layout::WorthQueryPrimaryGraphLayout;
pub(in crate::domain_computation) use schema_layout::WorthQueryPrimaryPrincipalBindingLayout;

pub(in crate::domain_computation) use application_contribution::SealedComputationRun;

pub(in crate::domain_computation) use application_contribution::observe_discarded;

#[cfg(test)]
pub(crate) use application_contribution::with_test_advancement;
pub(crate) use application_contribution::TestHostRequest;

pub use application_contribution::with_bootstrap_advancement;
pub use application_contribution::with_bootstrap_advancement_from_request;

pub use application_contribution::WorthQueryBootstrapAdvancementPhase;

mod lineage_ports;
mod application_checkpoint_runtime;
pub(in crate::domain_computation::primary_graph) use application_output_demand::settlement::{
    OriginalSettledProducerObservationExt, OutputDemandSettlementCurrentExt,
    OutputDemandSettlementReusedExt, OutputDemandSettlementRuntimeExt,
};
pub(crate) use lineage_ports::RetainedPreImageExt;
pub(in crate::domain_computation) use lineage_ports::{
    SnapshotLeaseRuntimeExt, SourceReadAuthorizationRuntimeExt,
};
pub(in crate::domain_computation::primary_graph) use application_contribution::producer::registry::semantic_edition_ext::InstalledProducerEditionExt;
pub(in crate::domain_computation::primary_graph) use lineage_ports::{
    downcast_managed_views, downcast_managed_views_arc, CommitAuthorityBindingExt,
    PrimaryGraphHandleWorkflowExt, ReadObservationRuntimeExt,
    SourceFactExpansionControlExt, WorkflowRetentionStore,
};

pub use observations::principal_target_reads_on_this_thread_for_test;

pub use application_runtime::WorthQueryOperationInputCaptureDenial;
pub(in crate::domain_computation) mod selected_read_indexes;

pub(crate) mod causal_inspection;
mod record_reservations;

pub(in crate::domain_computation) use provider::QueryDurablePreparationGuard;

pub use derived_source::DerivedSourceObservation;
#[cfg(test)]
#[path = "mod_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;

pub use application_output_demand::WorthQueryOutputDiscoveryRecipe;

mod original_operation;
pub use original_operation::{WorthQueryOriginalOperationDenial, WorthQueryOriginalOperationOwner};

pub(crate) use worth_query_execution_lineage::facade::primary_graph::application_attempt::compare_and_commit::commit_outcome::denial::advancement::WorthQueryAdvancementDenialCommitExt;
