//! One prepaid finish for a selected execution claim.

use std::sync::Arc;

use super::super::{
    progression::{execution_failure_is_retryable, execution_failure_reschedules},
    required_work::RequiredWorkMembership,
    DemandRecord, DemandState, WorthQueryOutputCheckpoint, WorthQueryOutputDemandInterest,
    WorthQueryOutputDemandRegistry, WorthQueryOutputProgress,
};
use crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission;
use crate::domain_computation::primary_graph::{
    WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenialKind,
    WorthQueryOutputDemandRecoveryPosture,
};

/// Prepared before Running and consumed by exactly one execution outcome.
pub(in crate::domain_computation::primary_graph) struct PreparedSelectedExecutionFinish<'a> {
    registry: &'a WorthQueryOutputDemandRegistry,
    interest: &'a WorthQueryOutputDemandInterest,
    member: Arc<RequiredWorkMembership>,
}

impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn prepare_selected_execution_finish<'a>(
        &'a self,
        interest: &'a WorthQueryOutputDemandInterest,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<PreparedSelectedExecutionFinish<'a>, WorthQueryOutputDemandDenial> {
        // The existing ordered lookup owner may allocate one subject of at
        // most 55 bytes. A refusal of this first envelope stays allocation free.
        let terminal = 55 + std::mem::size_of::<String>();
        admission
            .charge_external_work(terminal as u64)
            .map_err(|_| empty_work_denial())?;
        admission
            .admit_read_scratch(terminal as u64)
            .map_err(empty_admission_denial)?;
        // The largest later phase takes two record descents, one fixed
        // checkpoint/state move, an empty retained denial, and an Arc key
        // retain for deferred terminal cleanup. Its variable diagnostic
        // stays with the immediate caller.
        let fixed = 2 * std::mem::size_of::<PreparedSelectedExecutionFinish<'_>>()
            + 2 * std::mem::size_of::<DemandState>()
            + std::mem::size_of::<WorthQueryOutputProgress>()
            + std::mem::size_of::<WorthQueryOutputDemandDenial>()
            + std::mem::size_of::<WorthQueryOutputCheckpoint>()
            + 2 * std::mem::size_of::<Arc<RequiredWorkMembership>>()
            + 64;
        admission
            .charge_external_work(fixed as u64)
            .map_err(|_| empty_work_denial())?;
        if !Arc::ptr_eq(&self.state, &interest.owner.state) {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignDemand,
                "",
            ));
        }
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.charge_record_lookup(&interest.key, admission)?;
        // Other demands may grow the tree during execution. Its installed
        // finite record allowance bounds the finish lookup and, on terminal
        // failure, deferred-cleanup lookup before any effects.
        state.charge_maximum_record_lookup(&interest.key, admission)?;
        state.charge_maximum_record_lookup(&interest.key, admission)?;
        let record = state
            .records
            .get(&interest.key)
            .expect("a live selected Interest retains its record");
        let member = record
            .work_membership
            .as_ref()
            .cloned()
            .ok_or_else(empty_work_denial)?;
        Ok(PreparedSelectedExecutionFinish {
            registry: self,
            interest,
            member,
        })
    }
}

