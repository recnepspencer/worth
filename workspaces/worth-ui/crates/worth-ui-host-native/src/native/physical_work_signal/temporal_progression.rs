use std::num::NonZeroUsize;
use worth_signal::facade::{
    ClockAdvanceRequest, ClockDomain, ClockTick, ReadyTemporalWake, ResourceRequestHandle,
    ResourceRetryReason,
};

use super::routing::UiNativePhysicalSignalWork;
use super::worker::UiNativePhysicalSignalWorker;
use super::worker_graph::UiNativePhysicalSignalPerformed;
use super::UiNativePhysicalSignalOwner;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum UiNativePhysicalTemporalTransition {
    TimeoutScheduled {
        work: UiNativePhysicalSignalWork,
    },
    TimeoutTerminal {
        work: UiNativePhysicalSignalWork,
        handle: ResourceRequestHandle,
    },
    RetryAdmitted {
        work: UiNativePhysicalSignalWork,
        previous: ResourceRequestHandle,
        successor: ResourceRequestHandle,
    },
    PollReady {
        work: UiNativePhysicalSignalWork,
    },
}

pub(super) struct UiNativePhysicalPerformedTemporalTransition {
    transition: UiNativePhysicalTemporalTransition,
    performed: UiNativePhysicalSignalPerformed,
}

impl UiNativePhysicalSignalWorker {
    pub(super) fn next_due_tick(&self) -> Option<u64> {
        self.graph
            .runtime
            .temporal_frontier_snapshot()
            .next_due_tick()
            .map(ClockTick::get)
    }

    pub(super) fn advance_clock_to(
        &mut self,
        tick: u64,
    ) -> Result<Box<[UiNativePhysicalPerformedTemporalTransition]>, ()> {
        if tick < self.graph.context.clock_revision {
            return Err(());
        }
        if tick > self.graph.context.clock_revision {
            self.graph
                .runtime
                .advance_clock(ClockAdvanceRequest::new(
                    ClockDomain::MonotonicExecution,
                    ClockTick::new(tick),
                ))
                .map_err(|_| ())?;
            self.graph.context.clock_revision = tick;
        }
        let selection_limit = NonZeroUsize::new(self.requests.len().max(1)).ok_or(())?;
        let promotion = self
            .graph
            .runtime
            .promote_due_temporal_wakes_ready_bounded(selection_limit)
            .map_err(|_| ())?;
        let mut transitions = Vec::new();
        for ready in promotion.promotion().ready_wakes() {
            if self.admit_ready_poll(ready.clone(), &mut transitions)? {
                continue;
            }
            if self.admit_ready_retry(ready.clone(), &mut transitions)? {
                continue;
            }
            if self.admit_ready_timeout(ready.clone(), &mut transitions)? {
                continue;
            }
            return Err(());
        }
        let mut performed_transitions = Vec::with_capacity(transitions.len());
        for transition in transitions {
            let work = match transition {
                UiNativePhysicalTemporalTransition::TimeoutScheduled { work }
                | UiNativePhysicalTemporalTransition::TimeoutTerminal { work, .. }
                | UiNativePhysicalTemporalTransition::RetryAdmitted { work, .. }
                | UiNativePhysicalTemporalTransition::PollReady { work } => work,
            };
            let operation = self
                .requests
                .iter()
                .find(|request| request.work == work)
                .map(|request| request.operation)
                .ok_or(())?;
            let performed = self.graph.perform_transition(operation, work)?;
            performed_transitions.push(UiNativePhysicalPerformedTemporalTransition {
                transition,
                performed,
            });
        }
        Ok(performed_transitions.into_boxed_slice())
    }

    fn admit_ready_poll(
        &mut self,
        ready: ReadyTemporalWake,
        transitions: &mut Vec<UiNativePhysicalTemporalTransition>,
    ) -> Result<bool, ()> {
        let Some(index) = self
            .requests
            .iter()
            .position(|request| request.poll_wake == Some(ready.id()))
        else {
            return Ok(false);
        };
        self.graph
            .runtime
            .retire_temporal_wake(
                ready.id(),
                worth_signal::facade::TemporalWakeRetirementReason::Consumed,
            )
            .map_err(|_| ())?;
        self.requests[index].poll_wake = None;
        transitions.push(UiNativePhysicalTemporalTransition::PollReady {
            work: self.requests[index].work,
        });
        Ok(true)
    }

