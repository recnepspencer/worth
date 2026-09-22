use std::time::{Duration, Instant};

use worth_ui_platform_pulse::observation_contract::{
    PlatformPulseLifecycleObservation, PlatformPulseQueryProjectionPosture,
    PlatformPulseSemanticFocusCause,
};

use crate::external_observation::{NativeClientPixelCapture, NativeClientPixelPoint};
use crate::installation::{CanonicalPlatformPulse, IsolatedPulseInstallation};
use crate::native_platform::{CertifiedNativePlatform, NativePlatformContract};
use crate::product_process::{CargoBuiltPlatformPulse, SuccessfulPlatformPulseExit};
use crate::source_delta::{QueryStatusV1, QueryStatusV2};

#[test]
fn authored_query_revisions_reach_the_real_pulse_process() {
    let mut installation = IsolatedPulseInstallation::install(CanonicalPlatformPulse::checked_in())
        .expect("install the current complete Pulse source");
    let mut launch = CargoBuiltPlatformPulse::exact()
        .expect("resolve the product executable")
        .launch(
            installation.source_root(),
            installation.source_root(),
            installation.source_root(),
        )
        .expect("launch the product with its authored sources");
    let deadline = Instant::now() + Duration::from_secs(35);
    let mut first_frame = false;
    let mut pending_issued = false;
    let mut pending_published = false;
    while !first_frame || !pending_issued || !pending_published {
        let envelope = launch
            .lifecycle
            .next(deadline)
            .expect("product lifecycle event");
        match envelope.outcome() {
            PlatformPulseLifecycleObservation::FirstFramePublished(frame) => {
                assert!(frame.actual_native_effect_count() > 0);
                first_frame = true;
            }
            PlatformPulseLifecycleObservation::QueryProjectionIssued(projection) => {
                assert_eq!(projection.projection_identity(), "platform.pulse.status");
                assert_eq!(
                    projection.posture(),
                    PlatformPulseQueryProjectionPosture::Pending
                );
                assert_eq!(projection.native_value(), None);
                pending_issued = true;
            }
            PlatformPulseLifecycleObservation::QueryProjectionPublished(publication) => {
                assert_eq!(
                    publication.projection().projection_identity(),
                    "platform.pulse.status"
                );
                assert_eq!(
                    publication.projection().posture(),
                    PlatformPulseQueryProjectionPosture::Pending
                );
                pending_published = true;
            }
            PlatformPulseLifecycleObservation::TerminalFailure(failure) => {
                panic!("product failed before its first authored Query publication: {failure:?}")
            }
            _ => {}
        }
    }
    let platform = CertifiedNativePlatform::certified().expect("native desktop available");
    let client = platform
        .bind_process_client_area(launch.process.id(), deadline)
        .expect("product presents one native client area");
    let pending_pixels = platform
        .capture_client_area(&client)
        .expect("capture pending status");
    QueryStatusV1
        .apply(&installation)
        .expect("publish the first external status revision");
    let mut current_issued = false;
    let mut current_published = false;
    while !current_issued || !current_published {
        let envelope = launch
            .lifecycle
            .next(deadline)
            .expect("current Query lifecycle event");
        match envelope.outcome() {
            PlatformPulseLifecycleObservation::QueryProjectionIssued(projection) => {
                assert_eq!(
                    projection.posture(),
                    PlatformPulseQueryProjectionPosture::Current
                );
                assert_eq!(projection.native_value(), Some(QueryStatusV1::VALUE));
                current_issued = true;
            }
            PlatformPulseLifecycleObservation::QueryProjectionPublished(publication) => {
                assert_eq!(
                    publication.projection().posture(),
                    PlatformPulseQueryProjectionPosture::Current
                );
                assert_eq!(
                    publication.projection().native_value(),
                    Some(QueryStatusV1::VALUE)
                );
                current_published = true;
            }
            PlatformPulseLifecycleObservation::TerminalFailure(failure) => {
                panic!("product failed before the first current Query publication: {failure:?}")
            }
            _ => {}
        }
    }
    let online_pixels = platform
        .capture_client_area(&client)
        .expect("capture online status");
    assert!(changed_status_pixels(&pending_pixels, &online_pixels) >= 9);
    QueryStatusV2
        .apply(&installation)
        .expect("publish the second external status revision");
    let mut second_issued = false;
    let mut second_published = false;
    while !second_issued || !second_published {
        let envelope = launch
            .lifecycle
            .next(deadline)
            .expect("second current Query lifecycle event");
        match envelope.outcome() {
            PlatformPulseLifecycleObservation::QueryProjectionIssued(projection) => {
                assert_eq!(
                    projection.posture(),
                    PlatformPulseQueryProjectionPosture::Current
                );
                assert_eq!(projection.native_value(), Some(QueryStatusV2::VALUE));
                second_issued = true;
            }
            PlatformPulseLifecycleObservation::QueryProjectionPublished(publication) => {
                assert_eq!(
                    publication.projection().posture(),
                    PlatformPulseQueryProjectionPosture::Current
                );
                assert_eq!(
                    publication.projection().native_value(),
                    Some(QueryStatusV2::VALUE)
                );
                second_published = true;
            }
            PlatformPulseLifecycleObservation::TerminalFailure(failure) => {
                panic!("product failed before the second current Query publication: {failure:?}")
            }
            _ => {}
        }
    }
    let synchronized_pixels = platform
        .capture_client_area(&client)
        .expect("capture synchronized status");
    assert!(changed_status_pixels(&online_pixels, &synchronized_pixels) >= 9);
    let baseline = platform
        .capture_client_area(&client)
        .expect("capture current dashboard");
    let review_target = NativeClientPixelPoint::interior(
        &baseline,
        1434 * baseline.width() / 1536,
        792 * baseline.height() / 1024,
        1,
    )
    .expect("review target lies inside the native client area");
    platform
        .deliver_pointer_activation(&client, review_target)
        .expect("open the review modal through the current product target");
    let mut review_presented = false;
    while !review_presented {
        let envelope = launch
            .lifecycle
            .next(deadline)
            .expect("review modal lifecycle event");
        match envelope.outcome() {
            PlatformPulseLifecycleObservation::SemanticFocusPublished(focus) => {
                review_presented = focus.cause() == PlatformPulseSemanticFocusCause::PortalInitial;
            }
            PlatformPulseLifecycleObservation::TerminalFailure(failure) => {
                panic!("product failed while opening review: {failure:?}")
            }
            _ => {}
        }
    }
    let modal = platform
        .capture_client_area(&client)
        .expect("capture presented review modal");
    let approve_target = NativeClientPixelPoint::interior(
        &modal,
        892 * modal.width() / 1536,
        681 * modal.height() / 1024,
        1,
    )
    .expect("approve target lies inside the native client area");
    platform
        .deliver_pointer_activation(&client, approve_target)
        .expect("approve the review through the native modal target");
    let mut action_issued = false;
    let mut action_published = false;
    while !action_issued || !action_published {
        let envelope = launch
            .lifecycle
            .next(deadline)
            .expect("action Query lifecycle event");
        match envelope.outcome() {
            PlatformPulseLifecycleObservation::QueryProjectionIssued(projection) => {
                assert_eq!(projection.native_value(), Some("Deployed"));
                action_issued = true;
            }
            PlatformPulseLifecycleObservation::QueryProjectionPublished(publication) => {
                assert_eq!(publication.projection().native_value(), Some("Deployed"));
                action_published = true;
            }
            PlatformPulseLifecycleObservation::TerminalFailure(failure) => {
                panic!("product failed before the authored action consequence: {failure:?}")
            }
            _ => {}
        }
    }
    let deployed_pixels = platform
        .capture_client_area(&client)
        .expect("capture the accepted action consequence");
    assert!(changed_status_pixels(&synchronized_pixels, &deployed_pixels) >= 9);
    platform
        .request_normal_close(&client)
        .expect("close product window");
    let mut shutdown = false;
    while !shutdown {
        let envelope = launch
            .lifecycle
            .next(deadline)
            .expect("shutdown lifecycle event");
        match envelope.outcome() {
            PlatformPulseLifecycleObservation::ShutdownCompleted(_) => shutdown = true,
            PlatformPulseLifecycleObservation::TerminalFailure(failure) => {
                panic!("product failed during normal close: {failure:?}")
            }
            _ => {}
        }
    }
    SuccessfulPlatformPulseExit::wait(&mut launch.process, deadline)
        .expect("product exits after normal close");
    platform
        .verify_process_window_released(launch.process.id())
        .expect("native window released");
    installation
        .close()
        .expect("release the isolated source installation");
}

fn changed_status_pixels(
    before: &NativeClientPixelCapture,
    after: &NativeClientPixelCapture,
) -> usize {
    assert_eq!(
        [before.width(), before.height()],
        [after.width(), after.height()]
    );
    let [x, y, width, height] = [1392_u32, 779, 85, 27];
    let left = x * before.width() / 1536;
    let top = y * before.height() / 1024;
    let right = (x + width) * before.width() / 1536;
    let bottom = (y + height) * before.height() / 1024;
    (top..bottom)
        .flat_map(|row| (left..right).map(move |column| (row, column)))
        .filter(|(row, column)| {
            let index = ((*row * before.width() + *column) * 4) as usize;
            before.rgba()[index..index + 3] != after.rgba()[index..index + 3]
        })
        .count()
}