impl PreparedSelectedExecutionFinish<'_> {
    pub(in crate::domain_computation::primary_graph) fn relinquish(self) {
        let mut state = self
            .registry
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let record = state
            .records
            .get_mut(&self.interest.key)
            .expect("the selected executor retains its Interest");
        if record
            .work_membership
            .as_ref()
            .is_some_and(|member| Arc::ptr_eq(member, &self.member))
        {
            relinquish_record(record);
        }
        drop(state);
        // The retained member is destroyed after the registry guard.
        drop(self);
    }

    /// The checkpoint stays with the caller if another owner moved this
    /// exact row before installation. The registry never clones its receipt.
    pub(in crate::domain_computation::primary_graph) fn publish(
        self,
        checkpoint: WorthQueryOutputCheckpoint,
    ) -> Result<(), (WorthQueryOutputDemandDenial, WorthQueryOutputCheckpoint)> {
        let mut state = self
            .registry
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(record) = state.records.get_mut(&self.interest.key) else {
            return Err((
                empty_denial(WorthQueryOutputDemandDenialKind::Closed),
                checkpoint,
            ));
        };
        if !record
            .work_membership
            .as_ref()
            .is_some_and(|member| Arc::ptr_eq(member, &self.member))
            || !matches!(record.state, DemandState::Running)
        {
            let denial = match &record.state {
                DemandState::Failed(denial) => retained_denial(denial),
                _ => empty_denial(WorthQueryOutputDemandDenialKind::SchedulingRejected),
            };
            return Err((denial, checkpoint));
        }
        let (output, retired_observation) =
            WorthQueryOutputProgress::new_with_detached_observation(checkpoint);
        record.state = DemandState::Output(output);
        let retired_source = record.performed_source.take();
        record.successor_of = None;
        record.wake.notify();
        drop(state);
        drop(retired_source);
        drop(retired_observation);
        Ok(())
    }

    /// Keep the complete denial with the caller. The registry stores only its
    /// typed kind and posture; all terminal custody is detached under the
    /// guard and destroyed afterward.
    pub(in crate::domain_computation::primary_graph) fn failure(
        self,
        denial: &mut WorthQueryOutputDemandDenial,
    ) {
        let retryable = execution_failure_is_retryable(denial);
        let rescheduled = execution_failure_reschedules(denial);
        denial.recovery_posture = if retryable {
            WorthQueryOutputDemandRecoveryPosture::Retryable
        } else {
            WorthQueryOutputDemandRecoveryPosture::Terminal
        };
        let mut state = self
            .registry
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(record) = state.records.get_mut(&self.interest.key) else {
            return;
        };
        if !record
            .work_membership
            .as_ref()
            .is_some_and(|member| Arc::ptr_eq(member, &self.member))
            || !matches!(record.state, DemandState::Running)
        {
            return;
        }
        let (released_bytes, obligations) = if rescheduled {
            record.leave_refresh_unclaimed(DemandState::Scheduled);
            (0, Vec::new())
        } else {
            record.state = DemandState::Failed(retained_denial(denial));
            let bytes = record.obligation_reserved_bytes();
            (bytes, std::mem::take(&mut record.performed_obligations))
        };
        record.wake.notify();
        if !rescheduled {
            state.defer_terminal_cleanup(self.member.key_arc(), 0);
        }
        drop(state);
        drop(obligations);
        if released_bytes != 0 {
            let mut state = self
                .registry
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            state.obligation_reserved_bytes = state
                .obligation_reserved_bytes
                .saturating_sub(released_bytes);
        }
    }
}

fn empty_denial(kind: WorthQueryOutputDemandDenialKind) -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(kind, "")
}

fn retained_denial(source: &WorthQueryOutputDemandDenial) -> WorthQueryOutputDemandDenial {
    let mut retained = empty_denial(source.kind());
    retained.recovery_posture = source.recovery_posture();
    retained
}

pub(super) fn relinquish_record(record: &mut DemandRecord) {
    let unclaimed = match record.state {
        DemandState::Scheduling => DemandState::Admitted,
        DemandState::Running => DemandState::Scheduled,
        _ => return,
    };
    record.leave_refresh_unclaimed(unclaimed);
    record.wake.notify();
}

fn empty_work_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(WorthQueryOutputDemandDenialKind::WorkBudgetExceeded, "")
}

fn empty_admission_denial(
    stop: worth_relational::facade::mvcc::CompanionPreflightStop,
) -> WorthQueryOutputDemandDenial {
    use worth_relational::facade::mvcc::CompanionPreflightStop as Stop;
    let kind = match stop {
        Stop::WorkExhausted { .. } | Stop::WorkCounterOverflow => {
            WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
        }
        _ => WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
    };
    WorthQueryOutputDemandDenial::new(kind, "")
}
