use std::{
    collections::VecDeque,
    sync::{Arc, Condvar, Mutex, Weak},
};

use super::PhysicalSubmissionState;
use crate::physical_runtime::{
    PhysicalSignalAspectBindingDigest, PhysicalWorkConsumerHandle, PhysicalWorkIdentity,
};

pub(in crate::physical_runtime) trait PhysicalWorkAbandonmentWake:
    Send + Sync
{
    fn wake(&self);
}

#[derive(Clone)]
pub(in crate::physical_runtime) struct PhysicalWorkAbandonmentPublisher {
    shared: Arc<PhysicalWorkAbandonmentQueue>,
}

pub(in crate::physical_runtime) struct PhysicalWorkAbandonmentInbox {
    shared: Arc<PhysicalWorkAbandonmentQueue>,
}

pub(in crate::physical_runtime) struct PhysicalWorkAbandonment {
    state: Weak<PhysicalSubmissionState>,
    queue: Weak<PhysicalWorkAbandonmentQueue>,
    identity: PhysicalWorkIdentity,
    route: PhysicalSignalAspectBindingDigest,
    consumer: Option<PhysicalWorkConsumerHandle>,
    release: Arc<super::super::command_storage::PhysicalCommandRelease>,
    completed: bool,
}

struct PhysicalWorkAbandonmentQueue {
    capacity: usize,
    state: Mutex<PhysicalWorkAbandonmentQueueState>,
    settled: Condvar,
    #[cfg(feature = "certification-test-authority")]
    wait_observed: Condvar,
    wake: Arc<dyn PhysicalWorkAbandonmentWake>,
}

struct PhysicalWorkAbandonmentQueueState {
    pending: VecDeque<PhysicalWorkAbandonment>,
    outstanding: usize,
    accepting: bool,
    #[cfg(feature = "certification-test-authority")]
    await_idle_observed_outstanding: usize,
}

#[cfg(feature = "certification-test-authority")]
#[derive(Clone)]
pub struct CertificationPhysicalAbandonmentWaitObserver {
    shared: Arc<PhysicalWorkAbandonmentQueue>,
}

#[cfg(feature = "certification-test-authority")]
impl CertificationPhysicalAbandonmentWaitObserver {
    pub fn await_observed_outstanding(&self, expected: usize) -> bool {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        let mut state = self
            .shared
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        while state.await_idle_observed_outstanding < expected {
            let Some(remaining) = deadline.checked_duration_since(std::time::Instant::now()) else {
                return false;
            };
            let (next, wait) = self
                .shared
                .wait_observed
                .wait_timeout(state, remaining)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state = next;
            if wait.timed_out() && state.await_idle_observed_outstanding < expected {
                return false;
            }
        }
        true
    }
}

pub(in crate::physical_runtime) fn physical_work_abandonment_channel(
    capacity: usize,
    wake: Arc<dyn PhysicalWorkAbandonmentWake>,
) -> (
    PhysicalWorkAbandonmentPublisher,
    PhysicalWorkAbandonmentInbox,
) {
    let shared = Arc::new(PhysicalWorkAbandonmentQueue {
        capacity,
        state: Mutex::new(PhysicalWorkAbandonmentQueueState {
            pending: VecDeque::with_capacity(capacity),
            outstanding: 0,
            accepting: true,
            #[cfg(feature = "certification-test-authority")]
            await_idle_observed_outstanding: 0,
        }),
        settled: Condvar::new(),
        #[cfg(feature = "certification-test-authority")]
        wait_observed: Condvar::new(),
        wake,
    });
    (
        PhysicalWorkAbandonmentPublisher {
            shared: Arc::clone(&shared),
        },
        PhysicalWorkAbandonmentInbox { shared },
    )
}

impl PhysicalWorkAbandonmentPublisher {
    pub(super) fn publish(
        &self,
        state: &Arc<PhysicalSubmissionState>,
        identity: PhysicalWorkIdentity,
        registration: super::super::command_storage::PhysicalCommandSignalRegistration,
        release: Arc<super::super::command_storage::PhysicalCommandRelease>,
    ) -> bool {
        let mut queue_state = self
            .shared
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if !queue_state.accepting || queue_state.outstanding >= self.shared.capacity {
            return false;
        }
        queue_state.outstanding += 1;
        queue_state.pending.push_back(PhysicalWorkAbandonment {
            state: Arc::downgrade(state),
            queue: Arc::downgrade(&self.shared),
            identity,
            route: registration.route,
            consumer: registration.consumer,
            release,
            completed: false,
        });
        drop(queue_state);
        self.shared.wake.wake();
        true
    }

