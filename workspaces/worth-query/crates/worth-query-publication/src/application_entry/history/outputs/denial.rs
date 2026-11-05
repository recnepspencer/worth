//! Refusal descriptions leave actual cleanup owners in their document slot.
#[derive(Debug)]
pub enum WorthQueryHistoryOutputDenial {
    Busy,
    ForeignSource,
    CapacityOverflow,
    NeedsDisposal,
    Query(crate::application_entry::WorthQueryApplicationRequestQueryDenial),
    Navigation(worth_query_execution::publication_boundary::document_history::WorthQueryHistoryNavigationDenial),
    Source(worth_query_execution::publication_boundary::document_history::WorthQueryHistoryOutputReadDenial),
    Root(worth_query_execution::publication_boundary::document_history::WorthQueryHistoryRootOutputAdmissionDenial),
    Preparation(crate::application_entry::WorthQueryRequiredOutputPreparationDenial),
    Qualification(worth_query_execution::publication_boundary::original_read_outputs::WorthQueryOriginalReadOutputDenial),
    Close(worth_query_execution::facade::primary_graph::WorthQueryOutputReadCloseDenial),
    GraphQualification(worth_query_execution::publication_boundary::document_history::WorthQueryHistoryOutputQualificationDenial),
    ManagedCleanup,
    Execution(worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial),
    Memory(worth_execution::LeaseDenial),
    Work(worth_execution::WorkCeilingDenial),
}

impl From<worth_query_execution::publication_boundary::document_history::WorthQueryHistoryOutputNavigationDenial>
    for WorthQueryHistoryOutputDenial {
    fn from(cause: worth_query_execution::publication_boundary::document_history::WorthQueryHistoryOutputNavigationDenial) -> Self {
        use worth_query_execution::publication_boundary::document_history::WorthQueryHistoryOutputNavigationDenial as Native;
        match cause { Native::Navigation(cause) => Self::Navigation(cause), Native::Source(cause) => Self::Source(cause) }
    }
}
