//! The attached document owns its actual head and exact model association.
mod close;
mod denial;
mod model;

pub use close::WorthQueryDocumentHistoryCloseFailure;
pub use denial::{WorthQueryDocumentHistoryAttachmentFailure, WorthQueryDocumentHistoryDenial};
pub(in crate::domain_computation::primary_graph) use model::HistoryModelIdentity;

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
