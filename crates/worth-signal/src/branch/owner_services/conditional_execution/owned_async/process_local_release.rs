use super::*;

/// Actual terminal evidence after exact process-local Native records expired.
/// This value has no source binding, queue or request progression capability.
pub struct SignalOwnedAsyncReleasedResource {
    terminal: InFlightResourceRequest,
    report: SignalOwnedAsyncManagedReport,
    completion: Option<crate::data::resource::ResourceCompletionCommitReport>,
    predecessor: Option<InFlightResourceRequest>,
    retry: Option<super::process_local_retry::SignalOwnedAsyncReleasedRetry>,
}

pub struct SignalOwnedAsyncResourceReleaseRefusal {
    owner: SignalOwnedAsyncManagedAdmission,
    completion: Option<SignalOwnedAsyncCompletionCommit>,
    cause: SignalOwnedAsyncQueueDenial,
}

impl SignalOwnedAsyncReleasedResource {
    pub fn terminal(&self) -> &InFlightResourceRequest {
        &self.terminal
    }

    pub fn report(&self) -> &SignalOwnedAsyncManagedReport {
        &self.report
    }

    pub fn completion_report(
        &self,
    ) -> Option<&crate::data::resource::ResourceCompletionCommitReport> {
        self.completion.as_ref()
    }

    pub fn released_predecessor(&self) -> Option<&InFlightResourceRequest> {
        self.predecessor.as_ref()
    }

    pub fn released_retry(
        &self,
    ) -> Option<&super::process_local_retry::SignalOwnedAsyncReleasedRetry> {
        self.retry.as_ref()
    }
}

impl SignalOwnedAsyncResourceReleaseRefusal {
    pub fn cause(&self) -> &SignalOwnedAsyncQueueDenial {
        &self.cause
    }

    pub fn into_parts(
        self,
    ) -> (
        SignalOwnedAsyncManagedAdmission,
        Option<SignalOwnedAsyncCompletionCommit>,
        SignalOwnedAsyncQueueDenial,
    ) {
        (self.owner, self.completion, self.cause)
    }
}

impl<D, I, T> SignalConditionalExecutionPort<D, I, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    /// Only this installed, queued posture can later release its own records.
    pub fn admit_process_local_managed_owned_async_request(
        &self,
        binding: &SignalOwnedAsyncSourceBinding,
        queue_capacity: u64,
    ) -> Result<SignalOwnedAsyncManagedAdmission, SignalConditionalServiceExecutionDenial> {
        self.require_owned_async_binding(binding)?;
        let state = self.with_owned_async_cell(&binding.basis, |cell, admission| {
            cell.admit_conditional_process_local_managed_owned_async(
                admission,
                &binding.basis,
                &self.definition,
                binding.node,
                queue_capacity,
            )
        })?;
        let mut owner =
            SignalOwnedAsyncManagedAdmission::from_initial(binding.clone(), state, true);
        owner.process_local = true;
        Ok(owner)
    }

    pub fn release_process_local_managed_request(
        &self,
        owner: SignalOwnedAsyncManagedAdmission,
        completion: Option<SignalOwnedAsyncCompletionCommit>,
    ) -> Result<SignalOwnedAsyncReleasedResource, SignalOwnedAsyncResourceReleaseRefusal> {
        let result = self.prepare_and_release_process_local(&owner, completion.as_ref());
        match result {
            Ok((terminal, predecessor)) => Ok(SignalOwnedAsyncReleasedResource {
                terminal,
                report: owner.report,
                completion: completion.map(|value| value.state.report),
                predecessor: predecessor.or(owner.released_predecessor),
                retry: owner.released_retry,
            }),
            Err(cause) => Err(SignalOwnedAsyncResourceReleaseRefusal {
                owner,
                completion,
                cause,
            }),
        }
    }

    fn prepare_and_release_process_local(
        &self,
        owner: &SignalOwnedAsyncManagedAdmission,
        completion: Option<&SignalOwnedAsyncCompletionCommit>,
    ) -> Result<
        (InFlightResourceRequest, Option<InFlightResourceRequest>),
        SignalOwnedAsyncQueueDenial,
    > {
        use SignalOwnedAsyncQueueDenial as Denial;
        let binding = &owner.origin;
        self.require_owned_async_binding(binding)
            .map_err(Denial::Service)?;
        if !owner.process_local || !owner.requires_queue || !owner.admitted_current_occurrence {
            return Err(Denial::Service(
                SignalConditionalServiceExecutionDenial::DefinitionMismatch,
            ));
        }
        if owner.cleanup.is_some()
            || completion.is_some_and(|value| !value.timeout_cleanup_complete())
        {
            return Err(Denial::CleanupPending);
        }
        if let Some(queue) = &owner.queue {
            let depth = self
                .observe_owned_async_safe_point(queue)?
                .pressure()
                .queue_depth();
            if depth != 0 {
                return Err(Denial::QueueNotDrained { depth });
            }
        }
        if let Some(committed) = completion {
            self.require_owned_async_binding(&committed.origin)
                .map_err(Denial::Service)?;
            self.require_compatible_owned_async_source(binding, &committed.origin)
                .map_err(Denial::Service)?;
            if committed.state.handle != owner.admitted.handle() {
                return Err(Denial::Service(
                    SignalConditionalServiceExecutionDenial::DefinitionMismatch,
                ));
            }
        }
        self.with_owned_async_cell(&binding.basis, |cell, admission| {
            cell.release_conditional_process_local_owned_async(
                admission,
                &binding.basis,
                &self.definition,
                binding.node,
                owner.admitted.handle(),
                completion.map(|value| value.state.handle),
                owner.retention.as_ref(),
            )
        })
        .map_err(Denial::Service)
    }

    /// Disposes only the actual denied admission's stored diagnostic record;
    /// the live request and its cleanup capability remain untouched.
    pub fn release_process_local_completion_denial(
        &self,
        completion: &SignalOwnedAsyncCompletionAdmission,
    ) -> Result<(), SignalConditionalServiceExecutionDenial> {
        let binding = &completion.origin;
        self.require_owned_async_binding(binding)?;
        let denied = completion
            .report
            .denied_completion()
            .ok_or(SignalConditionalServiceExecutionDenial::DefinitionMismatch)?;
        self.with_owned_async_cell(&binding.basis, |cell, admission| {
            cell.release_conditional_process_local_owned_async_denial(
                admission,
                &binding.basis,
                &self.definition,
                binding.node,
                denied,
            )
        })
    }
}