    fn admit_ready_retry(
        &mut self,
        ready: ReadyTemporalWake,
        transitions: &mut Vec<UiNativePhysicalTemporalTransition>,
    ) -> Result<bool, ()> {
        let wake = ready.id();
        let Some(index) = self
            .requests
            .iter()
            .position(|request| request.retry_wake == Some(wake))
        else {
            return Ok(false);
        };
        let previous = self.requests[index].handle;
        let report = self
            .graph
            .runtime
            .admit_scheduled_resource_retry(previous, ready)
            .map_err(|_| ())?;
        let admitted = report.admitted_retry().ok_or(())?.admitted_request();
        if !self.graph.replace_current_handle(
            self.requests[index].work,
            previous,
            admitted.handle(),
        ) {
            return Err(());
        }
        self.requests[index].handle = admitted.handle();
        self.requests[index].attempt = admitted.attempt();
        self.requests[index].retry_wake = None;
        transitions.push(UiNativePhysicalTemporalTransition::RetryAdmitted {
            work: self.requests[index].work,
            previous,
            successor: admitted.handle(),
        });
        Ok(true)
    }

    fn admit_ready_timeout(
        &mut self,
        ready: ReadyTemporalWake,
        transitions: &mut Vec<UiNativePhysicalTemporalTransition>,
    ) -> Result<bool, ()> {
        let wake = ready.id();
        let Some(index) = self.requests.iter().position(|request| {
            self.graph
                .runtime
                .in_flight_resource_request(request.handle)
                .and_then(|in_flight| in_flight.timeout_wake_id())
                == Some(wake)
        }) else {
            return Ok(false);
        };
        let request = self.requests[index];
        let timeout = self
            .graph
            .runtime
            .admit_resource_timeout(request.handle, ready)
            .map_err(|_| ())?;
        if timeout.timed_out_request().is_none() {
            return Err(());
        }
        let retry = self
            .graph
            .runtime
            .schedule_resource_retry(request.handle, ResourceRetryReason::TimedOut)
            .map_err(|_| ())?;
        if let Some(scheduled) = retry.scheduled_retry() {
            self.requests[index].retry_wake = Some(scheduled.backoff_wake_id());
            transitions
                .push(UiNativePhysicalTemporalTransition::TimeoutScheduled { work: request.work });
        } else {
            if let Some(poll_wake) = self.requests[index].poll_wake {
                self.graph
                    .runtime
                    .retire_temporal_wake(
                        poll_wake,
                        worth_signal::facade::TemporalWakeRetirementReason::Cancelled,
                    )
                    .map_err(|_| ())?;
                self.requests[index].poll_wake = None;
            }
            transitions.push(UiNativePhysicalTemporalTransition::TimeoutTerminal {
                work: request.work,
                handle: request.handle,
            });
        }
        Ok(true)
    }
}

impl UiNativePhysicalSignalOwner {
    pub(crate) fn next_due_tick(&self) -> Option<u64> {
        self.worker
            .as_ref()
            .and_then(UiNativePhysicalSignalWorker::next_due_tick)
    }

    pub(crate) fn advance_clock_to(&mut self, tick: u64) -> Result<(), ()> {
        let transitions = self.worker_mut()?.advance_clock_to(tick)?;
        for transition in transitions {
            match transition.transition {
                UiNativePhysicalTemporalTransition::TimeoutScheduled { work } => {
                    self.counters.timeout_observations =
                        self.counters.timeout_observations.saturating_add(1);
                    self.counters.retry_schedules = self.counters.retry_schedules.saturating_add(1);
                    self.wake.remove(work);
                }
                UiNativePhysicalTemporalTransition::TimeoutTerminal { work, handle } => {
                    self.counters.timeout_observations =
                        self.counters.timeout_observations.saturating_add(1);
                    let owner_handle = self.route.owner_handle(work).ok_or(())?;
                    let token = self
                        .route
                        .token_for(self.runtime_identity, work)
                        .map_err(|_| ())?;
                    if token.handle() != handle {
                        return Err(());
                    }
                    if !self.worker_mut()?.retire_timed_out_handle(handle)
                        || !self.route.remove(token)
                    {
                        return Err(());
                    }
                    self.admit_recovery_work(work)?;
                    if !self.route.adopt_owner_handle(work, owner_handle) {
                        return Err(());
                    }
                    self.wake.request(work);
                }
                UiNativePhysicalTemporalTransition::RetryAdmitted {
                    work,
                    previous,
                    successor,
                } => {
                    let token = self
                        .route
                        .token_for(self.runtime_identity, work)
                        .map_err(|_| ())?;
                    if token.handle() != previous || !self.route.replace_handle(token, successor) {
                        return Err(());
                    }
                    if transition.performed.work() != work {
                        return Err(());
                    }
                    self.publish_performed(transition.performed)?;
                }
                UiNativePhysicalTemporalTransition::PollReady { work } => {
                    if transition.performed.work() != work {
                        return Err(());
                    }
                    self.publish_performed(transition.performed)?;
                }
            }
        }
        Ok(())
    }
}
