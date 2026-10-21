//! One prepaid finish for a selected execution claim.

use std::sync::Arc;

use super::super::progression::execution_failure_is_retryable;
use super::super::progression::execution_failure_reschedules;
use super::super::required_work::RequiredWorkMembership;
use super::super::DemandRecord;
use super::super::DemandState;
use super::super::WorthQueryOutputCheckpoint;
use super::super::WorthQueryOutputDemandInterest;
use super::super::WorthQueryOutputDemandRegistry;
use super::super::WorthQueryOutputProgress;

use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandRecoveryPosture;

/// Prepared before Running and consumed by exactly one execution outcome.
pub(in crate::domain_computation::primary_graph) struct PreparedSelectedExecutionFinish<'a> {
    registry: &'a WorthQueryOutputDemandRegistry,
    key: Arc<super::super::WorthQueryOutputDemandKey>,
    member: Arc<RequiredWorkMembership>,
    publication_head: Arc<std::sync::atomic::AtomicUsize>,
}

/// Holds only this invocation's Running claim. Abandoning a prepared suffix
/// releases it back to Scheduled before its last Interest can be destroyed.
pub(in crate::domain_computation::primary_graph) struct RunningSelectedExecution<'a> {
    finish: Option<PreparedSelectedExecutionFinish<'a>>,
}
impl<'a> RunningSelectedExecution<'a> {
    pub(in crate::domain_computation::primary_graph) fn new(
        finish: PreparedSelectedExecutionFinish<'a>,
    ) -> Self {
        Self {
            finish: Some(finish),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn relinquish(mut self) {
        self.finish.take().expect("one Running finish").relinquish();
    }
    pub(in crate::domain_computation::primary_graph) fn publish(
        mut self,
        checkpoint: WorthQueryOutputCheckpoint,
    ) -> Result<(), (WorthQueryOutputDemandDenial, WorthQueryOutputCheckpoint)> {
        self.finish
            .take()
            .expect("one Running finish")
            .publish(checkpoint)
    }
    pub(in crate::domain_computation::primary_graph) fn failure(
        mut self,
        denial: &mut WorthQueryOutputDemandDenial,
    ) {
        self.finish
            .take()
            .expect("one Running finish")
            .failure(denial);
    }
}
impl Drop for RunningSelectedExecution<'_> {
    fn drop(&mut self) {
        if let Some(finish) = self.finish.take() {
            finish.relinquish();
        }
    }
}

impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn prepare_selected_execution_finish<'a>(
        &'a self,
        interest: &WorthQueryOutputDemandInterest,
    ) -> Result<PreparedSelectedExecutionFinish<'a>, WorthQueryOutputDemandDenial> {
        // The existing ordered lookup owner may allocate one subject of at
        // most 55 bytes. A refusal of this first envelope stays allocation free.

        // The largest later phase takes two record descents, one fixed
        // checkpoint/state move, an empty retained denial, and an Arc key
        // retain for deferred terminal cleanup. Its variable diagnostic
        // stays with the immediate caller.

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

        // Other demands may grow the tree during execution. Its installed
        // finite record allowance bounds the finish lookup and, on terminal
        // failure, deferred-cleanup lookup before any effects.
        let record = state
            .records
            .get(&interest.key)
            .expect("a live selected Interest retains its record");
        let member = record
            .work_membership
            .as_ref()
            .cloned()
            .ok_or_else(empty_work_denial)?;
        let publication_head = state
            .records
            .publication_counter(record.product_occurrence)?;
        Ok(PreparedSelectedExecutionFinish {
            registry: self,
            key: Arc::clone(member.key_arc()),
            member,
            publication_head,
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
        let mut record = state
            .records
            .get_mut(self.key.as_ref())
            .expect("the selected executor retains its Interest");
        if record
            .work_membership
            .as_ref()
            .is_some_and(|member| Arc::ptr_eq(member, &self.member))
        {
            relinquish_record(&mut record);
        }
        record.finish();
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
        let Some(mut record) = state.records.get_mut(self.key.as_ref()) else {
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
        let (mut output, retired_observation) =
            WorthQueryOutputProgress::new_with_detached_observation(checkpoint);
        output.bind_publication_head(self.publication_head);
        record.state = DemandState::Output(output);
        let retired_source = record.performed_source.take();
        record.successor_of = None;
        record.wake.notify();
        record.finish();
        drop(state);
        drop(retired_source);
        drop(retired_observation);
        Ok(())
    }

    /// Return the complete producer stop to its typed demand owner. The row
    /// stores diagnostics only; failed managed cleanup is never cloned here.
    /// Retired delivery obligations are detached under the guard and dropped afterward.
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
        let Some(mut record) = state.records.get_mut(self.key.as_ref()) else {
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
        let obligations = if rescheduled {
            record.leave_refresh_unclaimed(DemandState::Scheduled);
            Vec::new()
        } else {
            record.state = DemandState::Failed(retained_denial(denial));
            std::mem::take(&mut record.performed_obligations)
        };
        record.wake.notify();
        record.finish();
        if !rescheduled {
            state.defer_terminal_cleanup(self.member.key_arc());
        }
        drop(state);
        drop(obligations);
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
