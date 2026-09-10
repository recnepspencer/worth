use crate::branch::OwnerRetirementWork;
use crate::recovery::{
    ProductUnpublishedOwnerEffects, ProductUnpublishedRecoveryHandle, RecoveryCleanupOutcome,
    RecoveryContinuationContract,
};

use super::RuntimeWorldOwnerRoot;

#[cfg(test)]
#[path = "recovery_service/tests.rs"]
mod settlement_catalog_tests;

#[cfg(test)]
#[path = "recovery_service/concurrency_tests.rs"]
mod concurrency_tests;

#[cfg(test)]
#[path = "recovery_service/unwind_tests.rs"]
mod unwind_tests;

#[cfg(test)]
mod update_control;
#[cfg(test)]
use update_control::{install_test_recovery_update_pause, pause_test_recovery_update};

impl<D, I, E, Ctx, T> RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    /// Enumerate owner-issued recovery handles. A handle is only an
    /// attempt-affine inspection key; it cannot publish a product branch or
    /// mint a component-owner capability.
    #[cfg(test)]
    pub fn recovery_handles(&self) -> Vec<ProductUnpublishedRecoveryHandle> {
        let affinity = self.state.recovery.affinity();
        self.state
            .recovery
            .identities()
            .into_iter()
            .map(|identity| ProductUnpublishedRecoveryHandle::new(identity, affinity))
            .collect()
    }

    #[cfg(test)]
    pub fn recovery_record_count(&self) -> usize {
        self.state.recovery.installed_slots()
    }

    #[cfg(test)]
    pub fn inspect_recovery(
        &self,
        handle: &ProductUnpublishedRecoveryHandle,
    ) -> Option<ProductUnpublishedOwnerEffects> {
        self.state
            .recovery
            .lookup_record(handle)
            .map(ProductUnpublishedOwnerEffects::from_catalog_record)
    }

    /// Consume a caller capability and explicitly release its record. The
    /// capability is dropped first so that it cannot be the inspection that
    /// keeps the record retained; the release itself is
    /// [`Self::cleanup_recovery_handle`].
    #[cfg(test)]
    pub(crate) fn cleanup_recovery(
        &self,
        effects: ProductUnpublishedOwnerEffects,
    ) -> Option<Vec<OwnerRetirementWork>> {
        let handle = effects.recovery_handle();
        drop(effects);
        self.cleanup_recovery_handle(&handle)
    }

    /// Explicitly release a record only when no Relational settlement route
    /// remains. The catalog retains custody if a separate caller still
    /// inspects the same record.
    ///
    /// Releasing the record is also what drains the component branches its
    /// forks created: the record names the exact destination occurrence, and
    /// the custody registry is keyed by that occurrence, so the typed work the
    /// component owners still owe is returned here rather than left installed
    /// under a branch that will never exist. `None` is a record this call did
    /// not release; it drains nothing, because it retired nothing.
    #[cfg(test)]
    pub(crate) fn cleanup_recovery_handle(
        &self,
        handle: &ProductUnpublishedRecoveryHandle,
    ) -> Option<Vec<OwnerRetirementWork>> {
        let work = self.reserve_recovery_retirement_work(handle).ok()?;
        let released = self.state.recovery.cleanup_record(handle, None).ok()?;
        Some(self.drain_released_occurrence(&released, work))
    }

    /// Drain the custody charged to the occurrence a released record named.
    /// A record that named no occurrence was a publication: it charged no
    /// custody, and there is nothing to drain.
    fn reserve_recovery_retirement_work(
        &self,
        handle: &ProductUnpublishedRecoveryHandle,
    ) -> Result<Vec<OwnerRetirementWork>, crate::recovery::RuntimeWorldRecoveryDenial> {
        let record = self.state.recovery.inspect_record(handle)?;
        let destination = record
            .destination()
            .map(|(branch, incarnation)| (branch.clone(), incarnation));
        let work = match destination {
            Some((branch, incarnation)) => self
                .state
                .custody
                .reserve_retirement_work(&branch, incarnation)
                .map_err(|()| {
                    crate::recovery::RuntimeWorldRecoveryDenial::OutputCapacityExhausted
                })?,
            None => Vec::new(),
        };
        drop(record);
        Ok(work)
    }

    fn drain_released_occurrence(
        &self,
        released: &RecoveryCleanupOutcome,
        mut work: Vec<OwnerRetirementWork>,
    ) -> Vec<OwnerRetirementWork> {
        let Some((branch, incarnation)) = released.destination() else {
            return work;
        };
        self.state
            .custody
            .drain_retirement_work_into(branch, incarnation, &mut work);
        work
    }
}

