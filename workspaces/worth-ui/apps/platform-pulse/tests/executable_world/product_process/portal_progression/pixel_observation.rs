use crate::adjudication::{
    adjudicate_closed_portal_pixels, adjudicate_open_portal_pixels,
    PlatformPulsePortalPixelEvidence,
};
use crate::native_platform::{NativePlatformContract, NativePlatformFailure};

use super::{
    NativeBoundExecutableWorld, PlatformPulsePortalJourneyFailure, PIXEL_POLL_SLICE,
    TRANSITION_DEADLINE,
};

pub(super) fn capture(
    world: &mut NativeBoundExecutableWorld,
) -> Result<crate::external_observation::NativeClientPixelCapture, PlatformPulsePortalJourneyFailure>
{
    world
        .platform
        .capture_client_area(&world.native_client)
        .map_err(PlatformPulsePortalJourneyFailure::Native)
}

pub(super) fn export_capture(
    name: &str,
    capture: &crate::external_observation::NativeClientPixelCapture,
) -> Result<(), String> {
    let Some(directory) = std::env::var_os("WORTH_UI_NATIVE_CAPTURE_DIRECTORY") else {
        return Ok(());
    };
    let directory = std::path::PathBuf::from(directory);
    std::fs::create_dir_all(&directory)
        .map_err(|error| format!("create capture directory {}: {error}", directory.display()))?;
    let path = directory.join(name);
    let bytes = crate::failure_teardown::encode_native_capture_png(capture)
        .map_err(|error| format!("encode {}: {error}", path.display()))?;
    let mut file = std::fs::File::create(&path)
        .map_err(|error| format!("create {}: {error}", path.display()))?;
    std::io::Write::write_all(&mut file, &bytes)
        .map_err(|error| format!("write {}: {error}", path.display()))?;
    file.sync_all()
        .map_err(|error| format!("sync {}: {error}", path.display()))
}

pub(super) fn await_open_pixels(
    world: &mut NativeBoundExecutableWorld,
    baseline: &crate::external_observation::NativeClientPixelCapture,
) -> Result<PlatformPulsePortalPixelEvidence, PlatformPulsePortalJourneyFailure> {
    let deadline = std::time::Instant::now() + TRANSITION_DEADLINE;
    loop {
        let current = match capture(world) {
            Ok(current) => current,
            Err(PlatformPulsePortalJourneyFailure::Native(
                NativePlatformFailure::ClientCapture(_),
            )) if std::time::Instant::now() < deadline => {
                std::thread::sleep(PIXEL_POLL_SLICE);
                continue;
            }
            Err(failure) => return Err(failure),
        };
        if let Ok(evidence) = adjudicate_open_portal_pixels(baseline, &current) {
            return Ok(evidence);
        }
        if std::time::Instant::now() >= deadline {
            return adjudicate_open_portal_pixels(baseline, &current)
                .map_err(PlatformPulsePortalJourneyFailure::Pixels);
        }
        std::thread::sleep(PIXEL_POLL_SLICE);
    }
}

pub(super) fn await_closed_pixels(
    world: &mut NativeBoundExecutableWorld,
    baseline: &crate::external_observation::NativeClientPixelCapture,
) -> Result<crate::external_observation::NativeClientPixelCapture, PlatformPulsePortalJourneyFailure>
{
    let deadline = std::time::Instant::now() + TRANSITION_DEADLINE;
    loop {
        let current = match capture(world) {
            Ok(current) => current,
            Err(PlatformPulsePortalJourneyFailure::Native(
                NativePlatformFailure::ClientCapture(_),
            )) if std::time::Instant::now() < deadline => {
                std::thread::sleep(PIXEL_POLL_SLICE);
                continue;
            }
            Err(failure) => return Err(failure),
        };
        if adjudicate_closed_portal_pixels(baseline, &current).is_ok() {
            return Ok(current);
        }
        if std::time::Instant::now() >= deadline {
            adjudicate_closed_portal_pixels(baseline, &current)
                .map_err(PlatformPulsePortalJourneyFailure::Pixels)?;
            return Ok(current);
        }
        std::thread::sleep(PIXEL_POLL_SLICE);
    }
}
