//! Real Bank entries call the publication page, resume, subscribe and next doors.
pub use super::fixture::{ordinary_read_world, OWNER};
pub use crate::facade::support::request_scope;
pub use bank_server::{
    BankAccountActivityLiveOutcome as Live, BankApplicationQueryDenial as QueryDenial,
    BankReadControls,
};
pub use std::num::NonZeroUsize;
pub use worth_query_host::facade::{
    application_contribution::{
        WorthQueryAdvancementDenial as Denial,
        WorthQueryManagedComputationResourceDenial as Resource,
        WorthQueryMemoryLimitLevel as Level,
    },
    primary_graph::{
        bound_advancement_requests_on_this_thread_for_test as bound,
        installed_source_reads_on_this_thread_for_test as reads,
        place_managed_computations_on_this_thread_for_test as place,
        WorthQueryApplicationLiveControls as LiveControls,
        WorthQueryApplicationQueryResumeControls as Resume,
        WorthQueryExecutionPlacementForTest as Placement,
    },
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

pub fn query_cause(denial: QueryDenial) -> Denial {
    match denial {
        QueryDenial::ExecutionRequest(cause) => cause,
        other => panic!("opening cause lost: {other:?}"),
    }
}
