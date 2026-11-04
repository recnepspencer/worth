mod content;
pub(in crate::domain_computation) use content::retention_checkpoint;
pub use content::{
    AdmittedContent, ContentReadDenial, ContentWriteDenial, RelationalContentDenial,
    RelationalContentRejection,
};
mod access_strategy;
mod facade;
pub use facade::*;
mod aggregate_identity;
pub(crate) mod aggregate_observation;
pub(crate) mod aggregate_projection;
pub(crate) mod application_attempt;
pub(crate) mod application_branch;
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
pub use application_output_demand::{
    WorthQueryCompletedOriginalReadOutputs, WorthQueryOriginalReadOutputDenial,
};
mod application_program;
pub(crate) mod application_query;
pub(crate) mod application_resource;
pub(crate) mod application_runtime;
pub(in crate::domain_computation) use application_runtime::WorthQueryApplicationRuntimeCore;
pub(crate) mod authenticated_principal;
mod bootstrap;
pub(crate) mod bootstrap_publication;
pub(crate) mod conditional_operation;
pub(crate) mod publication_request_denial;
pub(crate) use conditional_operation::classify_bridge_signal;
pub use conditional_operation::conditional_intent_read_is_active_for_test;
mod denial;
pub(crate) mod entity_key;
pub(crate) mod entity_resolution;
mod entity_resolution_denial;
pub(crate) mod exact_basis_access;
mod expression;
pub(crate) mod freshness;
mod granular_invalidation;
pub(crate) mod handler;
pub(in crate::domain_computation) use handler::DecisionContextUse;
pub(crate) mod index_currency;
pub(crate) mod index_maintenance_budget;
pub(crate) mod index_refresh;
mod initial_schema_denial;
mod invariant_installation;
pub(crate) mod invariant_projection;
pub(crate) mod live_delivery;
pub(crate) mod managed_bridge;
pub(crate) mod merge_unique_values;
mod read_observation;
pub(crate) mod source_facts;
pub(in crate::domain_computation) use source_facts::{
    validate_source_principal_facts, SourceFactsAllocation,
};
pub(crate) mod derived_source;
mod source_read_equivalence;
pub(crate) mod source_read_verification;
pub(crate) mod structural_walk;
pub(crate) use managed_bridge::build_primary_graph_product_bridge;
pub(crate) mod fact_verification_observation;
mod observations;
mod operation_body;
mod ordinary_read;
pub(crate) mod output_lineage;
mod request_allocation_control;
pub(in crate::domain_computation) use output_lineage::SourceInvalidationOwner;
pub(crate) mod output_binding_identity;
pub(crate) mod output_family_identity;
pub(crate) mod output_reuse;
mod principal_currentness_capture;
mod principal_key;
pub(in crate::domain_computation) use principal_currentness_capture::capture_principal_currentness_admitted;
pub(in crate::domain_computation) use principal_currentness_capture::PrincipalCurrentnessCaptureStop;
pub(crate) mod product_activation;
pub(crate) mod product_operation;
pub use application_query::{
    WorthQueryHistoryOutputIntent, WorthQueryHistoryOutputNavigationDenial,
};
pub use product_operation::document_history::{
    WorthQueryAdmittedHistoryProgramOutput, WorthQueryCompletedHistoryOutputs,
    WorthQueryDocumentHistory, WorthQueryDocumentHistoryAdoptionFailure,
    WorthQueryDocumentHistoryAttachmentFailure, WorthQueryDocumentHistoryCandidateDenial,
    WorthQueryDocumentHistoryCapturePauseForTest, WorthQueryDocumentHistoryCloseFailure,
    WorthQueryDocumentHistoryCloseReceipt, WorthQueryDocumentHistoryClosing,
    WorthQueryDocumentHistoryClosingDenial, WorthQueryDocumentHistoryDenial,
    WorthQueryDocumentHistoryGroupAborting, WorthQueryDocumentHistoryGroupAdoptionFailure,
    WorthQueryDocumentHistoryGroupPreparation, WorthQueryDocumentHistoryGroupReservation,
    WorthQueryDocumentHistoryIdentity, WorthQueryDocumentHistoryNavigation,
    WorthQueryDocumentHistoryOperation, WorthQueryDocumentHistoryOperationConstraint,
    WorthQueryDocumentHistoryRecordPreparation, WorthQueryDocumentHistoryReservation,
    WorthQueryHistoryCommandContextDenial, WorthQueryHistoryContextRefreshDenial,
    WorthQueryHistoryContextRefreshLimits, WorthQueryHistoryContextRefreshPauseForTest,
    WorthQueryHistoryDependentOutputRead, WorthQueryHistoryDescriptionContext,
    WorthQueryHistoryEnrollmentObservationForTest, WorthQueryHistoryEnrollmentReportForTest,
    WorthQueryHistoryGroupCandidateCreation, WorthQueryHistoryGroupMemberAdmission,
    WorthQueryHistoryNavigationCompletion, WorthQueryHistoryNavigationDenial,
    WorthQueryHistoryNavigationEligibilityDenial, WorthQueryHistoryNavigationIntent,
    WorthQueryHistoryNavigationPauseForTest, WorthQueryHistoryNavigationPosture,
    WorthQueryHistoryNodeId, WorthQueryHistoryOutputAdvanceDenial,
    WorthQueryHistoryOutputAuthority, WorthQueryHistoryOutputDiscovery,
    WorthQueryHistoryOutputNavigationAdvance, WorthQueryHistoryOutputProfile,
    WorthQueryHistoryOutputSource, WorthQueryHistoryPage,
    WorthQueryHistoryPageAssemblyPauseForTest, WorthQueryHistoryPageCursor,
    WorthQueryHistoryPageDenial, WorthQueryHistoryPageEntry, WorthQueryHistoryPageLimits,
    WorthQueryHistoryPageRow, WorthQueryHistoryParentBoundary, WorthQueryHistoryPreview,
    WorthQueryHistoryProgramOutputAdvance, WorthQueryHistoryRedoChoice,
    WorthQueryHistoryRedoChoices, WorthQueryHistoryReleaseOutcome,
    WorthQueryHistoryRetentionChangePauseForTest, WorthQueryHistoryRetentionStatus,
    WorthQueryHistoryRootOutputAdmissionDenial, WorthQueryHistorySelection, WorthQueryHistoryStep,
    WorthQuerySettledHistoryProgramOutput,
};
pub(in crate::domain_computation) use product_operation::SharedSelectedProductOperation;
pub use product_operation::WorthQueryHistoryOutputControlDenial;
pub use product_operation::WorthQueryHistoryOutputReadDenial;
pub use product_operation::WorthQueryHistoryRootOutputRead;
pub(crate) mod program_occurrence;
pub(in crate::domain_computation) use program_occurrence::OriginalProgramMeaning;
pub(crate) mod provider;
pub(in crate::domain_computation) use application_attempt::retained_decision_facts::StorageControl;
pub(in crate::domain_computation::primary_graph) use provider::OutstandingDispatchInFlightLease;
pub(in crate::domain_computation) use provider::PrimaryDecisionReadSetComparison;
pub(in crate::domain_computation) use provider::RegisteredPrimaryRead;
pub(crate) mod component_selection;
mod request_local;
pub(crate) mod resolution;
pub(crate) mod resolution_denial;
pub(crate) mod root;
mod root_path_selection;
pub(crate) mod schema_layout;
mod typed_bootstrap;
pub(crate) mod value_decode;
pub use {
    crate::domain_computation::primary_graph::application_attempt::workflow::captured_input::PreparedWorkflowCapturedInput,
    crate::domain_computation::primary_graph::application_attempt::workflow::captured_input::WorkflowCapturedInputContract,
    crate::domain_computation::primary_graph::application_attempt::workflow::captured_input::WorkflowCapturedInputDenial,
    crate::domain_computation::primary_graph::application_attempt::workflow::captured_input::WorkflowCapturedInputSource,
    crate::domain_computation::primary_graph::application_attempt::workflow::captured_input::WorkflowCapturedInputs,
};

