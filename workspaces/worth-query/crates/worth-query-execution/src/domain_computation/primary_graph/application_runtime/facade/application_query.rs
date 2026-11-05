//! The runtime's consumer methods over `domain_computation::primary_graph::application_query`; each only calls its subsystem.
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationHistoryCursor;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationHistoryDenial;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationHistoryPage;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationHistoryPageLimits;
use crate::domain_computation::primary_graph::application_query::WorthQueryCapturedApplicationHistorySubjects;
use crate::domain_computation::primary_graph::WorthQueryDocumentHistoryDenial;
use crate::domain_computation::primary_graph::WorthQueryHistoryOutputNavigationDenial;
use crate::domain_computation::primary_graph::application_query::WorthQueryManagedDerivedViewDenial;
use crate::domain_computation::primary_graph::application_query::WorthQueryManagedIndexDenial;
use crate::domain_computation::primary_graph::application_query::WorthQueryAdmittedApplicationQueryPlan;
use crate::domain_computation::primary_graph::application_query::WorthQueryManagedDerivedViewKey;
use crate::domain_computation::primary_graph::provider::aftermath_causality::correction_history::page_allocation::Denial;
use crate::domain_computation::primary_graph::application_query::WorthQueryAdmittedApplicationHistoryCapture;
use crate::domain_computation::request_execution::WorthQueryAdvancementPhase;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationContinuationDenial;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationContinuationPageResult;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationHistoryCaptureLimits;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryContinuation;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationReadObservation;
use crate::domain_computation::primary_graph::WorthQueryDocumentHistory;
use crate::domain_computation::primary_graph::WorthQueryDocumentHistoryAttachmentFailure;
use crate::domain_computation::primary_graph::application_query::WorthQueryDocumentHistoryRead;
use crate::domain_computation::primary_graph::application_query::WorthQueryHistoryPreviewReadAdmission;
use crate::domain_computation::primary_graph::application_query::WorthQueryManagedIndex;
use crate::domain_computation::primary_graph::application_query::WorthQueryManagedIndexWatch;
use crate::domain_computation::primary_graph::application_query::WorthQueryManagedIndexWatchOutcome;
use crate::domain_computation::primary_graph::product_operation::WorthQueryOwnedProductHead;
use std::io::Cursor;
use worth_query_installation::facade::ApplicationSchema;
use std::collections::BTreeMap;
use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_query_installation::facade::ApplicationSchemaBindingIdentity;
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryBinding;
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryIdentity;
use crate::domain_computation::primary_graph::application_query::batch::*;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use crate::domain_computation::authorization::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;
use std::marker::PhantomData;
use worth_query_admission::facade::application_query::WorthQueryApplicationQueryLane;
use crate::domain_computation::primary_graph::application_query::authorized_read::WorthQueryAuthorizedApplicationReadDenial;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationProjection;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryAccessReceipt;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest;
use worth_query_declaration::facade::application_query::ApplicationQueryParameterSet;
use worth_query_installation::facade::WorthQueryInstalledApplicationQuery;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryAccessContext;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryAdmissionDenial;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryAdmissionDenialKind;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryControls;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryResumeControls;
use std::sync::Arc;
use worth_query_declaration::facade::application_query::ApplicationDerivedViewDefinition;
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryAuthorization;
use crate::domain_computation::primary_graph::application_query::derived_view::WorthQueryManagedDerivedValue;
use crate::domain_computation::primary_graph::application_query::derived_view::WorthQueryManagedDerivedView;
use crate::domain_computation::primary_graph::application_query::derived_view::WorthQueryManagedDerivedViewSnapshot;
use crate::basis::WorthQueryProductBranchLease;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationOneShotResult;
use crate::domain_computation::primary_graph::application_query::WorthQueryObservedSource;
use crate::domain_computation::primary_graph::application_query::derived_view::reconstruction_denial;
use worth_relational::facade::identity::EntityId;
use crate::domain_computation::primary_graph::application_query::derived_view::WorthQueryManagedDerivedViewReconciliation;
use crate::domain_computation::primary_graph::application_query::derived_view::registration::collection::pair::reconstruction::owner_stage as stage;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryBatchAdmission;
use crate::domain_computation::primary_graph::application_query::WorthQueryDependentPairReadContext;
use worth_execution::ExecutionMemoryReservation;
use worth_execution::ExecutionRequest;
use worth_execution::ExecutionWorkCeiling;
use crate::domain_computation::primary_graph::application_query::derived_view::registration::collection::pair::reconstruction::owner_stage;
use crate::domain_computation::primary_graph::application_query::DispatchWitness;
use crate::domain_computation::primary_graph::application_query::read_execution::read_plan::ReadPlan;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationAuthorizationWorkEvidence;
use crate::domain_computation::primary_graph::application_query::WorthQueryDerivedPairReadPlans;
use crate::domain_computation::primary_graph::value_decode::ValueDecodeState;
use worth_execution::ExecutionMap;
use worth_foundational::PartitionIdentity;
use worth_proof::CanonicalUniqueVec;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationBatchResult;
use crate::domain_computation::primary_graph::product_operation::document_history::confirm_document_head;
use crate::publication_boundary::WorthQueryProgramPublicationAccess;
use worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_declaration::facade::application_history::ApplicationHistoryAccessBinding;
use worth_query_declaration::facade::application_history::ApplicationHistoryBinding;
use worth_query_declaration::facade::application_history::ApplicationHistoryModel;
use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
use worth_query_installation::facade::WorthQueryInstalledDocumentHistory;
use crate::basis::WorthQueryProductBranchReadIdentity;
use crate::domain_computation::primary_graph::WorthQueryHistoryContextRefreshDenial;
use crate::domain_computation::primary_graph::WorthQueryHistoryContextRefreshLimits;
use worth_query_declaration::facade::application_history::ApplicationHistoryModelDomain;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationHistoryDenial as MetadataDenial;
use crate::domain_computation::authorization::application_disclosure::contract::compile_document_history_disclosure_contract;
use crate::domain_computation::primary_graph::WorthQueryHistoryStep;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityRef;
use worth_query_declaration::facade::application_schema::ApplicationOperationMarkerIdentity;
use worth_query_declaration::facade::application_schema::ApplicationOperationRef;
use crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation;
use crate::domain_computation::primary_graph::WorthQueryDocumentHistoryNavigation;
use crate::domain_computation::primary_graph::WorthQueryHistoryNavigationCompletion;
use crate::domain_computation::primary_graph::WorthQueryHistoryNavigationDenial as NavigationDenial;
use crate::domain_computation::primary_graph::WorthQueryHistoryNavigationEligibilityDenial;
use crate::domain_computation::primary_graph::WorthQueryHistoryNavigationIntent;
use crate::domain_computation::primary_graph::WorthQueryHistorySelection;
use worth_query_declaration::facade::application_history::ApplicationHistoryNavigationBinding;
use crate::domain_computation::primary_graph::application_query::HistoryOutputControlPlan;
use crate::domain_computation::primary_graph::product_operation::document_history::HistoryOutputLookup;
use crate::domain_computation::primary_graph::product_operation::document_history::HistoryOutputLookupPhase;
use crate::domain_computation::primary_graph::WorthQueryCompletedHistoryOutputs;
use crate::domain_computation::primary_graph::WorthQueryHistoryOutputNavigationAdvance as Step;
use crate::domain_computation::primary_graph::WorthQueryHistoryOutputProfile;
use crate::domain_computation::primary_graph::WorthQueryHistoryOutputReadDenial;
use crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation;
use worth_query_declaration::facade::application_operation::ApplicationMutationBinding;
use worth_query_declaration::facade::application_operation::ApplicationMutationScopeBinding;
use crate::domain_computation::primary_graph::product_operation::document_history::HistoryModelIdentity;
use crate::domain_computation::primary_graph::WorthQueryHistoryNavigationDenial;
use crate::domain_computation::primary_graph::WorthQueryObservedScopeSelector;
use worth_query_admission::facade::application_query::WorthQueryAdmittedApplicationQueryParameters;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
use crate::domain_computation::primary_graph::WorthQueryHistoryPage;
use crate::domain_computation::primary_graph::WorthQueryHistoryPageCursor;
use crate::domain_computation::primary_graph::WorthQueryHistoryPageDenial;
use crate::domain_computation::primary_graph::WorthQueryHistoryPageLimits;
use crate::domain_computation::primary_graph::WorthQueryHistoryNodeId;
use crate::domain_computation::primary_graph::WorthQueryHistoryPreview;
use crate::domain_computation::primary_graph::WorthQueryHistoryRedoChoices;
use crate::domain_computation::primary_graph::product_operation::document_history::HistoryRetentionChange;
use crate::domain_computation::primary_graph::WorthQueryHistoryRetentionStatus;
use worth_query_declaration::facade::application_history::ApplicationHistoryBookmarkBinding;
use worth_query_declaration::facade::application_history::ApplicationHistoryRecentSteps;
use worth_query_declaration::facade::application_history::ApplicationHistoryRetentionBinding;
use worth_query_declaration::facade::application_history::ApplicationHistoryWindowBinding;
use worth_query_declaration::facade::application_query::ApplicationHistoryQueryBinding;
use worth_query_installation::facade::WorthQueryInstalledApplicationHistoryQuery;
use crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess;
use worth_query_declaration::facade::application_query::ApplicationQueryScopeBinding;
use worth_query_declaration::facade::application_query::ApplicationManagedIndexBinding;
use crate::domain_computation::primary_graph::application_query::derived_view::PredicateNoticeDisposition;
use crate::domain_computation::primary_graph::live_delivery::WorthQueryLivePredicatePoll;
use crate::domain_computation::primary_graph::application_query::derived_view::PredicateWorld;
use crate::domain_computation::primary_graph::live_delivery::WorthQueryLiveAdmissionDenial;
use crate::domain_computation::primary_graph::application_query::observed_source::*;
use crate::domain_computation::authorization::WorthQueryOperationAdmissionIdentity;
use crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption;
use worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope;
use worth_relational::facade::mvcc::CompanionPreflightStop;
use denial::authorization_denial;
use crate::domain_computation::primary_graph::application_query::read_execution::read_bounded_root_rows;
use crate::domain_computation::primary_graph::application_query::WorthQueryAdmittedApplicationQueryControls;
use crate::domain_computation::primary_graph::WorthQueryAuthenticatedPrincipal;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationQueryBatchResourceDenial;
use crate::domain_computation::primary_graph::application_query::one_shot::*;
use crate::domain_computation::primary_graph::application_query::one_shot::map_authorized_read_denial;
use crate::domain_computation::primary_graph::application_query::one_shot::WorthQueryApplicationOneShotDenialKind;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationBasisIdentity;
use crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt;
use crate::basis::WorthQueryProductBranchAdmissionRefusalKind as AdmissionRefusal;
use crate::domain_computation::primary_graph::product_operation::ContextScopedOps;
use crate::basis::WorthQueryProductBranchAdmissionDenial;
use crate::basis::WorthQueryProductObservationLease;
impl<S: ApplicationSchema> super::super::WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub fn attach_document_history<'head, H: ApplicationHistoryBinding<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _access: &WorthQueryProgramPublicationAccess,
        head: WorthQueryOwnedProductHead<'head, S>,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
    ) -> Result<
        WorthQueryDocumentHistory<'head, S, H>,
        WorthQueryDocumentHistoryAttachmentFailure<'head, S>,
    >
    where
    <<ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<S, <ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::Query>,{
        <Self as crate::domain_computation::primary_graph::application_query::DocumentHistoryOps<
            S,
        >>::attach_document_history(self, phase, _access, head, read)
    }

    pub fn read_document_history<H: ApplicationHistoryBinding<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
    ) -> Result<WorthQueryApplicationOneShotResult<<ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::Query, <<ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value>, WorthQueryDocumentHistoryDenial>
    where
    <<ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<S, <ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::Query>,{
        <Self as crate::domain_computation::primary_graph::application_query::DocumentHistoryOps<
            S,
        >>::read_document_history(self, phase, _access, document, read)
    }

    pub fn refresh_document_history_context<H: ApplicationHistoryBinding<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        document: &mut WorthQueryDocumentHistory<'_, S, H>,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
        limits: WorthQueryHistoryContextRefreshLimits,
    ) -> Result<(), WorthQueryDocumentHistoryDenial>
    where
    <<ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<S, <ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::Query>,{
        <Self as crate::domain_computation::primary_graph::application_query::ContextOps<S>>::refresh_document_history_context(self, phase, access, document, read, limits)
    }

    pub fn read_document_history_step<H: ApplicationHistoryBinding<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
    ) -> Result<WorthQueryHistoryStep, WorthQueryDocumentHistoryDenial>
    where
    <<ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<S, <ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::Query>,{
        <Self as crate::domain_computation::primary_graph::application_query::MetadataOps<S>>::read_document_history_step(self, phase, access, document, read)
    }

    pub fn read_document_history_selection<H: ApplicationHistoryBinding<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
    ) -> Result<crate::domain_computation::primary_graph::WorthQueryHistorySelection, WorthQueryDocumentHistoryDenial>
    where
    <<ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<S, <ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::Query>,{
        <Self as crate::domain_computation::primary_graph::application_query::MetadataOps<S>>::read_document_history_selection(self, phase, access, document, read)
    }

    pub fn prepare_document_history_navigation<'runtime, H, N>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'runtime, S, H>,
        history_access: &H::Access,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
        expected: WorthQueryHistorySelection,
        intent: WorthQueryHistoryNavigationIntent,
        key: u128,
    ) -> Result<WorthQueryDocumentHistoryNavigation<'runtime, S>, WorthQueryHistoryNavigationDenial>
    where
        H: ApplicationHistoryBinding<S>,
        N: ApplicationHistoryNavigationBinding<S, H>,
    <<ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<S, <ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::Query>,{
        <Self as crate::domain_computation::primary_graph::application_query::NavigationOps<S>>::prepare_document_history_navigation::<H, N>(self, phase, access, document, history_access, read, expected, intent, key)
    }

    pub fn advance_document_history_navigation<'runtime, H, N>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'runtime, S, H>,
        history_access: &H::Access,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
        navigation: &WorthQueryDocumentHistoryNavigation<'runtime, S>,
        key: u128,
    ) -> Result<Option<WorthQueryHistoryNavigationCompletion>, WorthQueryHistoryNavigationDenial>
    where
        H: ApplicationHistoryBinding<S>,
        N: ApplicationHistoryNavigationBinding<S, H>,
    <<ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<S, <ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::Query>,{
        <Self as crate::domain_computation::primary_graph::application_query::NavigationOps<S>>::advance_document_history_navigation::<H, N>(self, phase, access, document, history_access, read, navigation, key)
    }

    pub fn prepare_document_history_navigation_with_outputs<'runtime, H, N, B>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'runtime, S, H>,
        history_access: &H::Access,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
        expected: WorthQueryHistorySelection,
        intent: WorthQueryHistoryNavigationIntent,
        key: u128,
        profile: WorthQueryHistoryOutputProfile,
        source_binding: &WorthQueryInstalledApplicationQueryBinding<S, B>,
        source: HistoryOutputControlPlan<'_, S, B>,
    ) -> Result<WorthQueryDocumentHistoryNavigation<'runtime, S>, WorthQueryHistoryOutputNavigationDenial>
    where
        H: ApplicationHistoryBinding<S>,
        N: ApplicationHistoryNavigationBinding<S, H>,
        B: ApplicationQueryBinding<S>,
    <<ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<S, <ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::Query>,{
        <Self as crate::domain_computation::primary_graph::application_query::OutputsOps<S>>::prepare_document_history_navigation_with_outputs::<H, N, B>(self, phase, access, document, history_access, read, expected, intent, key, profile, source_binding, source)
    }

    pub fn assess_document_history_operation<H, M>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        installed: &WorthQueryInstalledDocumentHistory<S, H>,
        selected: &WorthQuerySelectedProductOperation<'_, S>,
        admitted: &WorthQueryAdmittedApplicationOperation<S, M::Operation, M::Input, <<M as ApplicationMutationBinding<S>>::ScopeBinding as ApplicationMutationScopeBinding<S>>::Scope>,
    ) -> Result<(), WorthQueryDocumentHistoryDenial>
    where
        H: ApplicationHistoryBinding<S>,
        M: ApplicationMutationBinding<S>,
        M::ScopeBinding: ApplicationMutationScopeBinding<S, Scope = ApplicationHistoryModel<S, H>>,
    {
        <Self as crate::domain_computation::primary_graph::application_query::OperationOps<S>>::assess_document_history_operation::<H, M>(self, phase, _access, document, installed, selected, admitted)
    }

    pub fn read_document_history_page<H: ApplicationHistoryBinding<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
        limits: WorthQueryHistoryPageLimits,
        after: Option<&WorthQueryHistoryPageCursor>,
    ) -> Result<WorthQueryHistoryPage, WorthQueryDocumentHistoryDenial>
    where
    <<ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<S, <ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::Query>,{
        <Self as crate::domain_computation::primary_graph::application_query::PagingOps<S>>::read_document_history_page(self, phase, access, document, read, limits, after)
    }

    pub fn open_document_history_preview<H: ApplicationHistoryBinding<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
        node: WorthQueryHistoryNodeId,
    ) -> Result<WorthQueryHistoryPreview, WorthQueryDocumentHistoryDenial>
    where
    <<ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<S, <ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::Query>,{
        <Self as crate::domain_computation::primary_graph::application_query::PreviewOps<S>>::open_document_history_preview(self, phase, access, document, read, node)
    }

    pub fn prepare_document_history_preview_read<'phase, 'scope, H: ApplicationHistoryBinding<S>>(
        &self,
        phase: &'phase WorthQueryAdvancementPhase<'scope>,
        access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        preview: &WorthQueryHistoryPreview,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
    ) -> Result<WorthQueryHistoryPreviewReadAdmission<'phase, 'scope>, WorthQueryDocumentHistoryDenial>
    where
    <<ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<S, <ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::Query>,{
        <Self as crate::domain_computation::primary_graph::application_query::PreviewOps<S>>::prepare_document_history_preview_read(self, phase, access, document, preview, read)
    }

    pub fn read_document_history_preview<H: ApplicationHistoryBinding<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        admission: &WorthQueryHistoryPreviewReadAdmission<'_, '_>,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
    ) -> Result<WorthQueryApplicationOneShotResult<<ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::Query, <<ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value>, WorthQueryDocumentHistoryDenial>
    where
    <<ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<S, <ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::Query>,{
        <Self as crate::domain_computation::primary_graph::application_query::PreviewOps<S>>::read_document_history_preview(self, phase, _access, document, admission, read)
    }

    pub fn read_document_history_redo_choices<H: ApplicationHistoryBinding<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
        limits: WorthQueryHistoryPageLimits,
        after: Option<&WorthQueryHistoryPageCursor>,
    ) -> Result<WorthQueryHistoryRedoChoices, WorthQueryDocumentHistoryDenial>
    where
    <<ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<S, <ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::Query>,{
        <Self as crate::domain_computation::primary_graph::application_query::RedoOps<S>>::read_document_history_redo_choices(self, phase, access, document, read, limits, after)
    }

    pub fn change_document_history_window<H, N>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        history_access: &H::Access,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
        window: ApplicationHistoryRecentSteps,
    ) -> Result<WorthQueryHistoryRetentionStatus, WorthQueryDocumentHistoryDenial>
    where
        H: ApplicationHistoryBinding<S>,
        N: ApplicationHistoryWindowBinding<S, H>,
    <<ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<S, <ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::Query>,{
        <Self as crate::domain_computation::primary_graph::application_query::RetentionOps<S>>::change_document_history_window::<H, N>(self, phase, access, document, history_access, read, window)
    }

    pub fn change_document_history_bookmark<H, N>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        history_access: &H::Access,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
        node: WorthQueryHistoryNodeId,
        pin: bool,
    ) -> Result<WorthQueryHistoryRetentionStatus, WorthQueryDocumentHistoryDenial>
    where
        H: ApplicationHistoryBinding<S>,
        N: ApplicationHistoryBookmarkBinding<S, H>,
    <<ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<S, <ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::Query>,{
        <Self as crate::domain_computation::primary_graph::application_query::RetentionOps<S>>::change_document_history_bookmark::<H, N>(self, phase, access, document, history_access, read, node, pin)
    }

    pub fn read_document_history_retention<H: ApplicationHistoryBinding<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'_, S, H>,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
    ) -> Result<WorthQueryHistoryRetentionStatus, WorthQueryDocumentHistoryDenial>
    where
    <<ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<S, <ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::Query>,{
        <Self as crate::domain_computation::primary_graph::application_query::RetentionOps<S>>::read_document_history_retention(self, phase, access, document, read)
    }

    pub fn execute_application_query_binding_one_shot<B>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        binding: &WorthQueryInstalledApplicationQueryBinding<S, B>,
        external: &WorthQueryAuthenticatedExternalPrincipal<S>,
        plan: WorthQueryAdmittedApplicationQueryPlan< '_, S, <B as ApplicationQueryBinding<S>>::Query, <<B as ApplicationQueryBinding<S>>::ParameterBinding as ApplicationStructuredValueBinding>::Value, <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value, <B as ApplicationQueryBinding<S>>::Principal, <B as ApplicationQueryBinding<S>>::PrincipalIdentity, <<B as ApplicationQueryBinding<S>>::ScopeBinding as ApplicationQueryScopeBinding<S>>::Scope, >,
    ) -> Result<WorthQueryApplicationOneShotResult<B::Query, <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value>, WorthQueryManagedIndexDenial>
    where
        B: ApplicationQueryBinding<S>,
    <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<S, B::Query>,{
        <Self as crate::domain_computation::primary_graph::application_query::ReceiverOps<S>>::execute_application_query_binding_one_shot(self, phase, binding, external, plan)
    }

    pub fn evict_application_managed_index<
        B: ApplicationQueryBinding<S>,
        I: ApplicationManagedIndexBinding<S>,
    >(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        binding: &WorthQueryInstalledApplicationQueryBinding<S, B>,
        mut plan: WorthQueryAdmittedApplicationQueryPlan< '_, S, <B as ApplicationQueryBinding<S>>::Query, <<B as ApplicationQueryBinding<S>>::ParameterBinding as ApplicationStructuredValueBinding>::Value, <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value, <B as ApplicationQueryBinding<S>>::Principal, <B as ApplicationQueryBinding<S>>::PrincipalIdentity, <<B as ApplicationQueryBinding<S>>::ScopeBinding as ApplicationQueryScopeBinding<S>>::Scope, >,
        index: &WorthQueryManagedIndex<S, I>,
    ) -> Result<(), WorthQueryManagedIndexDenial> {
        <Self as crate::domain_computation::primary_graph::application_query::EvictionOps<S>>::evict_application_managed_index(self, phase, binding, plan, index)
    }

    pub fn open_application_managed_index<
        B: ApplicationQueryBinding<S>,
        I: ApplicationManagedIndexBinding<S>,
    >(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        binding: &WorthQueryInstalledApplicationQueryBinding<S, B>,
        mut plan: WorthQueryAdmittedApplicationQueryPlan< '_, S, <B as ApplicationQueryBinding<S>>::Query, <<B as ApplicationQueryBinding<S>>::ParameterBinding as ApplicationStructuredValueBinding>::Value, <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value, <B as ApplicationQueryBinding<S>>::Principal, <B as ApplicationQueryBinding<S>>::PrincipalIdentity, <<B as ApplicationQueryBinding<S>>::ScopeBinding as ApplicationQueryScopeBinding<S>>::Scope, >,
    ) -> Result<WorthQueryManagedIndex<S, I>, WorthQueryManagedIndexDenial> {
        <Self as crate::domain_computation::primary_graph::application_query::LifecycleOps<S>>::open_application_managed_index(self, phase, binding, plan)
    }

    pub fn rebuild_application_managed_index<
        B: ApplicationQueryBinding<S>,
        I: ApplicationManagedIndexBinding<S>,
    >(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        binding: &WorthQueryInstalledApplicationQueryBinding<S, B>,
        external: &WorthQueryAuthenticatedExternalPrincipal<S>,
        mut plan: WorthQueryAdmittedApplicationQueryPlan< '_, S, <B as ApplicationQueryBinding<S>>::Query, <<B as ApplicationQueryBinding<S>>::ParameterBinding as ApplicationStructuredValueBinding>::Value, <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value, <B as ApplicationQueryBinding<S>>::Principal, <B as ApplicationQueryBinding<S>>::PrincipalIdentity, <<B as ApplicationQueryBinding<S>>::ScopeBinding as ApplicationQueryScopeBinding<S>>::Scope, >,
        index: &WorthQueryManagedIndex<S, I>,
    ) -> Result<(), WorthQueryManagedIndexDenial> {
        <Self as crate::domain_computation::primary_graph::application_query::LifecycleOps<S>>::rebuild_application_managed_index(self, phase, binding, external, plan, index)
    }

    pub fn next_application_managed_index_watch<B, I>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        binding: &WorthQueryInstalledApplicationQueryBinding<S, B>,
        external: &WorthQueryAuthenticatedExternalPrincipal<S>,
        mut plan: WorthQueryAdmittedApplicationQueryPlan< '_, S, <B as ApplicationQueryBinding<S>>::Query, <<B as ApplicationQueryBinding<S>>::ParameterBinding as ApplicationStructuredValueBinding>::Value, <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value, <B as ApplicationQueryBinding<S>>::Principal, <B as ApplicationQueryBinding<S>>::PrincipalIdentity, <<B as ApplicationQueryBinding<S>>::ScopeBinding as ApplicationQueryScopeBinding<S>>::Scope, >,
        watch: &mut WorthQueryManagedIndexWatch<S, I>,
    ) -> Result<WorthQueryManagedIndexWatchOutcome<B::Query, <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value>, WorthQueryManagedIndexDenial>
    where
        B: ApplicationQueryBinding<S>,
        I: ApplicationManagedIndexBinding<S>,
    <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<S, B::Query>,{
        <Self as crate::domain_computation::primary_graph::application_query::NextOps<S>>::next_application_managed_index_watch(self, phase, binding, external, plan, watch)
    }

    pub fn open_application_managed_index_watch<B, I>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        binding: &WorthQueryInstalledApplicationQueryBinding<S, B>,
        external: &WorthQueryAuthenticatedExternalPrincipal<S>,
        mut plan: WorthQueryAdmittedApplicationQueryPlan< '_, S, <B as ApplicationQueryBinding<S>>::Query, <<B as ApplicationQueryBinding<S>>::ParameterBinding as ApplicationStructuredValueBinding>::Value, <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value, <B as ApplicationQueryBinding<S>>::Principal, <B as ApplicationQueryBinding<S>>::PrincipalIdentity, <<B as ApplicationQueryBinding<S>>::ScopeBinding as ApplicationQueryScopeBinding<S>>::Scope, >,
    ) -> Result<
        (
            WorthQueryApplicationOneShotResult<B::Query, <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value>,
            WorthQueryManagedIndexWatch<S, I>,
        ),
        WorthQueryManagedIndexDenial,
    >
    where
        B: ApplicationQueryBinding<S>,
        I: ApplicationManagedIndexBinding<S>,
    <<B as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<S, B::Query>,{
        <Self as crate::domain_computation::primary_graph::application_query::OpenOps<S>>::open_application_managed_index_watch(self, phase, binding, external, plan)
    }
}

