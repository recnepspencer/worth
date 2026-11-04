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
impl SignalBranchRetentionBinding {
    pub(crate) fn assert_busy_retirement_preserves_ledger(
        &self,
        original: &mut crate::branch::SignalBranchRetentionRetirement,
        request: &ExecutionRequest<'_, '_>,
    ) {
        let guard = self.ledger.state.lock().unwrap();
        let before = self.terminal_counts();
        assert_eq!(original.advance(request), Err(Denial::Busy));
        assert_eq!(self.terminal_counts(), before);
        drop(guard);
    }
}
