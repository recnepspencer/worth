mod access_strategy;
pub mod facade;
pub use facade::*;
mod aggregate_identity;
pub mod aggregate_observation;
mod aggregate_projection;
pub mod application_attempt;
pub mod application_branch;
pub mod application_checkpoint;
pub mod application_contribution;
pub(in crate::domain_computation) use application_contribution::ComputationPrior;
pub(in crate::domain_computation) use application_contribution::SealedComputationRetention;
pub(crate) mod application_discovery;
pub mod application_entry;
pub mod application_installation;
pub mod application_invariant;
mod application_invariant_preparation;
pub mod application_output_demand;
pub(in crate::domain_computation::primary_graph) use application_output_demand::PreparedPrerequisiteClaims;
pub(in crate::domain_computation) use application_output_demand::RequiredOutputDemandContext;
pub(in crate::domain_computation) use application_output_demand::RequiredOutputExecution;
pub use application_output_demand::WorthQueryCompletedReadOutputs;
pub use application_output_demand::WorthQueryCompletedRequiredOutputs;
pub use application_output_demand::WorthQueryFulfilledRequiredRoot;
pub use application_output_demand::{
    WorthQueryCompletedOriginalReadOutputs, WorthQueryOriginalReadOutputDenial,
};
pub mod application_program;
pub mod application_query;
pub(crate) mod application_resource;
pub mod application_runtime;
pub(crate) mod authenticated_principal;
mod authentication_clock;
pub mod bootstrap;
pub mod bootstrap_publication;
pub mod conditional_operation;
pub mod publication_request_denial;
pub(crate) use conditional_operation::classify_bridge_signal;
pub use conditional_operation::conditional_intent_read_is_active_for_test;
pub mod denial;
mod entity_key;
pub mod entity_resolution;
mod entity_resolution_denial;
pub mod exact_basis_access;
mod expression;
mod freshness;
mod granular_invalidation;
pub mod handler;
pub(in crate::domain_computation) use handler::DecisionContextUse;
pub mod index_currency;
pub mod index_maintenance_budget;
pub mod index_refresh;
mod initial_schema_denial;
mod invariant_installation;
pub mod invariant_projection;
pub mod live_delivery;
mod managed_bridge;
pub mod merge_unique_values;
mod read_observation;
pub mod source_facts;
pub(in crate::domain_computation) use source_facts::{
    validate_source_principal_facts, SourceFactsAllocation,
};
mod derived_source;
mod source_read_equivalence;
mod source_read_verification;
pub mod structural_walk;
pub(crate) use managed_bridge::build_primary_graph_product_bridge;
pub mod fact_verification_observation;
pub mod observations;
pub mod operation_body;
mod ordinary_read;
pub mod output_lineage;
mod request_allocation_control;
pub(in crate::domain_computation) use output_lineage::SourceInvalidationOwner;
pub mod output_binding_identity;
pub mod output_family_identity;
pub mod output_reuse;
mod principal_currentness_capture;
mod principal_key;
pub(in crate::domain_computation) use principal_currentness_capture::capture_principal_currentness_admitted;
pub(in crate::domain_computation) use principal_currentness_capture::PrincipalCurrentnessCaptureStop;
pub(crate) mod product_activation;
pub mod product_operation;
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
pub mod program_occurrence;
pub(in crate::domain_computation) use program_occurrence::OriginalProgramMeaning;
pub mod provider;
pub(in crate::domain_computation) use application_attempt::retained_decision_facts::StorageControl;
pub(in crate::domain_computation::primary_graph) use provider::OutstandingDispatchInFlightLease;
pub(in crate::domain_computation) use provider::PrimaryDecisionReadSetComparison;
pub(in crate::domain_computation) use provider::RegisteredPrimaryRead;
pub mod component_selection;
mod request_local;
mod resolution;
pub mod resolution_denial;
pub mod root;
mod root_path_selection;
pub mod schema_layout;
mod typed_bootstrap;
pub mod value_decode;
pub mod workflow;
pub use workflow::captured_input::{
    PreparedWorkflowCapturedInput, WorkflowCapturedInputContract, WorkflowCapturedInputDenial,
    WorkflowCapturedInputSource, WorkflowCapturedInputs,
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

pub(crate) use application_contribution::TestHostRequest;
pub(crate) use application_contribution::{with_test_advancement, with_test_bootstrap_advancement};

pub use application_contribution::with_bootstrap_advancement;
pub use application_contribution::with_bootstrap_advancement_from_request;

pub use application_contribution::WorthQueryBootstrapAdvancementPhase;

pub use crate::domain_computation::execution_runtime::product_world::installed_source_reads_on_this_thread_for_test;

pub use observations::principal_target_reads_on_this_thread_for_test;

pub use application_runtime::WorthQueryOperationInputCaptureDenial;
pub(in crate::domain_computation) mod selected_read_indexes;

pub mod admitted_capture;
pub mod causal_inspection;
mod record_reservations;

pub mod source_disclosure;
