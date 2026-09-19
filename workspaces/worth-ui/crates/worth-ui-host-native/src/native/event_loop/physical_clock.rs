use std::time::{Duration, Instant};

use winit::event_loop::ControlFlow;

#[derive(Clone)]
/// Read-only time in the native event loop's input-observation clock domain.
pub struct UiNativeObservationClock {
    epoch: Instant,
}

impl UiNativeObservationClock {
    pub fn sample_millis(&self) -> u64 {
        elapsed_millis(self.epoch.elapsed())
    }

    /// A deliberately simulated elapsed epoch for deterministic boundary tests.
    /// Sampling uses the native implementation; this does not certify OS timer delivery.
    #[cfg(feature = "certification-support")]
    #[doc(hidden)]
    pub fn from_certification_elapsed(millis: u64) -> Option<Self> {
        let epoch = Instant::now().checked_sub(Duration::from_millis(millis))?;
        Some(UiNativePhysicalEventClock { epoch }.observation_clock())
    }
}

pub(super) struct UiNativePhysicalEventClock {
    epoch: Instant,
}

impl UiNativePhysicalEventClock {
    pub(super) fn new() -> Self {
        Self {
            epoch: Instant::now(),
        }
    }

    pub(super) fn observation_clock(&self) -> UiNativeObservationClock {
        UiNativeObservationClock { epoch: self.epoch }
    }

    pub(super) fn current_tick(&self) -> u64 {
        elapsed_millis(self.epoch.elapsed())
    }

    pub(super) fn deadline(&self, tick: u64) -> Option<Instant> {
        self.epoch.checked_add(Duration::from_millis(tick))
    }
}

pub(super) fn tighten_deadline(current: ControlFlow, deadline: Instant) -> ControlFlow {
    match current {
        ControlFlow::Poll => ControlFlow::Poll,
        ControlFlow::Wait => ControlFlow::WaitUntil(deadline),
        ControlFlow::WaitUntil(current) => ControlFlow::WaitUntil(current.min(deadline)),
    }
}

fn elapsed_millis(elapsed: Duration) -> u64 {
    elapsed.as_millis().min(u128::from(u64::MAX)) as u64
}

#[cfg(test)]
mod tests {
    use super::{elapsed_millis, tighten_deadline, UiNativePhysicalEventClock};
    use std::time::{Duration, Instant};
    use winit::event_loop::ControlFlow;

    #[test]
    fn observation_reader_and_deadline_retain_the_physical_input_epoch() {
        // Fixed owner epoch isolates conversion/issuance from OS scheduling.
        let epoch = Instant::now() - Duration::from_secs(17);
        let physical = UiNativePhysicalEventClock { epoch };
        let before = physical.current_tick();
        let reader = physical.observation_clock();
        let observed = reader.sample_millis();
        let after = physical.current_tick();
        assert!(before <= observed && observed <= after);
        assert!(
            observed >= 17_000,
            "issuing a reader must not reset the epoch"
        );
        assert_eq!(
            physical.deadline(observed),
            Some(epoch + Duration::from_millis(observed))
        );
    }

    #[test]
    fn physical_signal_deadline_only_tightens_event_loop_waiting() {
        let now = Instant::now();
        let early = now + Duration::from_millis(8);
        let late = now + Duration::from_millis(13);
        assert_eq!(
            tighten_deadline(ControlFlow::Poll, early),
            ControlFlow::Poll
        );
        assert_eq!(
            tighten_deadline(ControlFlow::Wait, early),
            ControlFlow::WaitUntil(early)
        );
        assert_eq!(
            tighten_deadline(ControlFlow::WaitUntil(late), early),
            ControlFlow::WaitUntil(early)
        );
        assert_eq!(
            tighten_deadline(ControlFlow::WaitUntil(early), late),
            ControlFlow::WaitUntil(early)
        );
        assert_eq!(elapsed_millis(Duration::from_micros(7_999)), 7);
        assert_eq!(elapsed_millis(Duration::from_millis(8)), 8);
    }
}
