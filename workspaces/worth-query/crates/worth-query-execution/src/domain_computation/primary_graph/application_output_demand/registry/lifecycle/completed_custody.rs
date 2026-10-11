//! Drain one custody note at a time, after its complete price was admitted.
use super::super::refreshed_rejoin::{infallible, source_rows, OccurrenceSearchCharge};
use super::*;
use core::convert::Infallible;
use worth_runtime_world::facade::CompositeCommitIdentity;

impl DemandRegistryState {
    pub(in super::super) fn prune_completed_custody<
        C: OccurrenceSearchCharge<Refusal = Infallible>,
    >(
        &mut self,
        charge: &mut C,
    ) -> Result<(), Infallible> {
        while let Some(commit) = self.custody_cleanup.head.clone() {
            self.prune_custody_entry(&commit, charge)?;
        }
        Ok(())
    }

    pub(in super::super) fn prune_custody_entry<C: OccurrenceSearchCharge<Refusal = Infallible>>(
        &mut self,
        commit: &CompositeCommitIdentity,
        charge: &mut C,
    ) -> Result<(), Infallible> {
        self.forget_custody_note(commit);
        infallible(charge.inspect_row());
        let custody = &self.source_custody[commit];
        if custody.token_count != 0 {
            return Ok(());
        }
        if custody.retired.is_none() {
            let Some(bound) = &custody.bound_sources else {
                return Ok(());
            };
            let fully_superseded = !bound.is_empty()
                && bound
                    .iter()
                    .all(|source| custody.source_denial(&source.identity).is_some());
            if !custody.completed && !fully_superseded {
                return Ok(());
            }
            if !bound.iter().all(|source| {
                custody.consumed_sources.contains(&source.identity)
                    || custody.source_denial(&source.identity).is_some()
            }) {
                return Ok(());
            }
            let held = bound.iter().any(|source| {
                infallible(source_rows(&self.records, &source.identity, charge)).any(
                    |(key, record)| {
                        infallible(charge.inspect_row());
                        record.source_commits.contains(commit)
                            && (record.interests != 0
                                || infallible(refreshed_rejoin::awaited_by_stale_owner(
                                    &self.records,
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
                return Ok(());
            }
        }
        let custody = self
            .source_custody
            .remove(commit)
            .expect("priced entry exists");
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
