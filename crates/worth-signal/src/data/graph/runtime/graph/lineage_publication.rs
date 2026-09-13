//! Select ordinary or owner-reserved lineage publication at the graph boundary.
use super::{map_node_edit_accounting, map_node_edit_retention, SignalGraph};
use crate::data::error::SignalError;
use crate::data::retained_storage::RetainedStoragePreparation as Work;
use crate::diagnostics::lineage::LineageRecord;
use crate::diagnostics::state::LineagePublicationDenial;
use crate::logic::evaluation::EvaluationWork;

impl SignalGraph {
    pub(crate) fn record_evaluation_lineage(
        &mut self,
        record: LineageRecord,
        work: &mut EvaluationWork<'_>,
    ) -> Result<(), SignalError> {
        let Some(ledger) = self.arena.retained_node_ledger.clone() else {
            self.observation.diagnostics.record_lineage_record(record);
            return Ok(());
        };
        let result = match work {
            EvaluationWork::Conditional(work) => self
                .observation
                .diagnostics
                .record_retained_lineage(record, &ledger, work),
            EvaluationWork::Ordinary => {
                let maximum = self
                    .installed_runtime_policy()
                    .conditional_evaluation_budget()
                    .maximum_attempt_visits;
                self.observation.diagnostics.record_retained_lineage(
                    record,
                    &ledger,
                    &mut Work::new(maximum),
                )
            }
        };
        result.map_err(|denial| match denial {
            LineagePublicationDenial::Accounting(denial) => map_node_edit_accounting(denial),
            LineagePublicationDenial::Retention(denial) => map_node_edit_retention(denial),
            LineagePublicationDenial::Unprepared => SignalError::EvaluationStorageUnavailable,
        })
    }
}
