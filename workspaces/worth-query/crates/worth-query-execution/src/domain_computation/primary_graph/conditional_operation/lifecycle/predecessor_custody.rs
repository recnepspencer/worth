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

    pub(super) fn settle_resolved_continuation_wakes(&mut self, key: [u8; 32], intent: [u8; 32]) {
        // Active-binding wakes remain until the next active observation reports
        // their `AlreadyCommitted` provenance; superseded predecessor bindings
        // in `inactive_bindings` are never observed again and settle
        // immediately so they cannot re-record `Retain` on a later pump.
        for binding in self.inactive_bindings.values_mut() {
            settle_matching_wakes(&mut binding.retained_wakes, key, intent);
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
                if let Some(cont) = wake.continuation_identity {
                    if matches!(
                        wake.decision,
                        WorthQueryRetainedConditionalDecision::OperationIndeterminate(_, _)
                    ) && runtime
                        .installed_program_support()
                        .is_some_and(|support| !support.lifecycle().is_indeterminate(&cont.key))
                    {
                        wake.continuation_identity = None;
                        let prev = std::mem::replace(
                            &mut wake.decision,
                            WorthQueryRetainedConditionalDecision::InterruptedDuringReentry,
                        );
                        if let WorthQueryRetainedConditionalDecision::OperationIndeterminate(
                            evidence,
                            _,
                        ) = prev
                        {
                            wake.decision =
                                WorthQueryRetainedConditionalDecision::OperationAlreadyCommitted(
                                    evidence, None,
                                );
                        } else {
                            wake.decision = prev;
                        }
                        binding_flushed = true;
                        any_flushed = true;
                        continue;
                    }
                    super::super::application_operation_reentry::durable_decision::flush_record(
                        runtime, execution, wake,
                    )?;
                    if wake.continuation_identity.is_none() {
                        binding_flushed = true;
                        any_flushed = true;
                    }
                }
            }
            if binding_flushed {
                binding.retained_wakes.retain(|wake| {
                    wake.continuation_identity.is_some()
                        || !matches!(
                            wake.decision,
                            WorthQueryRetainedConditionalDecision::OperationCommitted(_, _)
                                | WorthQueryRetainedConditionalDecision::OperationAlreadyCommitted(
                                    _,
                                    _
                                )
                                | WorthQueryRetainedConditionalDecision::OperationRetiredNoEffect(
                                    _
                                )
                        )
                });
            }
        }
        if !any_flushed {
            return Ok(());
        }
        let active_affinity = self.active_affinity.as_ref();
        self.inactive_bindings.retain(|_, binding| {
            let same_branch = active_affinity.is_some_and(|active| {
                binding.affinity.is_predecessor_of(active.identity())
                    && binding.affinity.signal_basis().signal_basis().branch_id()
                        == active.signal_basis().signal_basis().branch_id()
            });
            !(same_branch
                && binding.pending_invalidations.retained_delivery_count() == 0
                && wakes_disposable_metered(execution, &binding.retained_wakes)
                && bridge.close_managed_clock(&binding.managed_clock).is_ok())
        });
        Ok(())
    }
}

fn settle_matching_wakes(
    wakes: &mut Vec<WorthQueryRetainedConditionalWake>,
    key: [u8; 32],
    intent: [u8; 32],
) {
    for wake in wakes.iter_mut() {
        if wake
            .continuation_identity
            .is_some_and(|cont| cont.key == key && cont.intent == intent)
        {
            wake.continuation_identity = None;
            let prev = std::mem::replace(
                &mut wake.decision,
                WorthQueryRetainedConditionalDecision::InterruptedDuringReentry,
            );
            wake.decision = match prev {
                WorthQueryRetainedConditionalDecision::OperationIndeterminate(evidence, _) => {
                    WorthQueryRetainedConditionalDecision::OperationAlreadyCommitted(evidence, None)
                }
                other => other,
            };
        }
    }
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
