//! The attached document owns its actual head and exact model association.
mod allocation;
pub(in crate::domain_computation::primary_graph) mod authority_lifetime;
mod candidate;
mod capture_pause;
mod close;
mod constraint;
mod context;
mod context_pause;
mod current_head;
pub use context::{WorthQueryHistoryContextRefreshDenial, WorthQueryHistoryContextRefreshLimits};
#[doc(hidden)]
pub use context_pause::WorthQueryHistoryContextRefreshPauseForTest;
mod denial;
mod description;
pub use description::{
    WorthQueryDocumentHistoryIdentity, WorthQueryHistoryCommandContextDenial,
    WorthQueryHistoryDescriptionContext,
};
mod disposal;
mod group;
mod head_custody;
pub use group::{
    WorthQueryDocumentHistoryGroupAborting, WorthQueryDocumentHistoryGroupAdoptionFailure,
    WorthQueryDocumentHistoryGroupPreparation, WorthQueryDocumentHistoryGroupReservation,
    WorthQueryHistoryGroupCandidateCreation, WorthQueryHistoryGroupMemberAdmission,
};
mod journal;
pub(crate) mod model;
pub(crate) mod navigation;
#[doc(hidden)]
pub use navigation::WorthQueryHistoryNavigationPauseForTest;
pub(in crate::domain_computation::primary_graph) use navigation::{
    HistoryBorrowedReadStop, HistoryDependentSource, HistoryProducerCutover,
};
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
mod preview;
mod receiver_pause;
mod retention_pause;
#[doc(hidden)]
pub use retention_pause::WorthQueryHistoryRetentionChangePauseForTest;
mod redo;
mod retention;
pub use redo::{WorthQueryHistoryRedoChoice, WorthQueryHistoryRedoChoices};
pub(in crate::domain_computation::primary_graph) use retention::HistoryRetentionChange;
pub use retention::{WorthQueryHistoryReleaseOutcome, WorthQueryHistoryRetentionStatus};
mod reservation;
mod sealing;
mod selection;
mod step;
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
pub use observation::{
    WorthQueryHistoryEnrollmentObservationForTest, WorthQueryHistoryEnrollmentReportForTest,
};
#[doc(hidden)]
pub use page_pause::WorthQueryHistoryPageAssemblyPauseForTest;
pub use paging::{
    WorthQueryHistoryPage, WorthQueryHistoryPageCursor, WorthQueryHistoryPageDenial,
    WorthQueryHistoryPageEntry, WorthQueryHistoryPageLimits, WorthQueryHistoryPageRow,
    WorthQueryHistoryParentBoundary,
};
pub use preview::WorthQueryHistoryPreview;
pub(in crate::domain_computation::primary_graph) use selection::HistorySelectionCell;
pub use step::{WorthQueryHistoryNodeId, WorthQueryHistoryStep};

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
    pub(in crate::domain_computation::primary_graph) application: &'runtime crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<S>,
    pub(in crate::domain_computation::primary_graph) selection:
        std::sync::Arc<HistorySelectionCell<'runtime, S>>,
    pub(in crate::domain_computation::primary_graph) model: HistoryModelIdentity,
    pub(in crate::domain_computation::primary_graph) binding: ApplicationSchemaBindingIdentity,
    pub(in crate::domain_computation::primary_graph) capture_pause:
        Option<std::sync::Arc<receiver_pause::ReceiverPause>>,
    pub(in crate::domain_computation::primary_graph) page_pause:
        Option<std::sync::Arc<receiver_pause::ReceiverPause>>,
    pub(in crate::domain_computation::primary_graph) context_pause:
        Option<std::sync::Arc<receiver_pause::ReceiverPause>>,
    pub(in crate::domain_computation::primary_graph) retention_pause:
        Option<(retention_pause::RetentionChangePoint, std::sync::Arc<receiver_pause::ReceiverPause>)>,
    pub(in crate::domain_computation::primary_graph) marker: std::marker::PhantomData<fn() -> H>,
}

impl<'runtime, S: ApplicationSchema, H: ApplicationHistoryBinding<S>>
    WorthQueryDocumentHistory<'runtime, S, H>
{
    pub fn application(&self) -> &'runtime crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<S>{
        self.application
    }

    /// Descriptive targeting for the Publication owner; it grants no READ permission.
    pub fn branch(&self) -> crate::basis::WorthQueryProductBranch {
        self.selection.lock().head.branch()
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

pub(in crate::domain_computation) use navigation::HistoryProducerInvocation;
