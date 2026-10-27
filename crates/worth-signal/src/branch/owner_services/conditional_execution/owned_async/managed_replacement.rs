use super::*;

impl<D, I, T> SignalConditionalExecutionPort<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn advance_managed_owned_async_request_to_timeout(
        &self,
        owner: &mut SignalOwnedAsyncManagedAdmission,
        coordinate: u64,
    ) -> Result<SignalOwnedAsyncManagedProgress, SignalOwnedAsyncQueueDenial> {
        let binding = owner.origin.clone();
        self.require_managed_replacement(&binding, owner)?;
        let handoff = self
            .with_owned_async_cell(&binding.basis, |cell, admission| {
                cell.advance_conditional_managed_owned_async_timeout(
                    admission,
                    &binding.basis,
                    &self.definition,
                    binding.node,
                    owner.admitted.handle(),
                    coordinate,
                )
            })
            .map_err(SignalOwnedAsyncQueueDenial::Service)?;
        Ok(owner.retain_handoff(&binding, handoff))
    }

    pub fn schedule_managed_owned_async_retry(
        &self,
        owner: &mut SignalOwnedAsyncManagedAdmission,
    ) -> Result<SignalOwnedAsyncRetrySchedule, SignalOwnedAsyncQueueDenial> {
        let binding = owner.origin.clone();
        self.require_managed_replacement(&binding, owner)?;
        let handoff = self
            .with_owned_async_cell(&binding.basis, |cell, admission| {
                cell.schedule_conditional_managed_owned_async_retry(
                    admission,
                    &binding.basis,
                    &self.definition,
                    binding.node,
                    owner.admitted.handle(),
                )
            })
            .map_err(SignalOwnedAsyncQueueDenial::Service)?;
        let SignalOwnedAsyncManagedReport::RetrySchedule(report) = &handoff.report else {
            unreachable!("the native schedule handoff retains its actual report")
        };
        let schedule = SignalOwnedAsyncRetrySchedule::new(binding.clone(), report.clone());
        owner.retain_handoff(&binding, handoff);
        Ok(schedule)
    }

    pub fn advance_managed_owned_async_retry(
        &self,
        target: &SignalOwnedAsyncSourceBinding,
        owner: &mut SignalOwnedAsyncManagedAdmission,
        schedule: &SignalOwnedAsyncRetrySchedule,
        coordinate: u64,
    ) -> Result<SignalOwnedAsyncManagedProgress, SignalOwnedAsyncQueueDenial> {
        self.require_managed_replacement(target, owner)?;
        self.require_compatible_owned_async_source(target, &schedule.origin)
            .map_err(SignalOwnedAsyncQueueDenial::Service)?;
        let queue = owner
            .queue
            .as_ref()
            .ok_or(SignalOwnedAsyncQueueDenial::QueueUnavailable)?;
        let handoff = self
            .with_owned_async_cell(&target.basis, |cell, admission| {
                cell.advance_conditional_managed_owned_async_retry(
                    admission,
                    &target.basis,
                    &self.definition,
                    target.node,
                    &queue.binding,
                    schedule,
                    coordinate,
                )
            })
            .map_err(SignalOwnedAsyncQueueDenial::Service)??;
        // No fallible assembly separates native progress from retained custody.
        Ok(owner.retain_handoff(target, handoff))
    }

    pub fn revalidate_managed_owned_async_request(
        &self,
        target: &SignalOwnedAsyncSourceBinding,
        owner: &mut SignalOwnedAsyncManagedAdmission,
        intent: ResourceRevalidationIntent,
    ) -> Result<SignalOwnedAsyncManagedProgress, SignalOwnedAsyncQueueDenial> {
        self.require_managed_replacement(target, owner)?;
        let queue = owner
            .queue
            .as_ref()
            .ok_or(SignalOwnedAsyncQueueDenial::QueueUnavailable)?;
        let handoff = self
            .with_owned_async_cell(&target.basis, |cell, admission| {
                cell.revalidate_conditional_managed_owned_async_request(
                    admission,
                    &target.basis,
                    &self.definition,
                    target.node,
                    &queue.binding,
                    intent,
                )
            })
            .map_err(SignalOwnedAsyncQueueDenial::Service)??;
        Ok(owner.retain_handoff(target, handoff))
    }

    pub fn finish_managed_owned_async_wake_cleanup(
        &self,
        owner: &mut SignalOwnedAsyncManagedAdmission,
    ) -> Result<(), SignalConditionalServiceExecutionDenial> {
        self.require_owned_async_binding(&owner.origin)?;
        let Some(cleanup) = &mut owner.cleanup else {
            return Ok(());
        };
        let binding = &owner.origin;
        self.with_owned_async_cell(&binding.basis, |cell, admission| {
            cell.finish_conditional_managed_owned_async_wake_cleanup(
                admission,
                &binding.basis,
                &self.definition,
                binding.node,
                cleanup,
            )
        })?;
        owner.cleanup = None;
        owner.refusal = None;
        Ok(())
    }

    fn require_managed_replacement(
        &self,
        target: &SignalOwnedAsyncSourceBinding,
        owner: &SignalOwnedAsyncManagedAdmission,
    ) -> Result<(), SignalOwnedAsyncQueueDenial> {
        self.require_owned_async_binding(target)
            .map_err(SignalOwnedAsyncQueueDenial::Service)?;
        self.require_compatible_owned_async_source(target, &owner.origin)
            .map_err(SignalOwnedAsyncQueueDenial::Service)?;
        if owner.cleanup.is_some() {
            return Err(SignalOwnedAsyncQueueDenial::CleanupPending);
        }
        Ok(())
    }

    fn require_compatible_owned_async_source(
        &self,
        target: &SignalOwnedAsyncSourceBinding,
        prior: &SignalOwnedAsyncSourceBinding,
    ) -> Result<(), SignalConditionalServiceExecutionDenial> {
        let owner = SignalOwner::upgrade(&self.owner)
            .map_err(SignalConditionalServiceExecutionDenial::OwnerUnavailable)?;
        if !owner.basis_has_owner_affinity(&prior.basis)
            || prior.incarnation != self.incarnation
            || prior.basis.owner_branch_id() != target.basis.owner_branch_id()
            || prior.node != target.node
            || !prior.definition.is_claimed_by(&self.claimant)
        {
            return Err(SignalConditionalServiceExecutionDenial::DefinitionMismatch);
        }
        Ok(())
    }
}
