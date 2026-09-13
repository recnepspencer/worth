use std::time::{Duration, Instant};

use crate::product_process::{Closed, Published, PulseExecutableWorld};

const TRANSITION_DEADLINE: Duration = Duration::from_secs(5);

pub(super) fn close_recovered_at_sequence<Stage>(
    recovered: PulseExecutableWorld<Published<Stage>>,
    expected_shutdown_sequence: u64,
) -> PulseExecutableWorld<Closed> {
    let closed = recovered
        .close_native_window(Instant::now() + TRANSITION_DEADLINE)
        .unwrap_or_else(|failure| {
            panic!("normal close, typed shutdown, successful exit, and cleanup: {failure}")
        });
    let cleanup = closed.evidence();
    assert_eq!(cleanup.close_request_count(), 1);
    assert_eq!(cleanup.shutdown_sequence(), expected_shutdown_sequence);
    assert!(cleanup.shutdown().host_session_released());
    assert!(cleanup.shutdown().query_watcher_joined());
    assert!(cleanup.shutdown().query_owner_terminal());
    assert_eq!(cleanup.shutdown().pending_query_observation_count(), 0);
    assert_eq!(cleanup.shutdown().live_query_source_count(), 0);
    assert_eq!(cleanup.shutdown().live_query_attempt_count(), 0);
    assert_eq!(cleanup.shutdown().live_query_resource_count(), 0);
    assert_eq!(cleanup.shutdown().live_query_consumer_lease_count(), 0);
    assert_eq!(cleanup.shutdown().retained_query_projection_count(), 0);
    assert_eq!(cleanup.shutdown().query_projection_receipt_count(), 0);
    assert_eq!(cleanup.shutdown().cancelled_visual_capture_count(), 0);
    assert_eq!(cleanup.shutdown().disposed_visual_snapshot_count(), 0);
    assert_eq!(cleanup.shutdown().disposed_visual_pixel_bytes(), 0);
    assert_eq!(cleanup.shutdown().disposed_visual_structural_bytes(), 0);
    assert_eq!(cleanup.shutdown().cancelled_pending_overlay_count(), 0);
    assert_eq!(cleanup.shutdown().disposed_published_overlay_count(), 0);
    assert_eq!(cleanup.shutdown().disposed_clearing_overlay_count(), 0);
    assert!(cleanup.successful_exit().status().success());
    assert!(cleanup.successful_exit().poll_count() > 0);
    assert!(cleanup.installation_removed());
    closed
}
