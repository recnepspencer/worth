pub(in crate::domain_computation::primary_graph) use selected_execution_finish::RunningSelectedExecution;
mod cached_reclaim;
pub(super) mod cleanup_price;
mod closed_retirement;
pub(super) mod completed_custody;
mod joined_ready;
mod performed_release;
pub(super) mod release;
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
        let visible = super::post_commit::custody_reader::visible(&state, &prepared.source_commit);
        let custody = state.source_custody
            .get_mut(&prepared.source_commit).filter(|_| visible).ok_or_else(|| {
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
        state.discard_custody(source_commit);
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
            && custody
                .source
                .as_ref()
                .is_some_and(|source| source.receipt.same_retained_output_source_as(receipt))
        {
            custody.completed = true;
            custody.source = None;
            custody.discovery = None;
        }
        state.note_custody(receipt.committed_product_publication().composite_commit());
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

    pub(in crate::domain_computation::primary_graph) fn prepared_source_count(&self) -> usize {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .source_custody
            .values()
            .map(SourceCustody::prepared_count)
            .sum()
    }

    // Completed cleanup notes contain coordinates, not a performed source or
    // its observation. The paid drain may retain those notes after release.
    pub(in crate::domain_computation::primary_graph) fn retained_source_custody_count(
        &self,
    ) -> usize {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .source_custody
            .values()
            .filter(|custody| custody.source.is_some() || custody.discovery.is_some())
            .count()
    }

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
        let mut record = state
            .records
            .get_mut(&interest.key)
            .expect("executing demand retains its owner record");
        selected_execution_finish::relinquish_record(&mut record);
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
