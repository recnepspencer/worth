mod cached_reclaim;
mod closed_retirement;
mod joined_ready;
mod occurrence_retirement;
mod selected_execution_finish;

use super::*;

impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn retain_program_source(
        &self,
        prepared: &crate::domain_computation::primary_graph::WorthQueryPreparedRequiredOutputSource,
        root_kind: super::PreparedOutputRootKind,
    ) -> Result<
        crate::domain_computation::primary_graph::WorthQueryPreparedRequiredOutputSource,
        crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial,
    > {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let custody = state.source_custody.get_mut(&prepared.source_commit).ok_or_else(|| {
            crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial::new(
                crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                "the program source publication no longer retains custody",
            )
        })?;
        if custody.occurrence != prepared.product_occurrence || custody.root_kind != root_kind {
            return Err(crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial::new(
                crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::ForeignSource,
                "the program source occurrence differs from its prepared custody",
            ));
        }
        if let Some(denial) = &custody.retired {
            return Err(denial.clone());
        }
        custody.token_count = custody.token_count.saturating_add(1);
        Ok(
            crate::domain_computation::primary_graph::WorthQueryPreparedRequiredOutputSource {
                runtime_authority: prepared.runtime_authority,
                source_commit: prepared.source_commit.clone(),
                product_occurrence: prepared.product_occurrence,
                owner: self.clone(),
            },
        )
    }

    pub(in crate::domain_computation::primary_graph) fn discard_prepared_source(
        &self,
        source_commit: &worth_runtime_world::facade::CompositeCommitIdentity,
    ) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.source_custody.remove(source_commit);
    }

    pub(in crate::domain_computation::primary_graph) fn release_prepared_token(
        &self,
        source_commit: &worth_runtime_world::facade::CompositeCommitIdentity,
    ) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(custody) = state.source_custody.get_mut(source_commit) {
            custody.token_count = custody.token_count.saturating_sub(1);
        }
        state.prune_completed_custody();
    }

    pub(in crate::domain_computation::primary_graph) fn complete_prepared_source(
        &self,
        receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    ) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(custody) = state
            .source_custody
            .get_mut(receipt.committed_product_publication().composite_commit())
        {
            if custody
                .source
                .as_ref()
                .is_some_and(|source| source.receipt.same_retained_output_source_as(receipt))
            {
                custody.completed = true;
                custody.source = None;
                custody.discovery = None;
            }
        }
        state.prune_completed_custody();
    }

    pub(in crate::domain_computation::primary_graph) fn begin_source_preparation(
        &self,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    ) -> WorthQueryRequiredOutputSourcePreparation {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let preparation = state.source_preparations.entry(occurrence).or_default();
        preparation.active = preparation.active.saturating_add(1);
        WorthQueryRequiredOutputSourcePreparation {
            occurrence,
            owner: self.clone(),
        }
    }

    #[cfg(feature = "test-primary-graph-faults")]
    pub(in crate::domain_computation::primary_graph) fn prepared_source_count(&self) -> usize {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .source_custody
            .values()
            .map(SourceCustody::prepared_count)
            .sum()
    }

    #[cfg(feature = "test-primary-graph-faults")]
    pub(in crate::domain_computation::primary_graph) fn retained_source_custody_count(
        &self,
    ) -> usize {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .source_custody
            .len()
    }

    #[cfg(feature = "test-primary-graph-faults")]
    pub(in crate::domain_computation::primary_graph) fn output_checkpoint_snapshot_state(
        &self,
    ) -> (usize, usize) {
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state
            .records
            .values()
            .fold((0, 0), |(outputs, pinned), record| {
                if let DemandState::Output(output) = &record.state {
                    (
                        outputs + 1,
                        pinned
                            + usize::from(
                                output
                                    .checkpoint
                                    .as_ref()
                                    .and_then(WorthQueryOutputCheckpoint::receipt)
                                    .is_some_and(|receipt| {
                                        receipt
                                            .committed_product_publication()
                                            .has_output_demand_observation_for_test()
                                    }),
                            ),
                    )
                } else {
                    (outputs, pinned)
                }
            })
    }

    pub(in crate::domain_computation::primary_graph) fn relinquish_execution(
        &self,
        interest: &WorthQueryOutputDemandInterest,
    ) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let record = state
            .records
            .get_mut(&interest.key)
            .expect("executing demand retains its owner record");
        selected_execution_finish::relinquish_record(record);
    }

    pub(in crate::domain_computation::primary_graph) fn release(
        &self,
        interest: &WorthQueryOutputDemandInterest,
    ) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut released_bytes = 0;
        let replaced = refreshed_rejoin::replaced_under_refresh(&state.records, &interest.key);
        let remove = state.records.get_mut(&interest.key).is_some_and(|record| {
            record.interests = record.interests.saturating_sub(1);
            if interest.requires_output {
                record.required_interests = record.required_interests.saturating_sub(1);
            }
            // Keep a ready ordinary output for an equivalent later demand.
            // Source supersession and occurrence retirement remove it; closing
            // the handle releases only the caller's interest.
            let ready = record.has_cached_ready();
            let stopped = matches!(record.state, DemandState::Failed(_))
                || matches!(&record.state, DemandState::Output(output)
                    if matches!(output.advancement, WorthQueryOutputAdvancement::Stopped { .. }));
            if stopped && record.interests == 0 && !record.pending_cleanup_queued {
                released_bytes += record.release_obligations();
            }
            record.interests == 0
                && record.framework_required_count == 0
                && record.prepared_prerequisite_claims == 0
                && !record.pending_cleanup_queued
                && ((!record.is_required() && !ready) || stopped)
                && !(ready && replaced)
                && record.performed_obligations.is_empty()
                && record.performed_source.is_none()
                && match &record.state {
                    DemandState::Scheduling | DemandState::Running => false,
                    DemandState::Output(output) => {
                        matches!(
                            output.advancement,
                            WorthQueryOutputAdvancement::Stopped { .. }
                        ) || matches!(
                            output.checkpoint,
                            Some(WorthQueryOutputCheckpoint::Ready(_))
                        )
                    }
                    _ => true,
                }
        });
        let terminal = state.records.get(&interest.key).is_some_and(|record| {
            matches!(record.state, DemandState::Failed(_))
                || matches!(&record.state, DemandState::Output(output)
                    if matches!(output.advancement, WorthQueryOutputAdvancement::Stopped { .. }))
        });
        let released_prerequisites =
            terminal.then(|| state.release_record_prerequisites(&interest.key));
        if remove {
            if let Some(record) = state.records.remove(&interest.key) {
                released_bytes += record.obligation_reserved_bytes();
                if record.unpublished_new_key() {
                    state.revive_replaced_ready(&interest.key);
                }
            }
        }
        if !state
            .records
            .get(&interest.key)
            .is_some_and(|record| record.pending_cleanup_queued)
        {
            state.remove_required_member_if_released(&interest.key);
        }
        // A held successor ends with its row's work or with the last owner
        // awaiting that row; it drops after the lock.
        let mut finished = state.take_finished_successor(&interest.key);
        let unawaited = state
            .records
            .get_mut(&interest.key)
            .and_then(DemandRecord::take_unawaited_reopened);
        // A closing stale owner may have been the last one awaiting the
        // newest row of its occurrence.
        if terminal {
            if let Some(newest) =
                refreshed_rejoin::newest_of_occurrence(&state.records, &interest.key)
            {
                state.remove_required_member_if_released(&newest);
                finished = finished.or_else(|| state.take_finished_successor(&newest));
            }
        }
        state.obligation_reserved_bytes = state
            .obligation_reserved_bytes
            .saturating_sub(released_bytes);
        // The closing row, each upstream its released claims leave unheld,
        // and each row it replaced, keeps only what it can still answer for.
        let candidates = std::iter::once(interest.key.clone())
            .chain(
                released_prerequisites
                    .iter()
                    .flatten()
                    .map(|upstream| upstream.as_ref().clone()),
            )
            .chain(refreshed_rejoin::replaced_by_published(
                &state.records,
                &interest.key,
            ))
            .collect();
        let (retired, released_claims) = state.retire_closed_rows(candidates);
        state.prune_completed_custody();
        drop(state);
        drop(retired);
        drop(released_claims);
        drop(released_prerequisites);
        drop(finished);
        drop(unawaited);
    }
}

