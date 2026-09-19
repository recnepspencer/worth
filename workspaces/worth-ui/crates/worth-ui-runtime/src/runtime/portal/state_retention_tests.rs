//! Portal live-table boundedness across remount incarnations.
//!
//! Every remount mints a fresh mounted-instance identity, so a portal opened
//! after a rebind is a different `UiPortalIdentity`. This family proves that
//! terminal portals leave the live table, that the retained duplicate-request
//! window stays inside its declared capacity, and that the command-routing
//! scan therefore stays proportional to the currently active portals.

use super::state::duplicate_request_capacity_for_test;
use super::test_support::{idempotency, open_request, portal, state};
use super::{UiPortalDismissalCause, UiPortalIdentity, UiPortalServiceRequest};

/// One remount incarnation: a distinct mounted instance for the same graph node.
fn remounted_portal(incarnation: u64) -> UiPortalIdentity {
    portal(7, incarnation)
}

/// Opens and terminally closes one portal, returning the exact close request so
/// a caller can replay it as a genuine duplicate.
fn open_then_close(
    state: &mut super::UiPortalRuntimeState,
    portal: UiPortalIdentity,
    lineage: u64,
) -> UiPortalServiceRequest {
    let open = state
        .prepare(open_request(portal, lineage))
        .expect("remounted portal opens");
    let surface = open.request().semantic_surface();
    state.commit_published(open).expect("open remains current");
    let close_request = UiPortalServiceRequest::close(
        portal,
        idempotency(lineage + 1),
        UiPortalDismissalCause::Escape,
        surface,
    );
    let close = state
        .prepare(close_request)
        .expect("remounted portal closes");
    state
        .commit_published(close)
        .expect("close remains current");
    close_request
}

#[test]
fn terminal_portals_leave_the_live_table_across_many_remount_incarnations() {
    let mut state = state();
    let capacity = duplicate_request_capacity_for_test();
    let incarnations = u64::try_from(capacity).expect("capacity fits a u64") * 4;

    for incarnation in 1..=incarnations {
        open_then_close(&mut state, remounted_portal(incarnation), incarnation * 10);
        assert_eq!(
            state.live_record_count(),
            0,
            "a terminally closed portal must not stay in the live table"
        );
    }

    // Retention is bounded by the declared window, not by remount count.
    assert_eq!(state.record_count(), capacity);
    assert!(state.record_count() < usize::try_from(incarnations).unwrap());
}

#[test]
fn command_routing_scan_does_no_work_once_every_portal_is_terminal() {
    let mut state = state();
    let incarnations = u64::try_from(duplicate_request_capacity_for_test()).unwrap() * 4;
    for incarnation in 1..=incarnations {
        open_then_close(&mut state, remounted_portal(incarnation), incarnation * 10);
    }

    assert_eq!(state.active_count(), 0);
    assert_eq!(state.active_portal_owner_graph_nodes().count(), 0);

    // One live portal costs exactly one visited owner, independent of history.
    let live = remounted_portal(incarnations + 1);
    let open = state
        .prepare(open_request(live, 9_001))
        .expect("live portal opens");
    state.commit_published(open).expect("open remains current");

    assert_eq!(state.active_count(), 1);
    assert_eq!(state.active_portal_owner_graph_nodes().count(), 1);
}

#[test]
fn a_duplicate_close_inside_the_window_still_settles_idempotently() {
    let mut state = state();
    let portal = remounted_portal(1);
    let close_request = open_then_close(&mut state, portal, 10);
    let idempotent_before = state.idempotent_requests();

    let duplicate = state
        .prepare(close_request)
        .expect("duplicate close prepares against the retained request");
    let receipt = state
        .commit_published(duplicate)
        .expect("duplicate close remains current");

    assert_eq!(
        receipt.disposition(),
        super::UiPortalServiceDisposition::Idempotent
    );
    assert_eq!(state.idempotent_requests(), idempotent_before + 1);
    assert_eq!(state.live_record_count(), 0);
}

#[test]
fn a_duplicate_close_evicted_from_the_window_is_denied_as_not_live() {
    let mut state = state();
    let evicted = remounted_portal(1);
    let close_request = open_then_close(&mut state, evicted, 10);

    let capacity = u64::try_from(duplicate_request_capacity_for_test()).unwrap();
    for incarnation in 2..=capacity + 1 {
        open_then_close(&mut state, remounted_portal(incarnation), incarnation * 10);
    }

    assert!(matches!(
        state.prepare(close_request),
        Err(super::UiPortalServiceTransitionDenial::PortalNotLive)
    ));
    assert_eq!(state.live_record_count(), 0);
}

#[test]
fn shutdown_releases_both_live_records_and_the_retained_window() {
    let mut state = state();
    for incarnation in 1..=3 {
        open_then_close(&mut state, remounted_portal(incarnation), incarnation * 10);
    }
    let live = remounted_portal(9);
    let open = state.prepare(open_request(live, 900)).unwrap();
    state.commit_published(open).unwrap();
    assert!(state.record_count() > 0);

    let report = state.shutdown();

    assert_eq!(report.closed_records(), 1);
    assert_eq!(report.final_active_records(), 0);
    assert_eq!(state.record_count(), 0);
}
