//! Metered predecessor wake disposability, inactive continuation flushing, and
//! safe Bridge clock reclamation.

use worth_query_execution_query::facade::ports::ApplicationRuntimeCoreInternalPort as _;
use worth_query_execution_runtime::facade::ports::AdvancementPhaseInternalPort as _;
use worth_runtime_bridge::facade::BridgeConditionalExecutionView;

use super::super::signal_decision_reentry::{
    WorthQueryRetainedConditionalDecision, WorthQueryRetainedConditionalWake,
};
use super::ErasedClockObservationOutcome;

pub(super) fn resolve_phase_execution<
    'phase,
    Schema: worth_query_installation::facade::ApplicationSchema,
>(
    caller_execution: &worth_execution::ExecutionRequest<'_, '_>,
    phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'phase>,
    runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<
        Schema,
    >,
) -> Result<worth_execution::ExecutionRequest<'phase, 'phase>, ErasedClockObservationOutcome> {
    if let Err(cause) = caller_execution.consult() {
        return Err(ErasedClockObservationOutcome::Failed {
            kind: super::WorthQueryConditionalClockObservationFailureKind::ExecutionRequest(
                crate::domain_computation::primary_graph::application_contribution::advancement_denial(
                    cause,
                ),
            ),
            detail: String::new(),
        });
    }
    phase
        .execution_request_for(runtime.inner.runtime.product_runtime_ref())
        .map_err(|cause| ErasedClockObservationOutcome::Failed {
            kind: super::WorthQueryConditionalClockObservationFailureKind::ExecutionRequest(
                cause.into(),
            ),
            detail: "foreign conditional advancement phase".to_string(),
        })
}

pub(super) fn wakes_disposable_metered(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    wakes: &[WorthQueryRetainedConditionalWake],
) -> bool {
    if wakes.is_empty() {
        return execution.consult().is_ok();
    }
    execution
        .run_native_with_checkpoint(|context| {
            for wake in wakes {
                let stop = if context.has_accounting() {
                    context.checkpoint(1)
                } else {
                    context.poll()
                };
                if stop.is_err() || wake.continuation_identity.is_some() {
                    return false;
                }
                if !matches!(
                    wake.decision,
                    WorthQueryRetainedConditionalDecision::OperationCommitted(_, _)
                        | WorthQueryRetainedConditionalDecision::OperationAlreadyCommitted(_, _)
                        | WorthQueryRetainedConditionalDecision::OperationRetiredNoEffect(_)
                        | WorthQueryRetainedConditionalDecision::OperationProductStale(_, _)
                        | WorthQueryRetainedConditionalDecision::Eligible(_)
                ) {
                    return false;
                }
            }
            true
        })
        .map(|(disposable, _)| disposable)
        .unwrap_or(false)
}

