use std::time::{Duration, Instant};

use crate::external_observation::ProcessBoundNativeClientAreaObservation;

use super::NativePlatformFailure;

const SETTLEMENT_DEADLINE: Duration = Duration::from_secs(2);
const SAMPLE_INTERVAL: Duration = Duration::from_millis(16);
const REQUIRED_STABLE_SAMPLES: u8 = 3;

pub(super) fn await_client_stability(
    observed: ProcessBoundNativeClientAreaObservation,
) -> Result<(), NativePlatformFailure> {
    let deadline = Instant::now() + SETTLEMENT_DEADLINE;
    let mut prior = capture(observed)?;
    let mut stable_samples = 0;
    loop {
        std::thread::sleep(SAMPLE_INTERVAL);
        let current = capture(observed)?;
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

fn capture(
    observed: ProcessBoundNativeClientAreaObservation,
) -> Result<Vec<u8>, NativePlatformFailure> {
    Ok(
        super::gdi_capture::capture_client_area(observed.bounds(), observed.process_id())?
            .rgba()
            .to_vec(),
    )
}
