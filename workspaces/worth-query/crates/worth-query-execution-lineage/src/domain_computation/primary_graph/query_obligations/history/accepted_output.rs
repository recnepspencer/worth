//! Lifecycle records use the one serialized Query publisher.
use super::*;
use journal::*;
use worth_execution::ExecutionRequest;
use worth_query_execution_obligations::facade::ledger_inspect;

impl QueryObligationHistory {
    pub fn record_accepted_output(
        &self,
        request: &ExecutionRequest<'_, '_>,
        row: QueryAcceptedOutputRecord,
    ) -> Result<(), QueryObligationDenial> {
        let Some(medium) = self.medium.get() else {
            return Ok(());
        };
        let driver = self.lock_publication()?;
        driver.finish_pending(medium.as_ref(), request)?;
        if self.state().journal.as_ref().is_some_and(|journal| {
            ledger_inspect::output(&journal.ledger).accepted.get(&(
                row.branch.clone(),
                row.producer.clone(),
                row.scope,
                row.partition,
            )) == Some(&row)
        }) {
            return Ok(());
        }
        driver.publish(medium.as_ref(), row, request)
    }
    pub fn accepted_outputs(&self) -> Vec<QueryAcceptedOutputRecord> {
        self.state()
            .journal
            .as_ref()
            .map(|journal| {
                ledger_inspect::output(&journal.ledger)
                    .accepted
                    .values()
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }
}
