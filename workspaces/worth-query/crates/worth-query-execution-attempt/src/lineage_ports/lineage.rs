//! PreparingCommitPort implementations for attempt-owned durable commit guards.

use worth_query_execution_lineage::facade::ports::PreparingCommitPort;

use crate::domain_computation::primary_graph::provider::application_attempt_state::commit_preparation::relational_commit::commit_execution::durable_custody::{
    QueryPreparingApplicationCommit, RetainedQueryDurablePreparation,
};

impl PreparingCommitPort for RetainedQueryDurablePreparation {
    fn is_unprepared(&self) -> bool {
        RetainedQueryDurablePreparation::is_unprepared(self)
    }
}

impl PreparingCommitPort for QueryPreparingApplicationCommit {
    fn is_unprepared(&self) -> bool {
        QueryPreparingApplicationCommit::is_unprepared(self)
    }
}