impl DemandRegistryState {
    pub(super) fn prune_completed_custody(&mut self) {
        let prior_count = self.source_custody.len();
        self.source_custody.retain(|commit, custody| {
            if custody.token_count != 0 {
                return true;
            }
            if custody.retired.is_some() {
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
            self.records.iter().any(|(key, record)| {
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
        if self.source_custody.len() != prior_count {
            for record in self.records.values_mut() {
                record
                    .source_commits
                    .retain(|commit| self.source_custody.contains_key(commit));
            }
        }
    }
}

impl Drop for WorthQueryRequiredOutputSourcePreparation {
    fn drop(&mut self) {
        let mut state = self
            .owner
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let remove = state
            .source_preparations
            .get_mut(&self.occurrence)
            .is_some_and(|preparation| {
                preparation.active = preparation.active.saturating_sub(1);
                preparation.active == 0
            });
        if remove {
            state.source_preparations.remove(&self.occurrence);
        }
    }
}

impl Drop for WorthQueryOutputDemandInterest {
    fn drop(&mut self) {
        self.owner.release(self);
    }
}

impl WorthQueryOutputDemandInterest {
    pub(in crate::domain_computation::primary_graph) fn notifications(
        &self,
    ) -> WorthQueryOutputDemandNotifications {
        self.notifications.clone()
    }

    pub(in crate::domain_computation::primary_graph) const fn key(
        &self,
    ) -> &WorthQueryOutputDemandKey {
        &self.key
    }

    /// Whether this interest supersedes `older` as a continuation: a newer
    /// source of the same occurrence, or the same row refreshed again.
    pub(in crate::domain_computation::primary_graph) fn supersedes(&self, older: &Self) -> bool {
        self.key
            .replacement_order(&older.key)
            .is_some_and(|order| order != std::cmp::Ordering::Less)
    }
}
