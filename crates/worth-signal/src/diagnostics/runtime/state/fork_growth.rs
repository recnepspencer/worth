//! Bounded conversion preparation; retained diagnostic frames are never walked.
use super::DiagnosticsState;
use crate::data::retained_storage::{
    RetainedStorageCharge as Charge, RetainedStorageForkGrowth,
    RetainedStorageForkGrowthDenial as DiagnosticForkGrowthDenial,
    RetainedStoragePreparation as Work, RetainedStoragePreparationDenial,
};

impl RetainedStorageForkGrowth for DiagnosticsState {
    /// Covers new conversion allocations only. Existing frames and inline root
    /// custody remain the enclosing evaluation owner's responsibility. Missing
    /// facts require explicit cold preparation; ordinary forks never repair them.
    fn prepare_fork_growth(
        &mut self,
        work: &mut Work,
    ) -> Result<Charge, DiagnosticForkGrowthDenial> {
        work.reserve_visits(std::mem::size_of::<Self>())?;
        let mut growth = Charge::ZERO;
        macro_rules! index {
            ($index:expr) => {{
                let index = &mut $index;
                growth = growth.checked_add(index.prepare_fork_growth(work)?)?;
            }};
        }
        index!(self.replay_events_by_branch);
        index!(self.replay_events_by_node);
        index!(self.replay_events_by_artifact);
        index!(self.replay_cursor_offsets);
        index!(self.snapshot_replay_cursors);
        index!(self.lineage_records_by_artifact);
        index!(self.lineage_records_by_node);
        index!(self.explanation_facts);
        index!(self.provenance_facts);
        index!(self.branch_catalog);
        if let Some(input) = &mut self.pending_input {
            index!(input.changed_nodes);
            index!(input.changed_aspects);
        }
        work.reserve_visits(
            usize::try_from(growth.bytes())
                .map_err(|_| RetainedStoragePreparationDenial::ChargeOverflow)?,
        )?;
        Ok(growth)
    }
}

#[cfg(test)]
mod tests;