pub(in crate::domain_computation) use application_attempt::application_resource_request;
pub(in crate::domain_computation) use application_attempt::precondition_binding::bind_empty_mutation_preconditions_admitted;
pub(in crate::domain_computation) use application_attempt::precondition_binding::bind_mutation_preconditions;
pub(in crate::domain_computation) use application_attempt::precondition_binding::EmptyMutationPreconditionAdmissionStop;
pub(in crate::domain_computation) use application_attempt::precondition_binding::WorthQueryBoundMutationPreconditions;
pub(in crate::domain_computation) use application_attempt::WorthQueryApplicationSnapshotLease;
pub(in crate::domain_computation) use application_attempt::WorthQueryApplicationSnapshotLeaseDenial;
pub(in crate::domain_computation) use application_attempt::WorthQueryApplicationSnapshotRelease;
pub(in crate::domain_computation) use exact_basis_access::WorthQueryExactBasisSnapshotDenial;
pub(in crate::domain_computation) use application_attempt::WorthQueryApplicationObservedFact;
pub(in crate::domain_computation) use application_query::observed_source::WorthQuerySourceExpectationKind;
pub(in crate::domain_computation) use application_attempt::progression_denied;
pub(in crate::domain_computation) use application_attempt::WorthQueryApplicationAttemptAffinity;
pub(in crate::domain_computation) use application_attempt::WorthQueryApplicationAttemptBasis;
pub(in crate::domain_computation) use application_attempt::WorthQueryPreparedApplicationProviderAttempt;
pub(in crate::domain_computation) use application_attempt::WorthQueryProviderAttemptRegistrationContext;
pub(in crate::domain_computation) use application_attempt::WorthQueryProviderProgressionOutcome;
pub(in crate::domain_computation) use application_attempt::WorthQueryRegisteredProviderAttempt;
pub(crate) use application_attempt::WorthQueryRetainedGovernedInput;
pub(in crate::domain_computation::primary_graph) use application_attempt::WorthQueryApplicationOutputCorrespondence;
pub(crate) use application_attempt::WorthQueryPerformedExternalRedispatchSeal;
pub(crate) use provider::WorthQueryRetainedPreImageSeal;
pub(crate) use provider::WorthQueryPrimaryGraphProvider;
pub(in crate::domain_computation) use provider::ApplicationFactTable;
pub(in crate::domain_computation) use provider::CompletedApplicationFacts;
pub(in crate::domain_computation) use provider::WorthQueryPrimaryGraphApplicationDecisionFact;
pub(in crate::domain_computation) use provider::WorthQueryUnpublishedIdempotencyDisposition;
pub(in crate::domain_computation) use application_branch::primary_relational_branch_id;
pub(in crate::domain_computation) use application_branch::primary_truth_branch_identity;
pub(crate) use application_query::WorthQueryApplicationHistoricalRead;
pub(crate) use application_query::WorthQueryApplicationQueryControls;
pub(in crate::domain_computation) use crate::domain_computation::application_aftermath::external_effect::WorthQueryAdmittedExternalDispatchAttempt;
pub(in crate::domain_computation) use application_runtime::WorthQueryExternalDispatchAttemptOrdinal;
pub(in crate::domain_computation) use application_runtime::WorthQueryPerformedInboundCompletion;
pub(in crate::domain_computation) use application_runtime::WorthQueryUnpublishedInboundCompletion;
pub(in crate::domain_computation::primary_graph) use application_runtime::InstalledTransportCompletion;
pub(in crate::domain_computation::primary_graph) use application_runtime::InstalledTransportResumeOutcome;
pub(in crate::domain_computation::primary_graph) use application_runtime::PerformedInstalledTransportCompletion;
pub(in crate::domain_computation) use application_runtime::WorthQueryInstalledTransportCompletionBinding;
pub(in crate::domain_computation) use entity_resolution::WorthQueryEntityResolutionTruth;
pub(in crate::domain_computation) use entity_resolution::WorthQueryObservedScopeSelector;
pub(in crate::domain_computation) use entity_resolution::WorthQueryInstalledEntityResolutionContext;
pub(in crate::domain_computation) use entity_resolution::WorthQueryResolvedEntity;
pub(in crate::domain_computation) use entity_resolution::WorthQueryIssuedSelectedScope;
pub(in crate::domain_computation) use resolution::WorthQueryIssuedSelectedPrincipal;
pub(in crate::domain_computation) use freshness::validate_freshness_at_snapshot;
pub(in crate::domain_computation) use freshness::WorthQueryDurablePrincipalCurrentness;
pub(in crate::domain_computation) use freshness::WorthQueryPrincipalFreshnessEvidence;

pub(in crate::domain_computation) use provider::WorthQueryApplicationBranchCommitCoordination;
pub(in crate::domain_computation) use provider::WorthQueryCommittedDispatchOutboxBinding;

pub(in crate::domain_computation) use schema_layout::WorthQueryPrimaryGraphLayout;
pub(in crate::domain_computation) use schema_layout::WorthQueryPrimaryPrincipalBindingLayout;

pub(in crate::domain_computation) use application_contribution::SealedComputationRun;

pub(in crate::domain_computation) use application_contribution::observe_discarded;

pub use crate::domain_computation::execution_runtime::product_world::installed_source_reads_on_this_thread_for_test;

pub use observations::principal_target_reads_on_this_thread_for_test;

pub use application_runtime::WorthQueryOperationInputCaptureDenial;
pub(in crate::domain_computation) mod selected_read_indexes;

pub(crate) mod admitted_capture;
pub(crate) mod causal_inspection;
mod record_reservations;

pub(crate) mod request_execution;
pub(crate) mod source_disclosure;
