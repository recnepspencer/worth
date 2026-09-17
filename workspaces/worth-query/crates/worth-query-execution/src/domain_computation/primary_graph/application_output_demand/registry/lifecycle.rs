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

    pub(in crate::domain_computation::primary_graph) fn release_product_occurrence(
        &self,
        occurrence: worth_runtime_world::facade::ProductBranchIncarnation,
    ) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for custody in state.source_custody.values_mut() {
            if custody.occurrence == occurrence {
                custody.retire(WorthQueryOutputDemandDenial::new(
                    crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind::Closed,
                    "product occurrence retired before required-output custody was consumed",
                ));
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
        state.prune_completed_custody();
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
        state.prune_completed_custody();
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
            self.records.values().any(|record| {
                record.source_commits.contains(commit)
                    && (record.interests != 0
                        || !matches!(
                            record.state,
                            DemandState::Completed(_) | DemandState::Failed(_)
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
}
