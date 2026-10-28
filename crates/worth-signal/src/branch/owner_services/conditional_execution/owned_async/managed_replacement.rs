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
        let queue = self.custodied_queue(owner)?;
        let handoff = self
            .with_owned_async_cell(&target.basis, |cell, admission| {
                cell.advance_conditional_managed_owned_async_retry(
                    admission,
                    &target.basis,
                    &self.definition,
                    target.node,
                    owner.admitted.handle(),
                    queue,
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
        let queue = self.custodied_queue(owner)?;
        let handoff = self
            .with_owned_async_cell(&target.basis, |cell, admission| {
                cell.revalidate_conditional_managed_owned_async_request(
                    admission,
                    &target.basis,
                    &self.definition,
                    target.node,
                    owner.admitted.handle(),
                    queue,
                    intent,
                    owner.process_local,
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
        let clears_retry = matches!(
            cleanup,
            SignalOwnedAsyncManagedWakeCleanup::ReadyRetry { .. }
                | SignalOwnedAsyncManagedWakeCleanup::RetiredRetry { .. }
        );
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
        let binding = owner.origin.clone();
        self.release_pending_process_local_retention(&binding, owner)
            .map_err(|cause| match cause {
                SignalOwnedAsyncQueueDenial::Service(cause) => cause,
                _ => SignalConditionalServiceExecutionDenial::DefinitionMismatch,
            })?;
        if clears_retry {
            owner.pending_retry = None;
        }
        Ok(())
    }

    fn require_managed_replacement(
        &self,
        target: &SignalOwnedAsyncSourceBinding,
        owner: &mut SignalOwnedAsyncManagedAdmission,
    ) -> Result<(), SignalOwnedAsyncQueueDenial> {
        self.require_owned_async_binding(target)
            .map_err(SignalOwnedAsyncQueueDenial::Service)?;
        self.require_compatible_owned_async_source(target, &owner.origin)
            .map_err(SignalOwnedAsyncQueueDenial::Service)?;
        if owner.requires_queue && owner.queue.is_none() {
            return Err(SignalOwnedAsyncQueueDenial::QueueUnavailable);
        }
        if owner.cleanup.is_some() {
            return Err(SignalOwnedAsyncQueueDenial::CleanupPending);
        }
        self.release_pending_process_local_retention(target, owner)
    }

    fn custodied_queue<'owner>(
        &self,
        owner: &'owner SignalOwnedAsyncManagedAdmission,
    ) -> Result<
        Option<&'owner crate::data::resource::ResourceManagedQueueBinding>,
        SignalOwnedAsyncQueueDenial,
    > {
        if owner.requires_queue && owner.queue.is_none() {
            return Err(SignalOwnedAsyncQueueDenial::QueueUnavailable);
        }
        Ok(owner.queue.as_ref().map(|queue| &queue.binding))
    }

    pub(super) fn require_compatible_owned_async_source(
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
