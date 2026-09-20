//! Launch the exact canonical Platform Pulse dashboard and hold it at rest:
//! the causal first publication, one process, the declared client extent, and
//! the Recent activity thumb painted where the declared geometry puts it.
use std::time::{Duration, Instant};

use crate::installation::CanonicalPlatformPulse;
use crate::product_process::{
    AwaitingFirstFrame, CargoBuiltPlatformPulse, DashboardAtRest, Installed, Published,
    PulseExecutableWorld,
};

const FIRST_FRAME_DEADLINE: Duration = Duration::from_secs(45);

pub(super) fn launch_dashboard_at_rest() -> PulseExecutableWorld<Published<DashboardAtRest>> {
    let installed: PulseExecutableWorld<Installed> =
        PulseExecutableWorld::install(CanonicalPlatformPulse::checked_in())
            .unwrap_or_else(|failure| panic!("install exact canonical source: {failure}"));
    let binary = CargoBuiltPlatformPulse::exact()
        .unwrap_or_else(|failure| panic!("resolve exact Cargo executable: {failure}"));
    let awaiting: PulseExecutableWorld<AwaitingFirstFrame> = installed
        .launch(binary)
        .unwrap_or_else(|failure| panic!("launch exact product process: {failure}"));
    let published = awaiting
        .await_dashboard_at_rest(Instant::now() + FIRST_FRAME_DEADLINE)
        .unwrap_or_else(|failure| {
            panic!("causal first publication plus the dashboard at rest: {failure}")
        });
    let evidence = published.evidence();
    assert_eq!(evidence.sequence_quad(), (1, 1, 3, 1));
    assert_eq!(
        evidence.pending_projection().projection_identity(),
        "platform.pulse.status"
    );
    assert!(evidence.first_frame().actual_native_effect_count() > 0);
    assert!(evidence.client_area().window_lookup_count() > 0);
    assert!(evidence.liveness().liveness_checks() >= 2);
    assert_eq!(evidence.capture_count(), 1);
    assert!(evidence.resting_thumb().length_px() >= 24);
    published
}
