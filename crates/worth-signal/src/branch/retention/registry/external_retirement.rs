//! Retain original ledger keys while explicitly retiring their live obligation.
use super::*;
use crate::branch::SignalBranchRetentionRetirementDenial as Denial;
use worth_execution::{ExecutionRequest, WorkCeilingDenial};
#[derive(Debug)]
pub(crate) struct RetiredSignalExternalObligation {
    _record: Option<SignalExternalObligationRecord>,
    _target_counter: Option<(SignalRetainedTargetKey, u32)>,
}
impl SignalBranchRetentionBinding {
    pub(crate) fn try_terminate_external(
        &self,
        lease_id: u64,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<RetiredSignalExternalObligation, Denial> {
        let mut state = self.ledger.state.try_lock().map_err(|_| Denial::Busy)?;
        let units = state.external_leases.get(&lease_id).map_or(1, |record| {
            1 + record.target.0.domain().len() as u64 + record.target.0.bytes().len() as u64
        });
        let (charged, _) = request
            .run_native_with_checkpoint(|context| {
                if context.has_accounting() {
                    context.checkpoint(units)
                } else {
                    context.poll()
                }
            })
            .map_err(Denial::Execution)?;
        charged.map_err(|cause| Denial::Execution(WorkCeilingDenial::Stopped(cause)))?;
        let posture = if state.owner_open {
            SignalBranchRetentionOwnerPosture::Live
        } else {
            SignalBranchRetentionOwnerPosture::Lost
        };
        let record = state.external_leases.remove(&lease_id);
        let target_counter = record.as_ref().and_then(|record| {
            decrement_obligation_count(&mut state.external_count_by_branch, &record.branch_id);
            let count = state.external_count_by_target.get_mut(&record.target)?;
            *count = count.saturating_sub(1);
            if *count == 0 {
                state.external_count_by_target.remove_entry(&record.target)
            } else {
                None
            }
        });
        self.ledger.terminality.record(
            SignalRetentionTerminalCause::ExplicitRelease,
            posture,
            record.is_some(),
        );
        Ok(RetiredSignalExternalObligation {
            _record: record,
            _target_counter: target_counter,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::branch::SignalBranchRetentionRetirementProgress;
    use crate::facade::{SignalGraph, SignalRuntime};
    use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};
    #[test]
    fn original_signal_lease_survives_busy_and_stop_then_retires_once_without_drop_release() {
        let mut runtime: SignalRuntime<(), (), (), (), ()> =
            SignalRuntime::builder(SignalGraph::new())
                .with_kernel_defaults()
                .build();
        let basis = runtime
            .observe_signal_branch_basis(runtime.current_branch())
            .unwrap();
        let identity = basis
            .observation()
            .target()
            .as_basis()
            .unwrap()
            .graph_instance_id()
            .parse()
            .unwrap();
        let registry = SignalBranchRetentionRegistry::new(identity);
        let lease = registry.acquire_external(&basis).unwrap();
        let before = registry.terminal_counts();
        let mut original = lease.into_retirement();
        let live = SerialRequest::from_memory(
            SerialMemoryBudget::new(None),
            CancellationSource::new().token(),
            None,
        );
        let request = ExecutionRequest::serial(&live);
        let guard = registry.ledger.state.lock().unwrap();
        assert_eq!(original.advance(&request), Err(Denial::Busy));
        assert_eq!(registry.terminal_counts(), before);
        drop(guard);
        let cancellation = CancellationSource::new();
        cancellation.cancel();
        let stopped =
            SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
        assert!(matches!(
            original.advance(&ExecutionRequest::serial(&stopped)),
            Err(Denial::Execution(_))
        ));
        assert_eq!(registry.test_observation().external_lease_count, 1);
        assert_eq!(registry.terminal_counts(), before);
        assert_eq!(
            original.advance(&request),
            Ok(SignalBranchRetentionRetirementProgress::Complete)
        );
        assert_eq!(registry.test_observation().external_lease_count, 0);
        let after = registry.terminal_counts();
        assert_eq!(
            original.advance(&ExecutionRequest::serial(&stopped)),
            Ok(SignalBranchRetentionRetirementProgress::Complete)
        );
        assert_eq!(registry.terminal_counts(), after);
        original.into_retired().unwrap().reclaim();
        assert_eq!(registry.terminal_counts(), after);
    }
}

