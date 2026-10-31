//! The attached document owns its actual head and exact model association.
mod allocation;
mod candidate;
mod capture_pause;
mod close;
mod constraint;
mod current_head;
mod denial;
mod disposal;
mod journal;
mod model;
mod operation_context;
mod reservation;
mod sealing;
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
pub(in crate::domain_computation::primary_graph) use journal::HistoryJournal;
pub(in crate::domain_computation::primary_graph) use model::HistoryModelIdentity;
pub use step::{WorthQueryHistoryNodeId, WorthQueryHistoryStep};

use super::WorthQueryOwnedProductHead;
use crate::basis::WorthQueryProductBranchReadIdentity;
use worth_query_declaration::facade::application_history::ApplicationHistoryBinding;
use worth_query_installation::facade::{ApplicationSchema, ApplicationSchemaBindingIdentity};

/// Execution's model/head owner. The ordinary frontend freshly admits every read.
#[must_use = "retain the document's head custody or close it explicitly"]
pub struct WorthQueryDocumentHistory<
    'runtime,
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
> {
    pub(in crate::domain_computation::primary_graph) head: WorthQueryOwnedProductHead<'runtime, S>,
    pub(in crate::domain_computation::primary_graph) model: HistoryModelIdentity,
    pub(in crate::domain_computation::primary_graph) expected: WorthQueryProductBranchReadIdentity,
    pub(in crate::domain_computation::primary_graph) binding: ApplicationSchemaBindingIdentity,
    pub(in crate::domain_computation::primary_graph) journal: HistoryJournal<'runtime, S>,
    pub(in crate::domain_computation::primary_graph) capture_pause:
        Option<std::sync::Arc<capture_pause::CapturePause>>,
    pub(in crate::domain_computation::primary_graph) marker: std::marker::PhantomData<fn() -> H>,
}

impl<'runtime, S: ApplicationSchema, H: ApplicationHistoryBinding<S>>
    WorthQueryDocumentHistory<'runtime, S, H>
{
    pub fn application(&self) -> &'runtime crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<S>{
        self.head.application()
    }

    /// Descriptive targeting for the Publication owner; it grants no READ permission.
    pub fn branch(&self) -> crate::basis::WorthQueryProductBranch {
        self.head.branch()
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
