use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

/// Requested delay before offering the next Motion tick.
///
/// This leaves nominal headroom within the qualified 60 Hz display interval.
/// It is not a deadline guarantee: timer rounding, scheduling, preparation,
/// host submission and compositor presentation can all delay visible progress.
/// Only the native active-scroll traces qualify end-to-end cadence.
const MOTION_WAKE_PERIOD: Duration = Duration::from_millis(8);

pub(super) struct UiNativeMotionReadinessLane {
    schedule: Arc<(Mutex<UiNativeMotionReadinessSchedule>, Condvar)>,
    worker: Option<std::thread::JoinHandle<()>>,
}

#[derive(Default)]
struct UiNativeMotionReadinessSchedule {
    deadline: Option<Instant>,
    closed: bool,
}

impl UiNativeMotionReadinessLane {
    pub(super) fn start(
        port: worth_ui_host_native::UiNativeApplicationReadinessPort,
    ) -> Result<Self, ()> {
        let schedule = Arc::new((
            Mutex::new(UiNativeMotionReadinessSchedule::default()),
            Condvar::new(),
        ));
        let worker_schedule = Arc::clone(&schedule);
        let worker = std::thread::Builder::new()
            .name("worth-ui-motion-readiness".into())
            .spawn(move || {
                run_motion_readiness_worker(worker_schedule, move || {
                    port.signal().map(|_| ()).map_err(|_| ())
                })
            })
            .map_err(|_| ())?;
        Ok(Self {
            schedule,
            worker: Some(worker),
        })
    }

    pub(super) fn arm_now(&self) {
        self.arm_at(Instant::now());
    }

    pub(super) fn arm_next_frame(&self) {
        self.arm_at(Instant::now() + MOTION_WAKE_PERIOD);
    }

    fn arm_at(&self, deadline: Instant) {
        let (state, wake) = &*self.schedule;
        let Ok(mut state) = state.lock() else {
            return;
        };
        if state.closed {
            return;
        }
        if state.deadline.is_none_or(|current| deadline < current) {
            state.deadline = Some(deadline);
            wake.notify_one();
        }
    }

    pub(super) fn shutdown(&mut self) {
        let (state, wake) = &*self.schedule;
        if let Ok(mut state) = state.lock() {
            state.closed = true;
            state.deadline = None;
            wake.notify_one();
        }
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

impl Drop for UiNativeMotionReadinessLane {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn run_motion_readiness_worker(
    schedule: Arc<(Mutex<UiNativeMotionReadinessSchedule>, Condvar)>,
    mut signal: impl FnMut() -> Result<(), ()>,
) {
    let (state, wake) = &*schedule;
    let Ok(mut state) = state.lock() else {
        return;
    };
    loop {
        if state.closed {
            return;
        }
        let Some(deadline) = state.deadline else {
            let Ok(next) = wake.wait(state) else {
                return;
            };
            state = next;
            continue;
        };
        let now = Instant::now();
        if now < deadline {
            let Ok((next, _)) = wake.wait_timeout(state, deadline.duration_since(now)) else {
                return;
            };
            state = next;
            continue;
        }
        state.deadline = None;
        drop(state);
        if signal().is_err() {
            return;
        }
        let Ok(next) = state_lock(&schedule) else {
            return;
        };
        state = next;
    }
}

#[cfg(test)]
impl UiNativeMotionReadinessLane {
    pub(super) fn start_for_test(
        mut signal: impl FnMut() -> Result<(), ()> + Send + 'static,
    ) -> Self {
        let schedule = Arc::new((
            Mutex::new(UiNativeMotionReadinessSchedule::default()),
            Condvar::new(),
        ));
        let worker_schedule = Arc::clone(&schedule);
        let worker = std::thread::Builder::new()
            .name("worth-ui-motion-readiness-test".into())
            .spawn(move || run_motion_readiness_worker(worker_schedule, &mut signal))
            .unwrap();
        Self {
            schedule,
            worker: Some(worker),
        }
    }
}

fn state_lock(
    schedule: &Arc<(Mutex<UiNativeMotionReadinessSchedule>, Condvar)>,
) -> Result<std::sync::MutexGuard<'_, UiNativeMotionReadinessSchedule>, ()> {
    schedule.0.lock().map_err(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    /// One 60 Hz display interval, the milestone's qualification display.
    const QUALIFICATION_DISPLAY_INTERVAL: Duration = Duration::from_micros(16_667);

    /// Check the requested delay only; this cannot prove delivered cadence.
    #[test]
    fn the_requested_wake_period_is_at_most_half_a_qualification_frame() {
        assert!(MOTION_WAKE_PERIOD < QUALIFICATION_DISPLAY_INTERVAL);
        assert!(MOTION_WAKE_PERIOD
            .checked_mul(2)
            .is_some_and(|two_wakes| two_wakes <= QUALIFICATION_DISPLAY_INTERVAL));
    }

    #[test]
    fn arms_coalesce_to_one_signal_and_shutdown_joins_the_worker() {
        let (signal_sender, signals) = mpsc::channel();
        let mut lane = UiNativeMotionReadinessLane::start_for_test(move || {
            signal_sender.send(()).map_err(|_| ())
        });
        let deadline = Instant::now() + Duration::from_millis(40);
        lane.arm_at(deadline);
        lane.arm_at(deadline + Duration::from_millis(20));
        lane.arm_at(deadline + Duration::from_millis(10));
        signals
            .recv_timeout(Duration::from_secs(1))
            .expect("the earliest coalesced deadline should signal");
        assert!(matches!(
            signals.recv_timeout(Duration::from_millis(80)),
            Err(mpsc::RecvTimeoutError::Timeout)
        ));

        lane.shutdown();
        lane.arm_now();
        assert!(matches!(
            signals.recv_timeout(Duration::from_millis(40)),
            Err(mpsc::RecvTimeoutError::Disconnected)
        ));
        assert!(lane.worker.is_none());
    }
}
