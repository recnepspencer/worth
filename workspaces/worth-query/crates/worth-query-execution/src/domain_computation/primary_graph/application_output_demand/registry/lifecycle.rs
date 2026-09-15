use super::*;

impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn discard_prepared_source(
        &self,
        source_commit: &worth_runtime_world::facade::CompositeCommitIdentity,
    ) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state
            .prepared_sources
            .retain(|(commit, _)| commit != source_commit);
        state.retired_prepared_sources.remove(source_commit);
    }

    pub(in crate::domain_computation::primary_graph) fn release_prepared_token(
        &self,
        source_commit: &worth_runtime_world::facade::CompositeCommitIdentity,
    ) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.retired_prepared_sources.remove(source_commit);
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

    pub(in crate::domain_computation::primary_graph) fn release_product_occurrence(
        &self,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    ) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Self::retire_program_recovery_occurrence(&mut state, occurrence);
        let prepared_sources = std::mem::take(&mut state.prepared_sources);
        for (commit, source) in prepared_sources {
            if source.receipt.product_branch().occurrence() == occurrence {
                state.retired_prepared_sources.insert(
                    commit,
                    WorthQueryOutputDemandDenial::new(
                        crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::Closed,
                        "product occurrence retired before required-output custody was consumed",
                    ),
                );
            } else {
                state.prepared_sources.push((commit, source));
            }
        }
        if let Some(preparation) = state.source_preparations.get_mut(&occurrence) {
            preparation.retired = true;
        }
        state.records.retain(|_, record| {
            if record.product_occurrence != occurrence {
                return true;
            }
            if record.interests == 0 {
                return false;
            }
            record.state = DemandState::Failed(WorthQueryOutputDemandDenial::new(
                crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::Closed,
                "product occurrence retired",
            ));
            record.performed_source = None;
            record.wake.notify();
            true
        });
    }

    #[cfg(feature = "test-primary-graph-faults")]
    pub(in crate::domain_computation::primary_graph) fn prepared_source_count(&self) -> usize {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .prepared_sources
            .len()
    }

    #[cfg(feature = "test-primary-graph-faults")]
    pub(in crate::domain_computation::primary_graph) fn program_recovery_count(&self) -> usize {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .program_recovery
            .len()
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
        if matches!(
            record.state,
            DemandState::Scheduling | DemandState::Running | DemandState::Recovering(_)
        ) {
            record.state = match std::mem::replace(&mut record.state, DemandState::Admitted) {
                DemandState::Scheduling => DemandState::Admitted,
                DemandState::Running => DemandState::Scheduled,
                DemandState::Recovering(completion) => DemandState::Completed(completion),
                _ => unreachable!("relinquished state was checked above"),
            };
            record.wake.notify();
        }
    }

    pub(in crate::domain_computation::primary_graph) fn release(
        &self,
        interest: &WorthQueryOutputDemandInterest,
    ) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let remove = state.records.get_mut(&interest.key).is_some_and(|record| {
            record.interests = record.interests.saturating_sub(1);
            if record.interests == 0 {
                match &record.state {
                    DemandState::Settled(settlement) => {
                        record.state = DemandState::Completed(settlement.completion());
                    }
                    DemandState::Recovering(completion) => {
                        record.state = DemandState::Completed(completion.clone());
                        record.wake.notify();
                    }
                    _ => {}
                }
            }
            record.interests == 0
                && (!record.required || matches!(record.state, DemandState::Failed(_)))
                && record.performed_source.is_none()
                && !matches!(
                    record.state,
                    DemandState::Scheduling
                        | DemandState::Running
                        | DemandState::Recovering(_)
                        | DemandState::Delivering
                        | DemandState::DeliveryPending(_)
                        | DemandState::EvaluatingReadiness
                        | DemandState::ReadinessPending(_)
                        | DemandState::Completed(_)
                )
        });
        if remove {
            state.records.remove(&interest.key);
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
}
