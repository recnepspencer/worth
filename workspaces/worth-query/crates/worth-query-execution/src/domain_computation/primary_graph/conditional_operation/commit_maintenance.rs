use worth_query_installation::facade::ApplicationSchema;
use worth_relational::facade::{history::RelationalCommitReceipt, transactions::RecordRef};

use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema + 'static,
{
    /// Records owner movement for exact successor-product reconsideration.
    /// Existing product observations are immutable and never refresh from a
    /// later Relational commit.
    pub(in crate::domain_computation::primary_graph) fn maintain_conditional_commit(
        &self,
        commit: &RelationalCommitReceipt,
        records: Vec<RecordRef>,
    ) {
        self.primary_provider
            .record_conditional_commit(commit, records);
    }
}
