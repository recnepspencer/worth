use std::time::{Duration, Instant};

use super::{PlatformPulseApplicationRuntime, PlatformPulseTerminalError};

// Product policy: allow ten elapsed seconds for one native frame attempt.
// Callback frequency and unrelated publications cannot consume this allowance.
const PRESENTATION_ALLOWANCE: Duration = Duration::from_secs(10);

impl PlatformPulseApplicationRuntime {
    pub(super) fn sample_frame_time(&mut self) -> Option<(u64, u64)> {
        match frame_time(self.frame_time_origin, Instant::now()) {
            Some(reading) => Some(reading),
            None => {
                self.fail(
                    PlatformPulseTerminalError::FrameExecution(
                        "monotonic presentation clock is unrepresentable".to_owned(),
                    ),
                    Ok(()),
                );
                None
            }
        }
    }
}

fn frame_time(origin: Instant, now: Instant) -> Option<(u64, u64)> {
    let elapsed = now.checked_duration_since(origin)?;
    let tick = u64::try_from(elapsed.as_millis()).ok()?;
    let allowance = u64::try_from(PRESENTATION_ALLOWANCE.as_millis()).ok()?;
    Some((tick, tick.checked_add(allowance)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asynchronous_completion_uses_elapsed_time_not_callback_count() {
        let origin = Instant::now();
        let (submitted, deadline) = frame_time(origin, origin).unwrap();
        assert_eq!(submitted, 0);
        assert_eq!(deadline, 10_000);
        for _ in 0..100_000 {
            assert_eq!(frame_time(origin, origin).unwrap().0, submitted);
        }
        let completed = frame_time(origin, origin + Duration::from_millis(250))
            .unwrap()
            .0;
        assert!(
            completed < deadline,
            "a ready frame may finish asynchronously"
        );
        assert_eq!(
            frame_time(origin, origin + Duration::from_millis(9_999))
                .unwrap()
                .0,
            deadline - 1
        );
        assert_eq!(
            frame_time(origin, origin + Duration::from_secs(10))
                .unwrap()
                .0,
            deadline
        );
        assert!(frame_time(origin, origin - Duration::from_millis(1)).is_none());
    }
}
