//! Charge one bounded eligibility or release transition before owner effects.
use super::*;

impl<S> HistoryJournal<'_, S> {
    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn quote_frontier(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        // At most the current node, real bookmarks and one pending reservation
        // precede the oldest eligible entry. Each tree operation also pays its
        // conservative balanced path; no historical annotation range is walked.
        let skipped = self.bookmarked.saturating_add(2) as u64;
        let units = skipped
            + 4 * seek(self.retained.len())
            + 3 * seek(self.nodes.len())
            + 2 * seek(self.children.len())
            + self.releases.len() as u64
            + 8;
        self.quote_retention_work(execution, units)
    }

    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn quote_release(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        // One queue rotation, the funded unfinished-source slots, one custody
        // lookup/removal and bounded exact-key finalization. Native close charges
        // its own actual work independently after this platform transition.
        let units = self.navigation_sources.bound() as u64
            + self.releases.len() as u64
            + 2 * seek(self.past_heads.len())
            + 4;
        self.quote_retention_work(execution, units)
    }

    fn quote_retention_work(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        units: u64,
    ) -> Result<(), Denial> {
        use crate::domain_computation::request_execution::advancement_denial;
        let outcome = execution
            .run_native_with_checkpoint(|context| {
                if context.has_accounting() {
                    context.checkpoint(units)
                } else {
                    context.poll()
                }
            })
            .map_err(|cause| Denial::Stopped(advancement_denial(cause)))?;
        outcome.0.map_err(|stop| {
            Denial::Stopped(advancement_denial(
                worth_execution::WorkCeilingDenial::Stopped(stop),
            ))
        })?;
        if let Some(observation) = &self.observation {
            observation.retention_work(units);
        }
        Ok(())
    }
}

fn seek(len: usize) -> u64 {
    2 * u64::from(usize::BITS - len.leading_zeros()) + 1
}