impl<Schema: ApplicationSchema> super::super::WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub fn resolve_application_query_limits(
        &self,
        limits: worth_query_installation::facade::WorthQueryInstalledApplicationQueryLimits,
    ) -> worth_query_installation::facade::WorthQueryResolvedApplicationQueryLimits {
        <Self as crate::domain_computation::primary_graph::application_query::WorkLimitOps<
            Schema,
        >>::resolve_application_query_limits(self, limits)
    }

    pub fn plan_application_query_batch_reads<B: ApplicationQueryBinding<Schema>>(
        &self,
        binding: &WorthQueryInstalledApplicationQueryBinding<Schema, B>,
        count: usize,
    ) -> Result<
        WorthQueryApplicationQueryBatchReadPlan,
        WorthQueryApplicationQueryBatchResourceDenial,
    > {
        <Self as crate::domain_computation::primary_graph::application_query::PlanningOps<Schema>>::plan_application_query_batch_reads(self, binding, count)
    }

    pub fn plan_installed_query_batch_reads<Q, P, R, S>(
        &self,
        query: &worth_query_installation::facade::WorthQueryInstalledApplicationQuery<
            Schema,
            Q,
            P,
            R,
            S,
        >,
        count: usize,
        maximum_results: std::num::NonZeroUsize,
    ) -> Result<
        WorthQueryApplicationQueryBatchReadPlan,
        WorthQueryApplicationQueryBatchResourceDenial,
    > {
        <Self as crate::domain_computation::primary_graph::application_query::PlanningOps<Schema>>::plan_installed_query_batch_reads(self, query, count, maximum_results)
    }

    pub fn application_query_batch_from_installed_reads(
        &self,
        reads: &[WorthQueryApplicationQueryBatchReadPlan],
    ) -> Result<
        WorthQueryApplicationQueryBatchAdmission,
        WorthQueryApplicationQueryBatchResourceDenial,
    > {
        <Self as crate::domain_computation::primary_graph::application_query::PlanningOps<Schema>>::application_query_batch_from_installed_reads(self, reads)
    }

    pub fn extend_application_query_batch_once(
        &self,
        batch: &WorthQueryApplicationQueryBatchAdmission,
        reads: &[WorthQueryApplicationQueryBatchReadPlan],
    ) -> Result<(), WorthQueryApplicationQueryBatchResourceDenial> {
        <Self as crate::domain_computation::primary_graph::application_query::PlanningOps<Schema>>::extend_application_query_batch_once(self, batch, reads)
    }

    pub fn execute_application_query_continuation_page<
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        mut plan: WorthQueryAdmittedApplicationQueryPlan<
            '_,
            Schema,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
    ) -> Result<
        WorthQueryApplicationContinuationPageResult<Schema, Query, Parameters, QueryResult, Scope>,
        WorthQueryApplicationContinuationDenial,
    >
    where
        QueryResult: WorthQueryApplicationProjection<Schema, Query>,
    {
        <Self as crate::domain_computation::primary_graph::application_query::ContinuationOps<
            Schema,
        >>::execute_application_query_continuation_page(self, phase, plan)
    }

    pub fn readmit_application_query_continuation<
        'a,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >(
        &'a self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        query: &'a WorthQueryInstalledApplicationQuery<
            Schema,
            Query,
            Parameters,
            QueryResult,
            Scope,
        >,
        access: &WorthQueryApplicationQueryAccessContext<
            'a,
            Schema,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        parameters: ApplicationQueryParameterSet<Query>,
        continuation: WorthQueryApplicationQueryContinuation<
            Schema,
            Query,
            Parameters,
            QueryResult,
            Scope,
        >,
        controls: WorthQueryApplicationQueryResumeControls<'a>,
    ) -> Result<
        WorthQueryAdmittedApplicationQueryPlan<
            'a,
            Schema,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        WorthQueryApplicationQueryAdmissionDenial,
    > {
        <Self as crate::domain_computation::primary_graph::application_query::ReadmissionOps<
            Schema,
        >>::readmit_application_query_continuation(
            self,
            phase,
            query,
            access,
            parameters,
            continuation,
            controls,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn readmit_governed_application_query_continuation<
        'a,
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
        Capability,
        Operation,
        Input,
    >(
        &'a self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        query: &'a WorthQueryInstalledApplicationQuery<
            Schema,
            Query,
            Parameters,
            QueryResult,
            Scope,
        >,
        access: &WorthQueryApplicationQueryAccessContext<
            'a,
            Schema,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        capability: crate::domain_computation::authorization::WorthQueryAdmittedApplicationCapabilityAccess<
            Schema,
            Capability,
            Operation,
            Input,
        >,
        parameters: ApplicationQueryParameterSet<Query>,
        continuation: WorthQueryApplicationQueryContinuation<
            Schema,
            Query,
            Parameters,
            QueryResult,
            Scope,
        >,
        controls: WorthQueryApplicationQueryResumeControls<'a>,
    ) -> Result<
        WorthQueryAdmittedApplicationQueryPlan<
            'a,
            Schema,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        WorthQueryApplicationQueryAdmissionDenial,
    >
    where
        Input: ApplicationCapabilityRequest<Schema, Capability, Scope = Scope>,
    {
        <Self as crate::domain_computation::primary_graph::application_query::ReadmissionOps<
            Schema,
        >>::readmit_governed_application_query_continuation(
            self,
            phase,
            query,
            access,
            capability,
            parameters,
            continuation,
            controls,
        )
    }

    pub fn open_managed_derived_view<Query, Parameters, QueryResult, Scope, Value>(
        &self,
        definition: &ApplicationDerivedViewDefinition<
            Schema,
            Query,
            Parameters,
            QueryResult,
            Scope,
        >,
        query: &WorthQueryInstalledApplicationQuery<Schema, Query, Parameters, QueryResult, Scope>,
        product: &WorthQueryProductBranchLease,
    ) -> Result<WorthQueryManagedDerivedView<Query, Value>, WorthQueryManagedDerivedViewDenial>
    where
        Value: WorthQueryManagedDerivedValue,
    {
        <Self as crate::domain_computation::primary_graph::application_query::RegistrationOps<
            Schema,
        >>::open_managed_derived_view(self, definition, query, product)
    }

    pub fn observe_managed_derived_view<Query, Value>(
        &self,
        view: &WorthQueryManagedDerivedView<Query, Value>,
    ) -> Result<WorthQueryManagedDerivedViewSnapshot<Value>, WorthQueryManagedDerivedViewDenial>
    where
        Value: WorthQueryManagedDerivedValue,
    {
        <Self as crate::domain_computation::primary_graph::application_query::RegistrationOps<
            Schema,
        >>::observe_managed_derived_view(self, view)
    }

    pub fn reconstruct_managed_derived_view<Query, QueryResult, Value>(
        &self,
        view: &WorthQueryManagedDerivedView<Query, Value>,
        product: &WorthQueryProductBranchLease,
        result: &WorthQueryApplicationOneShotResult<Query, QueryResult>,
        mut project: impl FnMut(&QueryResult) -> Value,
    ) -> Result<(), WorthQueryManagedDerivedViewDenial>
    where
        Value: WorthQueryManagedDerivedValue,
    {
        <Self as crate::domain_computation::primary_graph::application_query::RegistrationOps<
            Schema,
        >>::reconstruct_managed_derived_view(self, view, product, result, project)
    }

    pub fn open_managed_derived_collection<
        MembershipQuery,
        MembershipParameters,
        MembershipResult,
        MembershipScope,
        EntryQuery,
        EntryParameters,
        EntryResult,
        EntryScope,
        Value,
    >(
        &self,
        definition: &ApplicationDerivedViewDefinition<
            Schema,
            MembershipQuery,
            MembershipParameters,
            MembershipResult,
            MembershipScope,
        >,
        membership_query: &WorthQueryInstalledApplicationQuery<
            Schema,
            MembershipQuery,
            MembershipParameters,
            MembershipResult,
            MembershipScope,
        >,
        entry_query: &WorthQueryInstalledApplicationQuery<
            Schema,
            EntryQuery,
            EntryParameters,
            EntryResult,
            EntryScope,
        >,
        product: &WorthQueryProductBranchLease,
    ) -> Result<
        WorthQueryManagedDerivedView<MembershipQuery, Value>,
        WorthQueryManagedDerivedViewDenial,
    >
    where
        Value: WorthQueryManagedDerivedValue,
    {
        <Self as crate::domain_computation::primary_graph::application_query::CollectionOps<
            Schema,
        >>::open_managed_derived_collection(
            self,
            definition,
            membership_query,
            entry_query,
            product,
        )
    }

    pub fn refresh_managed_derived_collection_entry<
        'a,
        MembershipQuery,
        EntryQuery,
        EntryParameters,
        EntryResult,
        Principal,
        PrincipalIdentity,
        EntryScope,
        Value,
    >(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        view: &WorthQueryManagedDerivedView<MembershipQuery, Value>,
        product: &WorthQueryProductBranchLease,
        key: &crate::domain_computation::primary_graph::application_query::derived_view::WorthQueryManagedDerivedViewKey,
        plan: WorthQueryAdmittedApplicationQueryPlan<
            'a,
            Schema,
            EntryQuery,
            EntryParameters,
            EntryResult,
            Principal,
            PrincipalIdentity,
            EntryScope,
        >,
        project: impl FnOnce(&EntryResult) -> Value,
    ) -> Result<std::sync::Arc<Value>, WorthQueryManagedDerivedViewDenial>
    where
        EntryResult: WorthQueryApplicationProjection<Schema, EntryQuery>,
        Value: WorthQueryManagedDerivedValue,
    {
        <Self as crate::domain_computation::primary_graph::application_query::CollectionOps<
            Schema,
        >>::refresh_managed_derived_collection_entry(
            self, execution, view, product, key, plan, project,
        )
    }

    pub fn open_managed_derived_collection_pair<
        MembershipQuery,
        MembershipParameters,
        MembershipResult,
        MembershipScope,
        FirstQuery,
        FirstParameters,
        FirstResult,
        FirstScope,
        SecondQuery,
        SecondParameters,
        SecondResult,
        SecondScope,
        Value,
    >(
        &self,
        definition: &ApplicationDerivedViewDefinition<
            Schema,
            MembershipQuery,
            MembershipParameters,
            MembershipResult,
            MembershipScope,
        >,
        membership_query: &WorthQueryInstalledApplicationQuery<
            Schema,
            MembershipQuery,
            MembershipParameters,
            MembershipResult,
            MembershipScope,
        >,
        first_query: &WorthQueryInstalledApplicationQuery<
            Schema,
            FirstQuery,
            FirstParameters,
            FirstResult,
            FirstScope,
        >,
        second_query: &WorthQueryInstalledApplicationQuery<
            Schema,
            SecondQuery,
            SecondParameters,
            SecondResult,
            SecondScope,
        >,
        product: &WorthQueryProductBranchLease,
    ) -> Result<
        WorthQueryManagedDerivedView<MembershipQuery, Value>,
        WorthQueryManagedDerivedViewDenial,
    >
    where
        Value: WorthQueryManagedDerivedValue,
    {
        <Self as crate::domain_computation::primary_graph::application_query::PairOps<Schema>>::open_managed_derived_collection_pair(self, definition, membership_query, first_query, second_query, product)
    }

    pub fn open_owner_sized_managed_derived_collection_pair<
        MembershipQuery,
        MembershipParameters,
        MembershipResult,
        MembershipScope,
        FirstQuery,
        FirstParameters,
        FirstResult,
        FirstScope,
        SecondQuery,
        SecondParameters,
        SecondResult,
        SecondScope,
        Value,
    >(
        &self,
        name: &'static str,
        membership_query: &WorthQueryInstalledApplicationQuery<
            Schema,
            MembershipQuery,
            MembershipParameters,
            MembershipResult,
            MembershipScope,
        >,
        first_query: &WorthQueryInstalledApplicationQuery<
            Schema,
            FirstQuery,
            FirstParameters,
            FirstResult,
            FirstScope,
        >,
        second_query: &WorthQueryInstalledApplicationQuery<
            Schema,
            SecondQuery,
            SecondParameters,
            SecondResult,
            SecondScope,
        >,
        product: &WorthQueryProductBranchLease,
    ) -> Result<
        WorthQueryManagedDerivedView<MembershipQuery, Value>,
        WorthQueryManagedDerivedViewDenial,
    >
    where
        Value: WorthQueryManagedDerivedValue,
    {
        <Self as crate::domain_computation::primary_graph::application_query::PairOps<Schema>>::open_owner_sized_managed_derived_collection_pair(self, name, membership_query, first_query, second_query, product)
    }

    pub fn admit_application_history_capture<'a, History>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        history: &'a WorthQueryInstalledApplicationHistoryQuery<Schema, History>,
        mut plan: crate::domain_computation::primary_graph::application_query::WorthQueryAdmittedApplicationQueryPlan< 'a, Schema, <<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding<Schema>>::Query, <<<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding<Schema>>::ParameterBinding as ApplicationStructuredValueBinding>::Value, <<<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding< Schema, >>::ResultBinding as ApplicationStructuredValueBinding>::Value, <<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding<Schema>>::Principal, <<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding<Schema>>::PrincipalIdentity, <<<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding< Schema, >>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope, >,
        limits: WorthQueryApplicationHistoryCaptureLimits,
    ) -> Result<
        WorthQueryAdmittedApplicationHistoryCapture<'a, Schema, History>,
        WorthQueryApplicationHistoryDenial,
    >
    where
        History: ApplicationHistoryQueryBinding<Schema>,
    {
        <Self as crate::domain_computation::primary_graph::application_query::CapturePlanOps<
            Schema,
        >>::admit_application_history_capture(self, execution, history, plan, limits)
    }

    pub fn admit_governed_application_history_capture<'a, History, Capability, Operation, Input>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        history: &'a WorthQueryInstalledApplicationHistoryQuery<Schema, History>,
        mut plan: crate::domain_computation::primary_graph::application_query::WorthQueryAdmittedApplicationQueryPlan< 'a, Schema, <<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding<Schema>>::Query, <<<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding<Schema>>::ParameterBinding as ApplicationStructuredValueBinding>::Value, <<<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding< Schema, >>::ResultBinding as ApplicationStructuredValueBinding>::Value, <<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding<Schema>>::Principal, <<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding<Schema>>::PrincipalIdentity, <<<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding< Schema, >>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope, >,
        capability: WorthQueryAdmittedApplicationCapabilityAccess<
            Schema,
            Capability,
            Operation,
            Input,
        >,
        limits: WorthQueryApplicationHistoryCaptureLimits,
    ) -> Result<WorthQueryAdmittedApplicationHistoryCapture<'a, Schema, History>, WorthQueryApplicationHistoryDenial>
    where
        History: ApplicationHistoryQueryBinding<Schema>,
        Input:
    ApplicationCapabilityRequest<Schema, Capability, Scope = <<<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding< Schema, >>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope>,{
        <Self as crate::domain_computation::primary_graph::application_query::CapturePlanOps<
            Schema,
        >>::admit_governed_application_history_capture(
            self, execution, history, plan, capability, limits,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn open_application_history<'a, History>(
        &'a self,
        phase: &WorthQueryAdvancementPhase<'_>,
        history: &'a WorthQueryInstalledApplicationHistoryQuery<Schema, History>,
        external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        subjects: &WorthQueryCapturedApplicationHistorySubjects<Schema, History>,
        security: WorthQuerySelectedProductOperation<'a, Schema>,
        truth: &WorthQuerySelectedProductOperation<'a, Schema>,
        request: &'a WorthQueryRequestScope,
        limits: WorthQueryApplicationHistoryPageLimits,
    ) -> Result<
        WorthQueryApplicationHistoryCursor<Schema, History>,
        WorthQueryApplicationHistoryDenial,
    >
    where
        History: ApplicationHistoryQueryBinding<Schema>,
    {
        <Self as crate::domain_computation::primary_graph::application_query::OpeningOps<Schema>>::open_application_history(self, phase, history, external, subjects, security, truth, request, limits)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn open_governed_application_history<
        'a,
        History,
        ReadCapability,
        ReadOperation,
        ReadInput,
        MetadataCapability,
        MetadataOperation,
        MetadataInput,
    >(
        &'a self,
        phase: &WorthQueryAdvancementPhase<'_>,
        history: &'a WorthQueryInstalledApplicationHistoryQuery<Schema, History>,
        external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        subjects: &WorthQueryCapturedApplicationHistorySubjects<Schema, History>,
        security: WorthQuerySelectedProductOperation<'a, Schema>,
        truth: &WorthQuerySelectedProductOperation<'a, Schema>,
        read: WorthQueryAdmittedApplicationCapabilityAccess<Schema, ReadCapability, ReadOperation, ReadInput>,
        metadata: WorthQueryAdmittedApplicationCapabilityAccess<Schema, MetadataCapability, MetadataOperation, MetadataInput>,
        request: &'a WorthQueryRequestScope,
        limits: WorthQueryApplicationHistoryPageLimits,
    ) -> Result<WorthQueryApplicationHistoryCursor<Schema, History>, WorthQueryApplicationHistoryDenial>
    where
        History: ApplicationHistoryQueryBinding<Schema>,
        ReadInput: ApplicationCapabilityRequest<
            Schema,
            ReadCapability,
            Scope = <<<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding< Schema, >>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope,
        >,
        MetadataInput: ApplicationCapabilityRequest<
            Schema,
            MetadataCapability,
            Scope = <<<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding< Schema, >>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope,
    >,{
        <Self as crate::domain_computation::primary_graph::application_query::OpeningOps<Schema>>::open_governed_application_history(self, phase, history, external, subjects, security, truth, read, metadata, request, limits)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn open_application_history_with_governed_subject<
        'a,
        History,
        Capability,
        Operation,
        Input,
    >(
        &'a self,
        phase: &WorthQueryAdvancementPhase<'_>,
        history: &'a WorthQueryInstalledApplicationHistoryQuery<Schema, History>,
        external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        subjects: &WorthQueryCapturedApplicationHistorySubjects<Schema, History>,
        security: WorthQuerySelectedProductOperation<'a, Schema>,
        truth: &WorthQuerySelectedProductOperation<'a, Schema>,
        read: WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
        request: &'a WorthQueryRequestScope,
        limits: WorthQueryApplicationHistoryPageLimits,
    ) -> Result<WorthQueryApplicationHistoryCursor<Schema, History>, WorthQueryApplicationHistoryDenial>
    where
        History: ApplicationHistoryQueryBinding<Schema>,
        Input:
    ApplicationCapabilityRequest<Schema, Capability, Scope = <<<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding< Schema, >>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope>,{
        <Self as crate::domain_computation::primary_graph::application_query::OpeningOps<Schema>>::open_application_history_with_governed_subject(self, phase, history, external, subjects, security, truth, read, request, limits)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn open_application_history_with_governed_metadata<
        'a,
        History,
        Capability,
        Operation,
        Input,
    >(
        &'a self,
        phase: &WorthQueryAdvancementPhase<'_>,
        history: &'a WorthQueryInstalledApplicationHistoryQuery<Schema, History>,
        external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        subjects: &WorthQueryCapturedApplicationHistorySubjects<Schema, History>,
        security: WorthQuerySelectedProductOperation<'a, Schema>,
        truth: &WorthQuerySelectedProductOperation<'a, Schema>,
        metadata: WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
        request: &'a WorthQueryRequestScope,
        limits: WorthQueryApplicationHistoryPageLimits,
    ) -> Result<WorthQueryApplicationHistoryCursor<Schema, History>, WorthQueryApplicationHistoryDenial>
    where
        History: ApplicationHistoryQueryBinding<Schema>,
        Input:
    ApplicationCapabilityRequest<Schema, Capability, Scope = <<<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding< Schema, >>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope>,{
        <Self as crate::domain_computation::primary_graph::application_query::OpeningOps<Schema>>::open_application_history_with_governed_metadata(self, phase, history, external, subjects, security, truth, metadata, request, limits)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn read_application_history<'a, History>(
        &'a self,
        phase: &WorthQueryAdvancementPhase<'_>,
        history: &'a WorthQueryInstalledApplicationHistoryQuery<Schema, History>,
        external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        subjects: &WorthQueryCapturedApplicationHistorySubjects<Schema, History>,
        cursor: &mut WorthQueryApplicationHistoryCursor<Schema, History>,
        security: WorthQuerySelectedProductOperation<'a, Schema>,
        truth: &WorthQuerySelectedProductOperation<'a, Schema>,
        request: &'a WorthQueryRequestScope,
        limits: WorthQueryApplicationHistoryPageLimits,
    ) -> Result<WorthQueryApplicationHistoryPage, WorthQueryApplicationHistoryDenial>
    where
        History: ApplicationHistoryQueryBinding<Schema>,
    {
        <Self as crate::domain_computation::primary_graph::application_query::FreshPagingOps<
            Schema,
        >>::read_application_history(
            self, phase, history, external, subjects, cursor, security, truth, request, limits,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn read_governed_application_history<
        'a,
        History,
        ReadCapability,
        ReadOperation,
        ReadInput,
        MetadataCapability,
        MetadataOperation,
        MetadataInput,
    >(
        &'a self,
        phase: &WorthQueryAdvancementPhase<'_>,
        history: &'a WorthQueryInstalledApplicationHistoryQuery<Schema, History>,
        external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        subjects: &WorthQueryCapturedApplicationHistorySubjects<Schema, History>,
        cursor: &mut WorthQueryApplicationHistoryCursor<Schema, History>,
        security: WorthQuerySelectedProductOperation<'a, Schema>,
        truth: &WorthQuerySelectedProductOperation<'a, Schema>,
        read: WorthQueryAdmittedApplicationCapabilityAccess<Schema, ReadCapability, ReadOperation, ReadInput>,
        metadata: WorthQueryAdmittedApplicationCapabilityAccess<Schema, MetadataCapability, MetadataOperation, MetadataInput>,
        request: &'a WorthQueryRequestScope,
        limits: WorthQueryApplicationHistoryPageLimits,
    ) -> Result<WorthQueryApplicationHistoryPage, WorthQueryApplicationHistoryDenial>
    where
        History: ApplicationHistoryQueryBinding<Schema>,
        ReadInput: ApplicationCapabilityRequest<
            Schema,
            ReadCapability,
            Scope = <<<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding< Schema, >>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope,
        >,
        MetadataInput: ApplicationCapabilityRequest<
            Schema,
            MetadataCapability,
            Scope = <<<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding< Schema, >>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope,
    >,{
        <Self as crate::domain_computation::primary_graph::application_query::FreshPagingOps<
            Schema,
        >>::read_governed_application_history(
            self, phase, history, external, subjects, cursor, security, truth, read, metadata,
            request, limits,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn read_application_history_with_governed_subject<
        'a,
        History,
        Capability,
        Operation,
        Input,
    >(
        &'a self,
        phase: &WorthQueryAdvancementPhase<'_>,
        history: &'a WorthQueryInstalledApplicationHistoryQuery<Schema, History>,
        external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        subjects: &WorthQueryCapturedApplicationHistorySubjects<Schema, History>,
        cursor: &mut WorthQueryApplicationHistoryCursor<Schema, History>,
        security: WorthQuerySelectedProductOperation<'a, Schema>,
        truth: &WorthQuerySelectedProductOperation<'a, Schema>,
        read: WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
        request: &'a WorthQueryRequestScope,
        limits: WorthQueryApplicationHistoryPageLimits,
    ) -> Result<WorthQueryApplicationHistoryPage, WorthQueryApplicationHistoryDenial>
    where
        History: ApplicationHistoryQueryBinding<Schema>,
        Input:
    ApplicationCapabilityRequest<Schema, Capability, Scope = <<<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding< Schema, >>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope>,{
        <Self as crate::domain_computation::primary_graph::application_query::FreshPagingOps<
            Schema,
        >>::read_application_history_with_governed_subject(
            self, phase, history, external, subjects, cursor, security, truth, read, request,
            limits,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn read_application_history_with_governed_metadata<
        'a,
        History,
        Capability,
        Operation,
        Input,
    >(
        &'a self,
        phase: &WorthQueryAdvancementPhase<'_>,
        history: &'a WorthQueryInstalledApplicationHistoryQuery<Schema, History>,
        external: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        subjects: &WorthQueryCapturedApplicationHistorySubjects<Schema, History>,
        cursor: &mut WorthQueryApplicationHistoryCursor<Schema, History>,
        security: WorthQuerySelectedProductOperation<'a, Schema>,
        truth: &WorthQuerySelectedProductOperation<'a, Schema>,
        metadata: WorthQueryAdmittedApplicationCapabilityAccess<Schema, Capability, Operation, Input>,
        request: &'a WorthQueryRequestScope,
        limits: WorthQueryApplicationHistoryPageLimits,
    ) -> Result<WorthQueryApplicationHistoryPage, WorthQueryApplicationHistoryDenial>
    where
        History: ApplicationHistoryQueryBinding<Schema>,
        Input:
    ApplicationCapabilityRequest<Schema, Capability, Scope = <<<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding< Schema, >>::ScopeBinding as ApplicationQueryScopeBinding<Schema>>::Scope>,{
        <Self as crate::domain_computation::primary_graph::application_query::FreshPagingOps<
            Schema,
        >>::read_application_history_with_governed_metadata(
            self, phase, history, external, subjects, cursor, security, truth, metadata, request,
            limits,
        )
    }

    pub fn bind_application_source_expectation<Binding, Scope>(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        admission: &mut WorthQueryAdmittedApplicationOperation<
            Schema,
            Binding::Operation,
            Binding::Input,
            Scope,
        >,
        source: WorthQueryObservedSource<
            <Binding::SourceExpectation as worth_query_declaration::facade::application_operation::ApplicationMutationSourceExpectation<Schema>>::Query,
        >,
        input: &Binding::Input,
    ) -> Result<WorthQueryPendingSourceExpectation<<Binding::SourceExpectation as worth_query_declaration::facade::application_operation::ApplicationMutationSourceExpectation<Schema>>::Query>, WorthQuerySourceExpectationDenial>
    where
        Binding: worth_query_declaration::facade::application_operation::ApplicationMutationBinding<
            Schema,
    >,{
        <Self as crate::domain_computation::primary_graph::application_query::PreparedExpectationOps<Schema>>::bind_application_source_expectation::<Binding, Scope>(self, phase, admission, source, input)
    }

    pub fn bind_application_result_set_expectation<Binding, Scope>(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        admission: &mut WorthQueryAdmittedApplicationOperation<
            Schema,
            Binding::Operation,
            Binding::Input,
            Scope,
        >,
        result_set: WorthQueryObservedResultSet<
            <Binding::SourceExpectation as worth_query_declaration::facade::application_operation::ApplicationMutationSourceExpectation<Schema>>::Query,
        >,
        input: &Binding::Input,
    ) -> Result<WorthQueryPendingSourceExpectation<<Binding::SourceExpectation as worth_query_declaration::facade::application_operation::ApplicationMutationSourceExpectation<Schema>>::Query>, WorthQuerySourceExpectationDenial>
    where
        Binding: worth_query_declaration::facade::application_operation::ApplicationMutationBinding<
            Schema,
    >,{
        <Self as crate::domain_computation::primary_graph::application_query::PreparedExpectationOps<Schema>>::bind_application_result_set_expectation::<Binding, Scope>(self, phase, admission, result_set, input)
    }

    pub fn execute_application_query_one_shot<
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        plan: WorthQueryAdmittedApplicationQueryPlan<
            '_,
            Schema,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
    ) -> Result<
        WorthQueryApplicationOneShotResult<Query, QueryResult>,
        WorthQueryApplicationOneShotDenial,
    >
    where
        QueryResult: WorthQueryApplicationProjection<Schema, Query>,
    {
        <Self as crate::domain_computation::primary_graph::application_query::OneShotOps<Schema>>::execute_application_query_one_shot(self, phase, plan)
    }

    pub fn execute_application_query_one_shot_in_batch<
        Query,
        Parameters,
        QueryResult,
        Principal,
        PrincipalIdentity,
        Scope,
    >(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        plan: WorthQueryAdmittedApplicationQueryPlan<
            '_,
            Schema,
            Query,
            Parameters,
            QueryResult,
            Principal,
            PrincipalIdentity,
            Scope,
        >,
        batch: &WorthQueryApplicationQueryBatchAdmission,
    ) -> Result<
        WorthQueryApplicationBatchResult<Query, QueryResult>,
        WorthQueryApplicationBatchReadDenial,
    >
    where
        QueryResult: WorthQueryApplicationProjection<Schema, Query>,
    {
        <Self as crate::domain_computation::primary_graph::application_query::BatchOps<Schema>>::execute_application_query_one_shot_in_batch(self, phase, plan, batch)
    }

    pub fn execute_application_history_capture<History>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        capture: WorthQueryAdmittedApplicationHistoryCapture<'_, Schema, History>,
    ) -> Result<WorthQueryApplicationHistoryCaptureResult<Schema, History>, WorthQueryApplicationHistoryDenial>
    where
        History: ApplicationHistoryQueryBinding<Schema>,
        <<<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value:
    WorthQueryApplicationProjection<Schema, <<History as ApplicationHistoryQueryBinding<Schema>>::SubjectRead as ApplicationQueryBinding<Schema>>::Query>,{
        <Self as crate::domain_computation::primary_graph::application_query::HistoryCaptureOps<
            Schema,
        >>::execute_application_history_capture(self, phase, capture)
    }

    pub fn application_basis_token(
        &self,
        basis: &WorthQueryApplicationBasisIdentity,
    ) -> Result<String, WorthQueryApplicationQueryAdmissionDenial> {
        <Self as crate::domain_computation::primary_graph::application_query::ReadinessOps<
            Schema,
        >>::application_basis_token(self, basis)
    }

    pub fn application_commit_basis_token(
        &self,
        receipt: &WorthQueryApplicationCommitReceipt,
    ) -> Result<String, WorthQueryApplicationQueryAdmissionDenial> {
        <Self as crate::domain_computation::primary_graph::application_query::ReadinessOps<
            Schema,
        >>::application_commit_basis_token(self, receipt)
    }

    pub fn select_application_read_observation(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        observation: &WorthQueryApplicationReadObservation,
    ) -> Result<
        WorthQuerySelectedProductOperation<'_, Schema>,
        WorthQueryProductBranchAdmissionDenial,
    > {
        <Self as crate::domain_computation::primary_graph::application_query::RetainedReadOps<
            Schema,
        >>::select_application_read_observation(self, phase, observation)
    }
}
