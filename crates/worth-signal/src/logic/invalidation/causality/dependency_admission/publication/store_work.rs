//! Gather selected store handles while final publication remains read-only.
use super::{PreparedDirectCausePublication, SignalGraph};
use crate::data::error::SignalError;
use crate::data::graph::storage::invalidation_causes::{NormalizedCauseSet, PendingCauseSetId};
use crate::logic::evaluation::EvaluationWork;
impl PreparedDirectCausePublication {
    pub(crate) fn validate_before_evaluation(
        &self,
        graph: &SignalGraph,
        version: crate::data::aspect::AspectVersion,
        regions: &[crate::data::output::ChangedRegion],
        work: &mut EvaluationWork<'_>,
    ) -> Result<(), SignalError> {
        if let Some(delta) = &self.admission.commit {
            work.reserve(Some(self.admission.replacements.len()))?;
            for replacement in &self.admission.replacements {
                graph.validate_prepared_causes_before_evaluation(
                    replacement.consumer,
                    &replacement.causes,
                    delta,
                    version,
                    regions,
                    work,
                )?;
            }
        }
        Ok(())
    }

    pub(crate) fn admit_cause_store_work(
        &self,
        graph: &SignalGraph,
        work: &mut EvaluationWork<'_>,
    ) -> Result<(), SignalError> {
        if self.admission.replacements.is_empty() {
            return Ok(());
        }
        graph.admit_pending_cause_handle_reads(self.admission.replacements.len() + 1, work)?;
        work.reserve(
            self.admission
                .replacements
                .len()
                .checked_mul(std::mem::size_of::<(PendingCauseSetId, &NormalizedCauseSet)>() + 3)
                .filter(|n| *n <= isize::MAX as usize),
        )?;
        let mut replacements = Vec::with_capacity(self.admission.replacements.len());
        for replacement in &self.admission.replacements {
            let current = graph.node_pending_cause_set_id(replacement.consumer)?;
            if current != replacement.slot.current() {
                return Err(SignalError::invalid_input(
                    "prepared cause slot belongs to another node state",
                ));
            }
            replacements.push((current, &replacement.causes));
        }
        graph.cause_sets.admit_replacement_batch_work(
            graph.node_pending_cause_set_id(self.admission.producer)?,
            &replacements,
            work,
        )
    }
}
