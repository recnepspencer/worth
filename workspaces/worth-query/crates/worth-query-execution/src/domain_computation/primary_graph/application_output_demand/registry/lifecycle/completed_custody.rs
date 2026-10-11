//! Drain one custody note at a time, after its complete price was admitted.
use super::super::refreshed_rejoin::{infallible, OccurrenceSearchCharge};
use super::*;
use core::convert::Infallible;
use worth_runtime_world::facade::CompositeCommitIdentity;

#[cfg(test)]
thread_local! { static CONTACTS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) }; }
#[cfg(test)]
pub(in super::super) fn contacts() -> u64 {
    CONTACTS.with(std::cell::Cell::get)
}

impl DemandRegistryState {
    pub(in super::super) fn prune_completed_custody<
        C: OccurrenceSearchCharge<Refusal = Infallible>,
    >(
        &mut self,
        charge: &mut C,
    ) -> Result<(), Infallible> {
        let visits = self.custody_cleanup.len;
        for _ in 0..visits {
            let Some(commit) = self.custody_cleanup.head.clone() else {
                break;
            };
            self.prune_custody_entry(&commit, charge)?;
        }
        Ok(())
    }

    pub(in super::super) fn prune_custody_entry<C: OccurrenceSearchCharge<Refusal = Infallible>>(
        &mut self,
        commit: &CompositeCommitIdentity,
        charge: &mut C,
    ) -> Result<(), Infallible> {
        #[cfg(test)]
        CONTACTS.with(|count| count.set(count.get() + 1));
        let note = self.take_custody_note(commit);
        let state = note.state();
        if !infallible(state.custody_prunable(commit, charge)) {
            note.requeue();
            return Ok(());
        }
        let custody = note.prune();
        for source in custody.bound_sources.iter().flatten() {
            infallible(charge.occurrence_path(self.records.len()));
            self.records
                .visit_source_rows_mut(&source.identity, |record| {
                    infallible(charge.inspect_row());
                    record
                        .source_commits
                        .retain(|known| self.source_custody.contains_key(known));
                });
        }
        Ok(())
    }
}
