//! A historical holder has no conversion to a selected or writable operation.
//! ```compile_fail,E0599
//! use worth_query_publication::facade::application_entry::WorthQueryApplicationHistoryPreview;
//! use worth_query_declaration::facade::{application_history::ApplicationHistoryBinding, application_schema::ApplicationSchema};
//! fn select<S: ApplicationSchema, H: ApplicationHistoryBinding<S>>(preview: WorthQueryApplicationHistoryPreview<S, H>) {
//!     preview.select_on();
//! }
//! ```
//! Keep or close the issued holder instead.
//! ```
//! use worth_query_publication::facade::application_entry::WorthQueryApplicationHistoryPreview;
//! use worth_query_declaration::facade::{application_history::ApplicationHistoryBinding, application_schema::ApplicationSchema};
//! fn close<S: ApplicationSchema, H: ApplicationHistoryBinding<S>>(preview: WorthQueryApplicationHistoryPreview<S, H>) {
//!     let _step = preview.step();
//!     preview.close();
//! }
//! ```
use super::*;
use worth_query_execution::publication_boundary::document_history::WorthQueryHistoryPreview as NativePreview;

mod open;
mod read;
mod result;
pub use result::WorthQueryHistoryPreviewResult;

#[must_use = "retain the historical holder or close it"]
pub struct WorthQueryApplicationHistoryPreview<
    S: ApplicationSchema,
    H: ApplicationHistoryBinding<S>,
> {
    owner: NativePreview,
    marker: std::marker::PhantomData<fn() -> (S, H)>,
}
impl<S: ApplicationSchema, H: ApplicationHistoryBinding<S>>
    WorthQueryApplicationHistoryPreview<S, H>
{
    pub fn step(&self) -> &WorthQueryHistoryStep {
        self.owner.step()
    }
    pub fn close(self) {
        self.owner.close()
    }
}
impl<S: ApplicationSchema, H: ApplicationHistoryBinding<S>> std::fmt::Debug
    for WorthQueryApplicationHistoryPreview<S, H>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryApplicationHistoryPreview")
            .finish_non_exhaustive()
    }
}
