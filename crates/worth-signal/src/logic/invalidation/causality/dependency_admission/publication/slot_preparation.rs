//! Bind semantic cause replacements to concrete future store handles.
use super::*;
use crate::data::graph::storage::invalidation_causes::PreparedCauseSlot;

impl SignalGraph {
    pub(super) fn prepare_direct_cause_slots(
        &self,
        admission: &PreparedDirectCauseAdmission,
        release_producer: bool,
        work: &mut Work,
    ) -> Result<Vec<PreparedCauseSlot>, SignalError> {
        let mut work = crate::logic::evaluation::EvaluationWork::Conditional(work);
        self.admit_pending_cause_handle_reads(admission.replacements.len() + 1, &mut work)?;
        work.reserve(
            admission
                .replacements
                .len()
                .checked_mul(std::mem::size_of::<PreparedCauseSlot>() + 1),
        )?;
        let mut cursor = self.cause_sets.prepare_cause_slots()?;
        if release_producer {
            cursor.release(
                self.node_pending_cause_set_id(admission.producer)?,
                &mut work,
            )?;
        }
        let mut slots = Vec::with_capacity(admission.replacements.len());
        for replacement in &admission.replacements {
            slots.push(cursor.replacement(
                self.node_pending_cause_set_id(replacement.consumer)?,
                replacement.causes.is_empty(),
                &mut work,
            )?);
        }
        Ok(slots)
    }
}
