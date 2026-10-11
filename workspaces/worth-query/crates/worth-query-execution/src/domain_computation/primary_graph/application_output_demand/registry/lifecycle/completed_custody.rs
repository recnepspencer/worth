//! Drain one custody note at a time, after its complete price was admitted.
use super::super::refreshed_rejoin::{infallible, source_rows, OccurrenceSearchCharge};
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
        infallible(charge.inspect_row());
        let custody = &state.source_custody[commit];
        if custody.token_count != 0 {
            note.requeue();
            return Ok(());
        }
        if custody.retired.is_none() {
            let Some(bound) = &custody.bound_sources else {
                note.requeue();
                return Ok(());
            };
            let fully_superseded = !bound.is_empty()
                && bound
                    .iter()
                    .all(|source| custody.source_denial(&source.identity).is_some());
            if !custody.completed && !fully_superseded {
                note.requeue();
                return Ok(());
            }
            if !bound.iter().all(|source| {
                custody.consumed_sources.contains(&source.identity)
                    || custody.source_denial(&source.identity).is_some()
            }) {
                note.requeue();
                return Ok(());
            }
            let held = bound.iter().any(|source| {
                infallible(source_rows(&state.records, &source.identity, charge)).any(
                    |(key, record)| {
                        infallible(charge.inspect_row());
                        record.source_commits.contains(commit)
                            && (record.interests != 0
                                || infallible(refreshed_rejoin::awaited_by_stale_owner(
                                    &state.records,
                                    key,
                                    charge,
                                ))
                                || !record.performed_obligations.is_empty()
                                || !matches!(
                                    record.state,
                                    DemandState::Output(WorthQueryOutputProgress {
                                        checkpoint: Some(WorthQueryOutputCheckpoint::Ready(_)),
                                        ..
                                    }) | DemandState::Failed(_)
                                ))
                    },
                )
            });
            if held {
                note.requeue();
                return Ok(());
            }
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