impl<D, I, E, Ctx, T> super::super::ports::RuntimeWorldRecoveryService
    for RuntimeWorldOwnerRoot<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + Send + Sync + 'static,
    I: Copy + Ord + Send + Sync + 'static,
    T: Copy + Ord + Send + Sync + 'static,
{
    fn inspect_effects(
        &self,
        handle: &ProductUnpublishedRecoveryHandle,
    ) -> Result<ProductUnpublishedOwnerEffects, crate::recovery::RuntimeWorldRecoveryDenial> {
        let _operation = self
            .reserve_recovery_operation_if_open_and_bootstrapped()
            .map_err(|_| super::super::RuntimeWorldOwnerUnavailable::new())?;
        self.state
            .recovery
            .inspect_record(handle)
            .map(ProductUnpublishedOwnerEffects::from_catalog_record)
    }
    fn release_effects(
        &self,
        handle: &ProductUnpublishedRecoveryHandle,
        minimum_age_ticks: u64,
    ) -> Result<Vec<OwnerRetirementWork>, crate::recovery::RuntimeWorldRecoveryDenial> {
        let _operation = self
            .reserve_recovery_operation_if_open_and_bootstrapped()
            .map_err(|_| super::super::RuntimeWorldOwnerUnavailable::new())?;
        let work = self.reserve_recovery_retirement_work(handle)?;
        let released = self
            .state
            .recovery
            .cleanup_record(handle, Some((self.state.clock.now(), minimum_age_ticks)))?;
        Ok(self.drain_released_occurrence(&released, work))
    }
    fn continue_effects(
        &self,
        effects: ProductUnpublishedOwnerEffects,
    ) -> Result<RecoveryContinuationContract, crate::recovery::RuntimeWorldRecoveryDenial> {
        let _operation = self
            .reserve_recovery_operation_if_open_and_bootstrapped()
            .map_err(|_| super::super::ports::RuntimeWorldOwnerUnavailable::new())?;
        let handle = effects.recovery_handle();
        drop(self.state.recovery.inspect_record(&handle)?);
        let settlement_required = effects.progress().relational_requires_settlement();
        if !settlement_required {
            let actions = effects.next_actions().to_vec();
            drop(effects);
            return Ok(RecoveryContinuationContract::new(actions));
        }

        drop(effects);
        let mut update = self.state.recovery.take_record_for_update(&handle)?;
        #[cfg(test)]
        pause_test_recovery_update(&handle);
        let record = update
            .record_mut()
            .ok_or(crate::recovery::RuntimeWorldRecoveryDenial::CallerCapabilityLive)?;
        let mut recovery = record.take_relational_recovery().map_err(|()| {
            crate::recovery::RuntimeWorldRecoveryDenial::SettlementEvidenceUnavailable
        })?;

        if let Some(performed) = recovery.take_performed() {
            match self
                .state
                .relational
                .settlement_port()
                .settle_performed_publication(performed)
            {
                Ok(result)
                    if result.outcome().commit.commit_id
                        == recovery.commit_identity().commit_id() =>
                {
                    recovery
                        .finish(|record, state| record.settle_relational_recovery(state, result));
                }
                Ok(_) => recovery.finish(|record, state| record.retain_identity_repair(state)),
                Err(error) => match error.deferred_settlement() {
                    Some(settlement)
                        if settlement.commit().commit_id
                            == recovery.commit_identity().commit_id() =>
                    {
                        recovery.finish(|record, state| {
                            record.retain_pending_relational_settlement(state, settlement.clone())
                        });
                    }
                    Some(_) | None => {
                        recovery.finish(|record, state| record.retain_identity_repair(state))
                    }
                },
            }
        } else if let Some(settlement) = recovery.settlement().cloned() {
            let performed_result = settlement.performed_result().clone();
            match self
                .state
                .relational
                .settlement_port()
                .repair_deferred_publication_settlement(&settlement)
            {
                Ok(receipt)
                    if receipt == *settlement.commit()
                        && performed_result.outcome().commit == receipt =>
                {
                    recovery.finish(|record, state| {
                        record.settle_relational_recovery(state, performed_result)
                    });
                }
                Ok(_) | Err(_) => drop(recovery),
            }
        } else if let Some(commit_identity) = recovery.take_identity_repair() {
            match self
                .state
                .relational
                .settlement_port()
                .repair_pending_publication_settlement(commit_identity.commit_id())
            {
                Ok(receipt) if receipt.commit_id == commit_identity.commit_id() => {
                    recovery.finish(|record, state| {
                        record.settle_relational_recovery_with_receipt(state, receipt)
                    });
                }
                Ok(_) | Err(_) => {
                    recovery.restore_identity_repair();
                    drop(recovery);
                }
            }
        } else {
            drop(recovery);
            update.finish();
            return Err(crate::recovery::RuntimeWorldRecoveryDenial::SettlementEvidenceUnavailable);
        }

        let actions = record.next_actions().to_vec();
        update.finish();
        Ok(RecoveryContinuationContract::new(actions))
    }
    fn recover_performed(
        &self,
        identity: &crate::identity::CompositeCommitIdentity,
    ) -> Result<
        crate::publication::PerformedCompositePublication,
        crate::recovery::PerformedPublicationRecoveryDenial,
    > {
        let _operation = self
            .reserve_recovery_operation_if_open_and_bootstrapped()
            .map_err(|_| {
                crate::recovery::PerformedPublicationRecoveryDenial::OwnerUnavailable(
                    crate::lifecycle::RuntimeWorldOwnerUnavailable::new(),
                )
            })?;
        self.state
            .history
            .recover_delivery(identity)
            .map(crate::publication::PerformedCompositePublication::owner_issued)
    }
}
