//! Before a button is delivered the client's pixels must have stopped
//! changing, so a pointer-hover repaint is not mistaken for the click's
//! effect. Neutral over any observer: the caller supplies its own capture.
use std::time::{Duration, Instant};

use super::NativePlatformFailure;

const SETTLEMENT_DEADLINE: Duration = Duration::from_secs(2);
const SAMPLE_INTERVAL: Duration = Duration::from_millis(16);
const REQUIRED_STABLE_SAMPLES: u8 = 3;

pub(in crate::native_platform) fn await_client_stability(
    capture: impl Fn() -> Result<Vec<u8>, NativePlatformFailure>,
) -> Result<(), NativePlatformFailure> {
    let deadline = Instant::now() + SETTLEMENT_DEADLINE;
    let mut prior = capture()?;
    let mut stable_samples = 0;
    loop {
        std::thread::sleep(SAMPLE_INTERVAL);
        let current = capture()?;
        if current == prior {
            stable_samples += 1;
            if stable_samples == REQUIRED_STABLE_SAMPLES {
                return Ok(());
            }
        } else {
            stable_samples = 0;
            prior = current;
        }
        if Instant::now() >= deadline {
            return Err(NativePlatformFailure::InputDelivery(
                "pointer appearance did not settle before activation".to_owned(),
            ));
        }
    }
}
