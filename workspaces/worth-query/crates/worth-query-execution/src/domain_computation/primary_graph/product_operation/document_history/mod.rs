//! The attached document owns its actual head and exact model association.
pub(in crate::domain_computation::primary_graph) use worth_query_execution_query::facade::domain_computation::primary_graph::product_operation::document_history::allocation;
pub(in crate::domain_computation::primary_graph) use worth_query_execution_query::facade::domain_computation::primary_graph::product_operation::document_history::authority_lifetime;
mod candidate;
mod capture_pause;
mod close;
mod constraint;
mod context;
mod context_pause;
pub(in crate::domain_computation::primary_graph) use worth_query_execution_query::facade::domain_computation::primary_graph::product_operation::document_history::current_head;
pub use context::{WorthQueryHistoryContextRefreshDenial, WorthQueryHistoryContextRefreshLimits};
#[doc(hidden)]
pub use context_pause::WorthQueryHistoryContextRefreshPauseForTest;
mod denial;
mod description;
pub(in crate::domain_computation::primary_graph) use worth_query_execution_query::facade::domain_computation::primary_graph::product_operation::document_history::discovery_recipes;
#[path = "discovery_recipes/enrollment.rs"]
mod discovery_recipes_enrollment;

pub(in crate::domain_computation::primary_graph) fn history_discovery_recipe_key<C: 'static>(
    profile: &WorthQueryHistoryOutputProfile,
    parent: (
        crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
        [u8; 32],
    ),
) -> discovery_recipes::HistoryDiscoveryRecipeKey {
    discovery_recipes::HistoryDiscoveryRecipeKey::from_parts::<C>(
        profile.discovery_recipe_tree_identity(),
        profile.source_intent().source_subject(),
        profile.source_intent().source_partition(),
        parent,
    )
}
pub use description::{
    WorthQueryDocumentHistoryIdentity, WorthQueryHistoryCommandContextDenial,
    WorthQueryHistoryDescriptionContext,
};
mod disposal;
mod envelope_recording;
pub use envelope_recording::{
    WorthQueryDocumentHistoryCorrectionMeaning, WorthQueryDocumentHistoryEnvelopeConstraint,
    WorthQueryDocumentHistoryEnvelopeIntent,
};
mod group;
mod head_custody;
pub use group::{
    WorthQueryDocumentHistoryGroupAborting, WorthQueryDocumentHistoryGroupAdoptionFailure,
    WorthQueryDocumentHistoryGroupPreparation, WorthQueryDocumentHistoryGroupReservation,
    WorthQueryHistoryGroupCandidateCreation, WorthQueryHistoryGroupMemberAdmission,
};
mod journal;
pub(crate) use worth_query_execution_query::facade::domain_computation::primary_graph::product_operation::document_history::model;
pub(crate) mod navigation;
pub(in crate::domain_computation::primary_graph) use navigation::HistoryDependentSource;
#[doc(hidden)]
pub use navigation::WorthQueryHistoryNavigationPauseForTest;
pub(in crate::domain_computation::primary_graph) use navigation::{
    HistoryOutputLookup, HistoryOutputLookupPhase,
};
pub use navigation::{
    WorthQueryAdmittedHistoryProgramOutput, WorthQueryCompletedHistoryOutputs,
    WorthQueryDocumentHistoryNavigation, WorthQueryHistoryDependentOutputRead,
    WorthQueryHistoryNavigationCompletion, WorthQueryHistoryNavigationDenial,
    WorthQueryHistoryNavigationEligibilityDenial, WorthQueryHistoryNavigationIntent,
    WorthQueryHistoryNavigationPosture, WorthQueryHistoryOutputAdvanceDenial,
    WorthQueryHistoryOutputAuthority, WorthQueryHistoryOutputDiscovery,
    WorthQueryHistoryOutputNavigationAdvance, WorthQueryHistoryOutputProfile,
    WorthQueryHistoryOutputQualificationDenial, WorthQueryHistoryOutputSource,
    WorthQueryHistoryProgramOutputAdvance, WorthQueryHistoryRootOutputAdmissionDenial,
    WorthQueryHistorySelection, WorthQueryRetainedHistoryOutputTree,
    WorthQuerySettledHistoryProgramOutput, WorthQueryVerifiedHistoryParent,
};
mod observation;
mod operation_context;
mod operation_scope;
pub(in crate::domain_computation::primary_graph) use operation_scope::HistoryOperationScope;
mod page_pause;
mod paging;
pub(in crate::domain_computation::primary_graph) use worth_query_execution_query::facade::domain_computation::primary_graph::product_operation::document_history::preview;
mod receiver_pause;
mod retention_pause;
#[doc(hidden)]
pub use retention_pause::WorthQueryHistoryRetentionChangePauseForTest;
mod redo;
mod retention;
pub use redo::{WorthQueryHistoryRedoChoice, WorthQueryHistoryRedoChoices};
pub(in crate::domain_computation::primary_graph) use retention::HistoryRetentionChange;
pub use retention::{WorthQueryHistoryReleaseOutcome, WorthQueryHistoryRetentionStatus};
mod recording;
mod recording_outputs;
mod reservation;
mod reversal_facts;
mod sealing;
pub use recording_outputs::{
    WorthQueryCompletedRecordingOutputs, WorthQueryHistoryRecordingOutputOrigin,
    WorthQueryRecordingOutputQualificationDenial, WorthQueryRetainedRecordingOutputTree,
};
mod selection;
mod source_evidence;
pub use source_evidence::WorthQueryHistorySourceEvidence;
pub(in crate::domain_computation::primary_graph) use worth_query_execution_query::facade::domain_computation::primary_graph::product_operation::document_history::step;
pub use candidate::WorthQueryDocumentHistoryCandidateDenial;
#[doc(hidden)]
pub use capture_pause::WorthQueryDocumentHistoryCapturePauseForTest;
pub use constraint::WorthQueryDocumentHistoryOperationConstraint;
pub use operation_context::WorthQueryDocumentHistoryOperation;
pub use reservation::{
    WorthQueryDocumentHistoryRecordPreparation, WorthQueryDocumentHistoryReservation,
};
pub use sealing::WorthQueryDocumentHistoryAdoptionFailure;