impl<Binding, Reconstruction, Execution, Clock, Input>
    super::WorthQueryInstalledTemporalOperation<Binding, Reconstruction, Execution, Clock, Input>
{
    pub(super) fn prepare_observe_execution<
        'phase,
        Schema: worth_query_installation::facade::ApplicationSchema,
    >(
        &mut self,
        caller_execution: &worth_execution::ExecutionRequest<'_, '_>,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'phase>,
        bridge: &BridgeConditionalExecutionView<'_>,
        runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<
            Schema,
        >,
    ) -> Result<worth_execution::ExecutionRequest<'phase, 'phase>, ErasedClockObservationOutcome>
    {
        let phase_exec = resolve_phase_execution(caller_execution, phase, runtime)?;
        self.flush_and_reclaim_inactive_continuations(&phase_exec, bridge, runtime)
            .map_err(|cause| ErasedClockObservationOutcome::Failed {
                detail: format!("conditional continuation publication refused: {cause:?}"),
                kind: super::WorthQueryConditionalClockObservationFailureKind::Obligation(cause),
            })?;
        Ok(phase_exec)
    }

    pub(super) fn settle_resolved_continuation_wakes(
        &mut self,
        key: [u8; 32],
        intent: [u8; 32],
        finish_confirmed: bool,
        bridge: Option<&BridgeConditionalExecutionView<'_>>,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) {
        // Active-binding wakes keep their decision provenance so the next
        // active observation still reports `AlreadyCommitted` via World, while
        // immediately clearing continuation custody on confirmed `Finish` (or
        // recording `Some(false)` on refused `Finish` so the shared flush path
        // retries `Finish` and never republishes `Retain`).
        for wake in &mut self.retained_wakes {
            if wake
                .continuation_identity
                .is_some_and(|cont| cont.key == key && cont.intent == intent)
            {
                wake.original_retry_finish_confirmed = Some(finish_confirmed);
                if finish_confirmed {
                    wake.continuation_identity = None;
                }
            }
        }
        let mut any_settled_inactive = false;
        for binding in self.inactive_bindings.values_mut() {
            if settle_matching_inactive_wakes(
                &mut binding.retained_wakes,
                key,
                intent,
                finish_confirmed,
            ) {
                any_settled_inactive = true;
            }
        }
        if any_settled_inactive && finish_confirmed {
            self.inactive_reclaim_pending = true;
            if let Some(view) = bridge {
                self.reclaim_disposable_inactive_bindings(execution, view);
            }
        }
    }

    pub(super) fn flush_and_reclaim_inactive_continuations<
        Schema: worth_query_installation::facade::ApplicationSchema,
    >(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        bridge: &BridgeConditionalExecutionView<'_>,
        runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<
            Schema,
        >,
    ) -> Result<(), super::super::super::query_obligations::QueryObligationDenial> {
        let mut any_flushed = false;
        for binding in self.inactive_bindings.values_mut() {
            let mut binding_flushed = false;
            for wake in &mut binding.retained_wakes {
                if wake.continuation_identity.is_some() {
                    let retry_settled = wake.original_retry_finish_confirmed.is_some();
                    super::super::application_operation_reentry::durable_decision::flush_record(
                        runtime, execution, wake,
                    )?;
                    if wake.continuation_identity.is_none() {
                        if retry_settled {
                            promote_indeterminate_to_already_committed(&mut wake.decision);
                        }
                        binding_flushed = true;
                        any_flushed = true;
                    }
                }
            }
            if binding_flushed {
                prune_settled_inactive_wakes(&mut binding.retained_wakes);
            }
        }
        if !any_flushed && !self.inactive_reclaim_pending {
            return Ok(());
        }
        self.reclaim_disposable_inactive_bindings(execution, bridge);
        Ok(())
    }

    fn reclaim_disposable_inactive_bindings(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        bridge: &BridgeConditionalExecutionView<'_>,
    ) {
        let active_affinity = self.active_affinity.as_ref();
        let mut still_pending = false;
        self.inactive_bindings.retain(|_, binding| {
            let same_branch = active_affinity.is_some_and(|active| {
                binding.affinity.is_predecessor_of(active.identity())
                    && binding.affinity.signal_basis().signal_basis().branch_id()
                        == active.signal_basis().signal_basis().branch_id()
            });
            if !(same_branch
                && !binding.pending_invalidations.has_owed()
                && wakes_disposable_metered(execution, &binding.retained_wakes))
            {
                return true;
            }
            if bridge.close_managed_clock(&binding.managed_clock).is_ok() {
                false
            } else {
                still_pending = true;
                true
            }
        });
        self.inactive_reclaim_pending = still_pending;
    }
}

fn promote_indeterminate_to_already_committed(
    decision: &mut WorthQueryRetainedConditionalDecision,
) {
    let prev = std::mem::replace(
        decision,
        WorthQueryRetainedConditionalDecision::InterruptedDuringReentry,
    );
    *decision = match prev {
        WorthQueryRetainedConditionalDecision::OperationIndeterminate(evidence, _) => {
            WorthQueryRetainedConditionalDecision::OperationAlreadyCommitted(evidence, None)
        }
        other => other,
    };
}

fn settle_matching_inactive_wakes(
    wakes: &mut Vec<WorthQueryRetainedConditionalWake>,
    key: [u8; 32],
    intent: [u8; 32],
    finish_confirmed: bool,
) -> bool {
    let mut matched = false;
    for wake in wakes.iter_mut() {
        if wake
            .continuation_identity
            .is_some_and(|cont| cont.key == key && cont.intent == intent)
        {
            matched = true;
            wake.original_retry_finish_confirmed = Some(finish_confirmed);
            promote_indeterminate_to_already_committed(&mut wake.decision);
            if finish_confirmed {
                wake.continuation_identity = None;
            }
        }
    }
    if matched && finish_confirmed {
        prune_settled_inactive_wakes(wakes);
    }
    matched
}

fn prune_settled_inactive_wakes(wakes: &mut Vec<WorthQueryRetainedConditionalWake>) {
    wakes.retain(|wake| {
        wake.continuation_identity.is_some()
            || !matches!(
                wake.decision,
                WorthQueryRetainedConditionalDecision::OperationCommitted(_, _)
                    | WorthQueryRetainedConditionalDecision::OperationAlreadyCommitted(_, _)
                    | WorthQueryRetainedConditionalDecision::OperationRetiredNoEffect(_)
            )
    });
}
