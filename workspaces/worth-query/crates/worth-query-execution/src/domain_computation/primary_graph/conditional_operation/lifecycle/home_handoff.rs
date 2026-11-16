//! Graceful handoff releases actual live clocks after retaining continuation data.
use super::WorthQueryInstalledTemporalOperation;
use crate::domain_computation::primary_graph::{
    WorthQueryHomeRetirementDenial as Denial, WorthQueryHomeRetirementOwner as Owner,
    WorthQueryPrimaryGraphApplicationRuntime,
};

impl<Binding, Reconstruction, Execution, Clock, Input>
    WorthQueryInstalledTemporalOperation<Binding, Reconstruction, Execution, Clock, Input>
{
    pub(super) fn handoff_to_home<Schema: worth_query_installation::facade::ApplicationSchema>(
        &mut self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    ) -> Result<(), Denial> {
        for wake in self.retained_wakes.iter_mut().chain(
            self.inactive_bindings
                .values_mut()
                .flat_map(|binding| binding.retained_wakes.iter_mut()),
        ) {
            prepare_wake(request, runtime, wake)?;
        }
        let owner = runtime.runtime.bridge.conditional_operations();
        let bridge = owner
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let (Some(affinity), Some(clock)) = (&self.active_affinity, &self.managed_clock) {
            release_clock(&bridge, affinity, clock)?;
        }
        self.managed_clock = None;
        self.active_affinity = None;
        let inactive = self.inactive_bindings.keys().cloned().collect::<Vec<_>>();
        for key in inactive {
            let binding = self
                .inactive_bindings
                .get(&key)
                .expect("original inactive binding");
            release_clock(&bridge, &binding.affinity, &binding.managed_clock)?;
            self.inactive_bindings.remove(&key);
        }
        self.retained_wakes.clear();
        self.inactive_bindings.clear();
        self.reconstructed_intents.clear();
        self.commit_watch = Default::default();
        Ok(())
    }
}

fn prepare_wake<Schema: worth_query_installation::facade::ApplicationSchema>(
    request: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    wake: &mut super::super::signal_decision_reentry::WorthQueryRetainedConditionalWake,
) -> Result<(), Denial> {
    use super::super::signal_decision_reentry::WorthQueryRetainedConditionalDecision as Decision;
    request.consult().map_err(Denial::Execution)?;
    if let Decision::OperationCleanupRequired(_, cleanup) = &mut wake.decision {
        cleanup.retry_for_home(request);
    }
    super::super::application_operation_reentry::durable_decision::flush_record(
        runtime, request, wake,
    )
    .map_err(Denial::ManagedCleanup)?;
    if let Decision::OperationCleanupRequired(_, cleanup) = &wake.decision
        && cleanup.pending_kind().is_some()
    {
        use crate::domain_computation::primary_graph::query_obligations::QueryObligationDenial;
        let identity = cleanup.obligation_identity().ok_or(Denial::ManagedCleanup(
            QueryObligationDenial::UnknownObligation,
        ))?;
        return Err(Denial::ManagedCleanup(
            QueryObligationDenial::CleanupReleaseUnrecorded { identity },
        ));
    }
    match &wake.decision {
        Decision::Eligible(_)
        | Decision::Suppressed(_)
        | Decision::Deferred(_)
        | Decision::OperationBackpressured(_, _)
        | Decision::OperationCommitted(_)
        | Decision::OperationAlreadyCommitted(_) => Ok(()),
        Decision::OperationRetryable(_, _)
        | Decision::OperationCommitRetryable(_, _)
        | Decision::OperationProductStale(_, _) => match wake.continuation_identity {
            Some(_) => Ok(()),
            None => Err(Denial::Unsupported(Owner::ConditionalRuntime)),
        },
        Decision::OperationCleanupRequired(_, cleanup) => {
            if cleanup.logical_discharged() || wake.continuation_identity.is_some() {
                Ok(())
            } else {
                Err(Denial::Unsupported(Owner::ConditionalRuntime))
            }
        }
        Decision::OperationPreparationPending(_, _)
        | Decision::OperationPreparationRefused(_, _)
        | Decision::OperationProductUnpublished(_, _, _)
        | Decision::OperationNoEffect(_, _)
        | Decision::OperationExecutionControlRetryable(_, _)
        | Decision::OperationTerminalFailure(_, _)
        | Decision::OperationControlStopped(_, _)
        | Decision::OperationIndeterminate(_, _)
        | Decision::InterruptedDuringReentry
        | Decision::Failed(_) => Err(Denial::Unsupported(Owner::ConditionalRuntime)),
    }
}

fn release_clock(
    bridge: &crate::domain_computation::primary_graph::managed_bridge::conditional_execution::ConditionalExecutionOwner<'_>,
    affinity: &super::evaluation_affinity::WorthQueryConditionalEvaluationAffinity,
    clock: &worth_runtime_bridge::facade::BridgeManagedClockBinding,
) -> Result<(), Denial> {
    if !clock.is_live() {
        return Ok(());
    }
    bridge
        .with_view(affinity.signal_basis().signal_basis(), |view| {
            view.close_managed_clock(clock)
        })
        .map_err(|cause| Denial::ConditionalBinding(cause.kind()))?
        .map_err(|cause| Denial::ConditionalClock(cause.kind()))?;
    Ok(())
}