    pub(super) fn await_idle(&self) {
        let mut state = self
            .shared
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        #[cfg(feature = "certification-test-authority")]
        if state.outstanding != 0 {
            state.await_idle_observed_outstanding += 1;
            self.shared.wait_observed.notify_all();
        }
        while state.outstanding != 0 {
            state = self
                .shared
                .settled
                .wait(state)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
    }

    #[cfg(feature = "certification-test-authority")]
    pub(super) fn wait_observer(&self) -> CertificationPhysicalAbandonmentWaitObserver {
        CertificationPhysicalAbandonmentWaitObserver {
            shared: Arc::clone(&self.shared),
        }
    }
}

impl PhysicalWorkAbandonmentInbox {
    pub(in crate::physical_runtime) fn pop(&self) -> Option<PhysicalWorkAbandonment> {
        self.shared
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .pending
            .pop_front()
    }
}

impl Drop for PhysicalWorkAbandonmentInbox {
    fn drop(&mut self) {
        let pending = {
            let mut state = self
                .shared
                .state
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            state.accepting = false;
            state.pending.drain(..).collect::<Vec<_>>()
        };
        drop(pending);
    }
}

impl PhysicalWorkAbandonment {
    pub(in crate::physical_runtime) const fn identity(&self) -> PhysicalWorkIdentity {
        self.identity
    }

    pub(in crate::physical_runtime) const fn route(&self) -> PhysicalSignalAspectBindingDigest {
        self.route
    }

    pub(in crate::physical_runtime) const fn consumer(&self) -> Option<PhysicalWorkConsumerHandle> {
        self.consumer
    }

    pub(in crate::physical_runtime) fn complete(mut self) {
        self.finish(false, true);
    }

    fn release_command(&self, revoke_signal: bool, signal_joined: bool) {
        let Some(state) = self.state.upgrade() else {
            return;
        };
        if revoke_signal {
            state.signal_admission.revoke();
        }
        if self.release.complete_abandonment() {
            state.release_abandoned(self.identity, signal_joined);
        }
    }

    fn finish(&mut self, revoke_signal: bool, signal_joined: bool) {
        if self.completed {
            return;
        }
        let Some(queue) = self.queue.upgrade() else {
            self.release_command(revoke_signal, signal_joined);
            self.completed = true;
            return;
        };
        queue.complete_one_after(|| self.release_command(revoke_signal, signal_joined));
        self.completed = true;
    }
}

impl Drop for PhysicalWorkAbandonment {
    fn drop(&mut self) {
        if self.completed {
            return;
        }
        self.finish(true, false);
    }
}

impl PhysicalWorkAbandonmentQueue {
    fn complete_one_after(&self, release: impl FnOnce()) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        release();
        self.complete_one(&mut state);
    }

    fn complete_one(&self, state: &mut PhysicalWorkAbandonmentQueueState) {
        if state.outstanding == 0 {
            return;
        }
        state.outstanding -= 1;
        if state.outstanding == 0 {
            self.settled.notify_all();
        }
    }
}

impl PhysicalSubmissionState {
    pub(super) fn release_abandoned(&self, identity: PhysicalWorkIdentity, signal_joined: bool) {
        let Some(released) = self.commands.release(identity) else {
            return;
        };
        self.release_capacity(released.scope_members, released.semantic_bytes);
        self.accounting
            .record_terminal(released.operation, released.pressure);
        if !released.retry_pending
            && matches!(
                released.stage,
                crate::physical_runtime::PhysicalWorkTerminalStage::Dispatched
                    | crate::physical_runtime::PhysicalWorkTerminalStage::Settling
            )
        {
            self.terminal_ledger.record(
                crate::physical_runtime::work::PhysicalWorkTerminalEvent::AbandonedAfterDispatch(
                    identity,
                ),
            );
        } else {
            self.terminal_ledger.record(
                crate::physical_runtime::work::PhysicalWorkTerminalEvent::ReleasedBeforeDispatch {
                    identity,
                    consumer: if signal_joined {
                        None
                    } else {
                        released.consumer
                    },
                },
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{mpsc, Arc};

    use super::{physical_work_abandonment_channel, PhysicalWorkAbandonmentWake};

    struct NoopWake;

    impl PhysicalWorkAbandonmentWake for NoopWake {
        fn wake(&self) {}
    }

    #[test]
    fn completion_holds_queue_occupancy_until_capacity_release_is_recorded() {
        let wake: Arc<dyn PhysicalWorkAbandonmentWake> = Arc::new(NoopWake);
        let (publisher, _inbox) = physical_work_abandonment_channel(1, wake);
        publisher.shared.state.lock().unwrap().outstanding = 1;
        let queue = Arc::clone(&publisher.shared);
        let completion_queue = Arc::clone(&queue);
        let (release_started, release_observed) = mpsc::sync_channel(0);
        let (continue_release, release_waiting) = mpsc::sync_channel(0);
        let completion = std::thread::spawn(move || {
            completion_queue.complete_one_after(|| {
                release_started.send(()).unwrap();
                release_waiting.recv().unwrap();
            });
        });

        release_observed.recv().unwrap();
        assert!(queue.state.try_lock().is_err());
        continue_release.send(()).unwrap();
        completion.join().unwrap();
        assert_eq!(queue.state.lock().unwrap().outstanding, 0);
    }
}
