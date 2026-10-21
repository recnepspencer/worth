//! Drain actual queued custody while preserving held source ownership.

use super::*;
use worth_runtime_world::facade::CompositeCommitIdentity;

#[cfg(test)]
thread_local! { static CONTACTS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) }; }
#[cfg(test)]
pub(in super::super) fn contacts() -> u64 {
    CONTACTS.with(std::cell::Cell::get)
}

impl DemandRegistryState {
    #[cfg(test)]
    pub(in super::super) fn prune_completed_custody(
        &mut self,
    ) -> Result<(), core::convert::Infallible> {
        let visits = self.custody_cleanup.len;
        for _ in 0..visits {
            let Some(commit) = self.custody_cleanup.head.clone() else {
                break;
            };
            self.prune_custody_entry(&commit);
        }
        Ok(())
    }

    pub(in super::super) fn prune_custody_entry(&mut self, commit: &CompositeCommitIdentity) {
        #[cfg(test)]
        CONTACTS.with(|count| count.set(count.get() + 1));
        let note = self.take_custody_note(commit);
        let state = note.state();
        if !state.custody_prunable(commit) {
            note.requeue();
            return;
        }
        let custody = note.prune();
        for source in custody.bound_sources.iter().flatten() {
            self.records
                .visit_source_rows_mut(&source.identity, |record| {
                    record
                        .source_commits
                        .retain(|known| self.source_custody.contains_key(known));
                });
        }
    }
}
