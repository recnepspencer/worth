use std::time::{Duration, Instant};

use crate::external_observation::NativeClientPixelCapture;

use super::NativePlatformFailure;

const EXPOSURE_DEADLINE: Duration = Duration::from_secs(2);
const COMPOSITION_INTERVAL: Duration = Duration::from_millis(16);

/// Raising a client area to the top of the z-order asks the compositor to
/// present it; it does not make it presented. Until that composition reaches
/// the desktop the monitor source still reports whatever covered the client
/// area beforehand, while the window source already reports the raised
/// content, so the two disagree for a reason that has nothing to do with the
/// pixels under adjudication. This awaits the raise and reports the final
/// disagreement when it never arrives, so an occluded capture stays a failure
/// rather than becoming a pass.
pub(super) fn settled_exposure<Attempt>(
    mut attempt: Attempt,
) -> Result<NativeClientPixelCapture, NativePlatformFailure>
where
    Attempt: FnMut() -> Result<NativeClientPixelCapture, NativePlatformFailure>,
{
    let deadline = Instant::now() + EXPOSURE_DEADLINE;
    loop {
        match attempt() {
            Ok(capture) => return Ok(capture),
            Err(failure) => {
                if !matches!(failure, NativePlatformFailure::ClientCapture(_))
                    || Instant::now() >= deadline
                {
                    return Err(failure);
                }
                std::thread::sleep(COMPOSITION_INTERVAL);
            }
        }
    }
}
