//! One freshly admitted Native callback, lent through traversal and never stored.
use worth_query_execution::{
    facade::product::WorthQueryProductBranch,
    publication_boundary::document_history::WorthQueryHistoryOutputAuthority,
};

#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct ProgramOutputHistoryFrame<'call, 'phase, 'scope> {
    pub(in crate::application_entry) authority:
        &'call WorthQueryHistoryOutputAuthority<'phase, 'scope>,
    pub(in crate::application_entry) branch: WorthQueryProductBranch,
}

pub(super) fn history_mismatch(
) -> crate::application_entry::WorthQueryRequiredOutputPreparationDenial {
    crate::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand(
        crate::application_entry::WorthQueryApplicationOutputDemandDenial::FreshRequestMismatch,
    )
}

pub(super) fn history_read_denial(
    cause: worth_query_execution::publication_boundary::document_history::WorthQueryHistoryOutputReadDenial,
) -> crate::application_entry::WorthQueryRequiredOutputPreparationDenial {
    crate::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand(
        crate::application_entry::WorthQueryApplicationOutputDemandDenial::HistoryRead(cause),
    )
}