pub use close::WorthQueryDocumentHistoryCloseFailure;
pub(in crate::domain_computation::primary_graph) use current_head::confirm as confirm_document_head;
pub use denial::{WorthQueryDocumentHistoryAttachmentFailure, WorthQueryDocumentHistoryDenial};
pub use disposal::{
    WorthQueryDocumentHistoryCloseReceipt, WorthQueryDocumentHistoryClosing,
    WorthQueryDocumentHistoryClosingDenial,
};
pub(in crate::domain_computation::primary_graph) use model::HistoryModelIdentity;
#[doc(hidden)]
pub use recording::history_envelope_identities_for_test;
#[doc(hidden)]
pub use observation::{
    WorthQueryHeldNavigationLineageForTest, WorthQueryHistoryEnrollmentObservationForTest,
    WorthQueryHistoryEnrollmentReportForTest, WorthQueryHistorySourceIndexForTest,
};
#[doc(hidden)]
pub use page_pause::WorthQueryHistoryPageAssemblyPauseForTest;
#[doc(hidden)]
pub use reversal_facts::WorthQueryHistoryReversalFactForTest;
pub use paging::{
    WorthQueryHistoryPage, WorthQueryHistoryPageCursor, WorthQueryHistoryPageDenial,
    WorthQueryHistoryPageEntry, WorthQueryHistoryPageLimits, WorthQueryHistoryPageRow,
    WorthQueryHistoryParentBoundary,
};
pub use preview::WorthQueryHistoryPreview;
pub(in crate::domain_computation::primary_graph) use selection::{
    HistorySelectionCell, SharedHistorySelectionCell, WeakHistorySelectionCell,
};
pub use step::{WorthQueryHistoryNodeId, WorthQueryHistoryStep};
pub use worth_query_execution_query::facade::domain_computation::primary_graph::product_operation::document_history::WorthQueryHistoryStepMetadata;

use super::WorthQueryOwnedProductHead;
use worth_query_declaration::facade::application_history::ApplicationHistoryBinding;
use worth_query_installation::facade::{ApplicationSchema, ApplicationSchemaBindingIdentity};

/// Execution's model/head owner. The ordinary frontend freshly admits every read.
#[must_use = "retain the document's head custody or close it explicitly"]
pub struct WorthQueryDocumentHistory<
    'runtime,
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
> {
    pub(in crate::domain_computation::primary_graph) application:
        crate::domain_computation::primary_graph::product_operation::WorthQueryRuntimeCustody<
            'runtime,
            S,
        >,
    pub(in crate::domain_computation::primary_graph) selection: SharedHistorySelectionCell,
    pub(in crate::domain_computation::primary_graph) model: HistoryModelIdentity,
    pub(in crate::domain_computation::primary_graph) binding: ApplicationSchemaBindingIdentity,
    pub(in crate::domain_computation::primary_graph) capture_pause:
        Option<std::sync::Arc<receiver_pause::ReceiverPause>>,
    pub(in crate::domain_computation::primary_graph) page_pause:
        Option<std::sync::Arc<receiver_pause::ReceiverPause>>,
    pub(in crate::domain_computation::primary_graph) context_pause:
        Option<std::sync::Arc<receiver_pause::ReceiverPause>>,
    pub(in crate::domain_computation::primary_graph) retention_pause: Option<(
        retention_pause::RetentionChangePoint,
        std::sync::Arc<receiver_pause::ReceiverPause>,
    )>,
    pub(in crate::domain_computation::primary_graph) marker: std::marker::PhantomData<fn() -> H>,
}

impl<'runtime, S: ApplicationSchema, H: ApplicationHistoryBinding<S>>
    WorthQueryDocumentHistory<'runtime, S, H>
{
    pub fn application(
        &self,
    ) -> &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<S>
    {
        &self.application
    }

    /// A separate hold on the runtime, for an owner created from this document.
    pub(in crate::domain_computation::primary_graph) fn custody(
        &self,
    ) -> crate::domain_computation::primary_graph::product_operation::WorthQueryRuntimeCustody<
        'runtime,
        S,
    > {
        self.application.clone()
    }

    /// Descriptive targeting for the Publication owner; it grants no READ permission.
    pub fn branch(&self) -> crate::basis::WorthQueryProductBranch {
        self.selection.lock().head_branch()
    }
}

impl<S: ApplicationSchema, H: ApplicationHistoryBinding<S>> std::fmt::Debug
    for WorthQueryDocumentHistory<'_, S, H>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryDocumentHistory")
            .finish_non_exhaustive()
    }
}
#[cfg(test)]
#[path = "mod_inline_type_budget.rs"]
pub(crate) mod inline_type_budget;

pub(in crate::domain_computation) use navigation::HistoryProducerInvocation;
