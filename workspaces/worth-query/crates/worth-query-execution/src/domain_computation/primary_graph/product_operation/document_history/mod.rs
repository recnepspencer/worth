//! The attached document owns its actual head and exact model association.
mod allocation;
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
mod disposal;
mod journal;
mod model;
mod observation;
mod operation_context;
mod page_pause;
mod paging;
mod preview;
mod receiver_pause;
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
    WorthQueryHistoryPageLimits, WorthQueryHistoryPageRow,
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
