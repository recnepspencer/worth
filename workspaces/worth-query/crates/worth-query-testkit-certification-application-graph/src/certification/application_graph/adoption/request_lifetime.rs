//! A staged adoption carries facts across host calls, never request custody.
pub use std::num::NonZeroUsize;
pub use worth_query_host::facade::{
    application_contribution::{
        WorthQueryAdvancementDenial as Denial,
        WorthQueryManagedComputationResourceDenial as Resource,
        WorthQueryMemoryLimitLevel as Level,
    },
    application_entry::{
        WorthQueryApplicationRequestExt, WorthQueryBranchAdoptionPublicationOutcome as Outcome,
    },
    application_installation::WorthQueryProgramOwner,
    primary_graph::{
        advancement_requests_on_this_thread_for_test as reports,
        bound_advancement_requests_on_this_thread_for_test as bound,
        installed_source_reads_on_this_thread_for_test as reads,
        place_managed_computations_on_this_thread_for_test as place,
        WorthQueryExecutionPlacementForTest as Placement,
    },
};
pub use worth_query_testkit_document_retention::facade::document_retention_model::{
    host::publish_on_first_program,
    operator_identity::{authenticate_operator, request_scope},
    programs::RetentionProgramP1,
};
pub struct Restore(
    pub Placement,
    pub Option<worth_foundational::ExecutionBudget>,
);
impl Drop for Restore {
    fn drop(&mut self) {
        place(self.0);
        bound(self.1);
    }
}

#[path = "request_lifetime/entry_custody.rs"]
pub mod entry_custody;
