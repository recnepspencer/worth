//! Completed source cleanup visits only its declared source occurrences.
use super::*;

impl DemandRegistryState {
    pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn prune_completed_custody(
        &mut self,
    ) {
        let mut retired_keys = Vec::new();
        self.source_custody.retain(|commit, custody| {
            if custody.token_count != 0 {
                return true;
            }
            if custody.retired.is_some() {
                if let Some(bound) = &custody.bound_sources {
                    for source in bound {
                        retired_keys.extend(
                            self.records
                                .source_occurrence_rows(&source.identity)
                                .map(|(key, _)| key.clone()),
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
                self.records
                    .source_occurrence_rows(&source.identity)
                    .any(|(key, record)| {
                        record.source_commits.contains(commit)
                            && (record.interests != 0
                                || refreshed_rejoin::awaited_by_stale_owner(&self.records, key)
                                || !record.performed_obligations.is_empty()
                                || !matches!(
                                    record.state,
                                    DemandState::Output(WorthQueryOutputProgress {
                                        checkpoint: Some(WorthQueryOutputCheckpoint::Ready(_)),
                                        ..
                                    }) | DemandState::Failed(_)
                                ))
                    })
            });
            if !held {
                for source in bound {
                    retired_keys.extend(
                        self.records
                            .source_occurrence_rows(&source.identity)
                            .map(|(key, _)| key.clone()),
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
    }
}
