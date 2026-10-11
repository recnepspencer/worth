//! Completed custody sweeps entries and each bound source occurrence.
use super::super::refreshed_rejoin::OccurrenceSearchCharge;
use super::super::refreshed_rejoin::{infallible, source_rows};
use super::*;
use core::convert::Infallible;

impl DemandRegistryState {
    pub(in super::super) fn prune_completed_custody<
        C: OccurrenceSearchCharge<Refusal = Infallible>,
    >(
        &mut self,
        charge: &mut C,
    ) -> Result<(), C::Refusal> {
        let mut retired_keys = Vec::new();

        self.source_custody.retain(|commit, custody| {
            infallible(charge.inspect_row());
            if custody.token_count != 0 {
                return true;
            }
            if custody.retired.is_some() {
                if let Some(bound) = &custody.bound_sources {
                    for source in bound {
                        retired_keys.extend(
                            infallible(source_rows(&self.records, &source.identity, charge)).map(
                                |(key, _)| {
                                    infallible(charge.inspect_row());
                                    key.clone()
                                },
                            ),
                        );
                    }
                }
                return false;
            }
            let Some(bound) = &custody.bound_sources else {
                return true;
            };
            let fully_superseded = !bound.is_empty()
                && bound
                    .iter()
                    .all(|source| custody.source_denial(&source.identity).is_some());
            if !custody.completed && !fully_superseded {
                return true;
            }
            if !bound.iter().all(|source| {
                custody.consumed_sources.contains(&source.identity)
                    || custody.source_denial(&source.identity).is_some()
            }) {
                return true;
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
            if !held {
                for source in bound {
                    retired_keys.extend(
                        infallible(source_rows(&self.records, &source.identity, charge)).map(
                            |(key, _)| {
                                infallible(charge.inspect_row());
                                key.clone()
                            },
                        ),
                    );
                }
            }
            held
        });
        for key in retired_keys {
            if let Some(record) = self.records.get_mut(&key) {
                record
                    .source_commits
                    .retain(|commit| self.source_custody.contains_key(commit));
            }
        }
        self.pending_custody_prune = false;
        Ok(())
    }
}
