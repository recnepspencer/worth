//! Facade-level proof that a burst of wheel input leaves the session holding
//! no more than one notch's worth of anything.
//!
//! A reader spinning a wheel hands the host hundreds of events in a second,
//! and the host hands every one of them to the runtime through the same
//! ingress a single notch arrives on. Each event is validated, routed against
//! an ownership chain, and -- under a declared smooth wheel -- turned into
//! travel some settle has to carry. Every one of those is somewhere a burst
//! could leave a residue behind, and a residue left once per event is a leak
//! measured in how long the reader scrolled. Each batch here is applied before
//! the next is handed over, so what these scenarios measure is the residue of
//! the apply path; an undelivered queue that grows because the host outruns
//! the runtime is a backlog no scenario here builds.
//!
//! So the question these scenarios ask is not whether storage is bounded. It
//! is whether storage depends on the event count at all. Two bursts sixteen
//! times apart are driven through the production ingress and their storage is
//! compared for equality rather than against a limit: an implementation that
//! retained the history and merely capped it would differ below its cap, and
//! one that retained per event would differ everywhere.
//!
//! Both lengths are past every fixed-size window this burst feeds -- the
//! largest is the sixteen recent batch fingerprints a binding partition
//! remembers for duplicate detection -- so neither burst is still filling one
//! up, and an inequality is growth rather than warm-up. The path keeps larger
//! windows than that, but they are filled by what a burst of wheel events does
//! not do: no frame retires per event, so the terminal frame identities a host
//! exchange remembers stay where both bursts leave them.
//!
//! What is compared is what the runtime already reports about itself: the
//! mounted retention report, which counts retained items, structural bytes and
//! active leases for every retention class including the host observation
//! queue and its quarantine, and the runtime service resource census, which
//! counts the owners and the live tracks every service holds. Neither is
//! written for this; both are the report a native host reads.
//!
//! Non-vacuity is the other half. Storage that does not move is worthless
//! evidence if the events did nothing, so every scenario also shows the burst
//! doing work the runtime had to keep: the reader travels further than one
//! notch carries, and the settle a smooth burst stages aims past where a
//! single notch aims.

use super::super::super::UiScrollSettleDisposition;
use super::scroll_coarse_wheel::{immediate_world, one_notch_down, ONE_NOTCH as IMMEDIATE_NOTCH};
use super::scroll_pose_authority::{block, ScrollWorld};
use super::scroll_settle_commit::{
    one_notch_up, pending_transitions, smooth_world, LINE_EXTENT_POINTS, ONE_NOTCH as SMOOTH_NOTCH,
};
use worth_ui_host_contract::*;

/// The shorter burst. Already past the sixteen batch fingerprints a partition
/// remembers, so nothing here is still warming up.
const SHORT_BURST: u64 = 32;
/// The longer one, sixteen times the shorter. Storage that carried the history
/// would be sixteen times bigger here.
const LONG_BURST: u64 = 512;
/// How far the content this region carries can travel. Every burst runs past
/// it, which is what a reader spinning a wheel does.
const CONTENT_TRAVEL_POINTS: i64 = 30;

/// Everything a burst could leave behind, taken through the reports the
/// runtime already publishes about itself.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Storage {
    retention: crate::inspection::mounted_frame::UiMountedRetentionReport,
    services: worth_ui_inspection::UiRuntimeServiceResourceCensus,
    scroll_settles: usize,
}

fn storage(scroll: &ScrollWorld) -> Storage {
    Storage {
        retention: scroll.world.session.mounted_retention_report(),
        services: scroll.world.session.runtime_service_resource_census(),
        scroll_settles: pending_transitions(scroll),
    }
}

/// One wheel notch as a host delivers it: its own batch, its own sequence, and
/// a host monotonic reading that gives the gesture its input tick.
fn wheel_batch(
    scroll: &mut ScrollWorld,
    precision: UiHostScrollDeltaPrecision,
    y_subpixels: i64,
    sequence: u64,
) -> crate::facade::interaction::UiHostInteractionIngressOutcome {
    let presentation = scroll.presentation();
    let target = scroll.pointer_target();
    let UiHostProtocolNegotiation::Compatible(protocol) =
        UiHostProtocolContract::current().negotiate()
    else {
        panic!("the current host protocol negotiates")
    };
    let sequence = UiHostObservationSequence::new(sequence);
    let batch = UiHostObservationBatch::new(UiHostObservationBatchInput {
        protocol,
        host_session: scroll.world.session.host_session.identity().as_u64(),
        presentation,
        sequences: UiHostObservationSequenceRange::new(sequence, sequence),
        loss: UiHostObservationLoss::Complete,
        reports: vec![UiHostObservationReport::new(
            sequence,
            UiHostObservationTimeBasis::HostMonotonicMillis(sequence.value()),
            UiHostObservationPayload::ScrollDelta {
                source: UiHostScrollDeltaSource::PointerWheel,
                phase: UiHostScrollDeltaPhase::Updated,
                precision,
                target,
                x_subpixels: 0,
                y_subpixels,
            },
        )],
    })
    .expect("a single scroll report is a batch");
    scroll.world.session.admit_host_interaction_batch(batch)
}

/// Deliver `events` notches, each one reaching the session the way the host
/// sends it. Every one is admitted: a burst refused partway through would
/// leave the rest of the scenario measuring a session that stopped listening.
fn burst(
    scroll: &mut ScrollWorld,
    precision: UiHostScrollDeltaPrecision,
    y_subpixels: i64,
    events: u64,
) {
    for sequence in 1..=events {
        let outcome = wheel_batch(scroll, precision, y_subpixels, sequence);
        assert!(
            matches!(
                outcome,
                crate::facade::interaction::UiHostInteractionIngressOutcome::Applied(_)
            ),
            "notch {sequence} of {events} reaches the session: {outcome:?}"
        );
    }
}

/// A direct burst coalesces one unpublished successor, independent of event
/// count. Only its subsequent ordinary physical publication moves the offset.
#[test]
fn an_immediate_burst_leaves_the_storage_the_session_started_with() {
    let mut short = immediate_world(true);
    let mut long = immediate_world(true);
    let at_rest = storage(&short);
    assert_eq!(
        at_rest,
        storage(&long),
        "two sessions launched the same way hold the same things"
    );

    burst(&mut short, IMMEDIATE_NOTCH, one_notch_down(), SHORT_BURST);
    burst(&mut long, IMMEDIATE_NOTCH, one_notch_down(), LONG_BURST);

    assert_eq!(
        storage(&long),
        storage(&short),
        "sixteen times the notches, the same storage"
    );
    assert_eq!(
        storage(&short),
        at_rest,
        "no additional service track or retained frame is created per event"
    );
    assert_eq!(long.accepted_offset(), block(0));
    assert_eq!(short.accepted_offset(), block(0));
    assert_eq!(
        long.world
            .session
            .scroll
            .as_ref()
            .unwrap()
            .pending_direct_count(),
        1
    );
    assert_eq!(
        short
            .world
            .session
            .scroll
            .as_ref()
            .unwrap()
            .pending_direct_count(),
        1
    );
    assert!(long
        .world
        .session
        .mounted
        .has_pending_direct_scroll(long.surface()));
    assert!(short
        .world
        .session
        .mounted
        .has_pending_direct_scroll(short.surface()));
    short.publish_direct(SHORT_BURST + 1);
    long.publish_direct(LONG_BURST + 1);
    assert!(!long
        .world
        .session
        .mounted
        .has_pending_direct_scroll(long.surface()));
    assert!(!short
        .world
        .session
        .mounted
        .has_pending_direct_scroll(short.surface()));
    assert_eq!(
        long.accepted_offset(),
        block(CONTENT_TRAVEL_POINTS),
        "the reader is at the end of the content the region carries"
    );
    assert!(
        CONTENT_TRAVEL_POINTS > i64::from(LINE_EXTENT_POINTS),
        "which is further than a single notch reaches, so the burst traveled"
    );
    let _ = short.world.session.shutdown();
    let _ = long.world.session.shutdown();
}

/// The staged offset a settle is walking toward.
fn aimed(scroll: &ScrollWorld) -> crate::runtime::scroll::UiScrollOffset {
    scroll
        .world
        .session
        .scroll
        .as_ref()
        .expect("Scroll stays installed")
        .transition_target(scroll.owner, scroll.incarnation)
        .expect("the burst staged a target")
        .target_offset()
}

/// The same burst under a wheel that settles. Every notch stages travel a
/// Motion track has to carry, and a session that filed one track per notch
/// would be carrying five hundred. It carries one, and the one it carries aims
/// where the whole burst pointed rather than where the last notch did.
#[test]
fn a_smooth_burst_is_carried_by_one_settle_however_long_it_is() {
    let mut short = smooth_world(true);
    let mut long = smooth_world(true);

    burst(&mut short, SMOOTH_NOTCH, one_notch_up(), SHORT_BURST);
    burst(&mut long, SMOOTH_NOTCH, one_notch_up(), LONG_BURST);

    assert_eq!(
        storage(&long),
        storage(&short),
        "sixteen times the notches, the same storage"
    );
    assert_eq!(
        pending_transitions(&long),
        1,
        "one settle target, not one per notch"
    );
    assert_eq!(
        long.world
            .session
            .runtime_service_resource_census()
            .active_motion_tracks(),
        1,
        "and one track carrying it"
    );

    let mut single = smooth_world(true);
    burst(&mut single, SMOOTH_NOTCH, one_notch_up(), 1);
    assert_eq!(
        aimed(&single),
        block(i64::from(LINE_EXTENT_POINTS)),
        "one notch aims one line away"
    );
    assert_eq!(
        aimed(&long),
        block(CONTENT_TRAVEL_POINTS),
        "and the burst aims at the end of the content, so its notches accumulated"
    );
    let _ = short.world.session.shutdown();
    let _ = long.world.session.shutdown();
    let _ = single.world.session.shutdown();
}

/// One Motion frame the way the native shell runs it.
fn frame(scroll: &mut ScrollWorld, tick: u64) -> UiScrollSettleDisposition {
    super::scroll_settle_frame::settle_frame(scroll, tick)
}

/// What a burst took while it was in flight it gives back when it lands. The
/// mounted retention a settle leaves behind is the frames it published, which
/// is what publishing frames is for; the live work is the part that has to
/// end, and the census is where a session says how much of it it still holds.
#[test]
fn the_work_a_burst_creates_ends_when_its_settle_lands() {
    let mut scroll = smooth_world(true);
    let at_rest = storage(&scroll).services;

    burst(&mut scroll, SMOOTH_NOTCH, one_notch_up(), LONG_BURST);
    assert_eq!(pending_transitions(&scroll), 1);
    // The census has to be able to tell the two states apart, or the equality
    // the scenario ends on is two constants agreeing rather than a session
    // giving its work back.
    assert_ne!(
        storage(&scroll).services,
        at_rest,
        "a session carrying a settle is not a session at rest"
    );

    let mut tick = LONG_BURST;
    while pending_transitions(&scroll) > 0 {
        tick += 1;
        assert!(
            tick <= LONG_BURST + 64,
            "the settle a burst staged reaches its target"
        );
        assert_eq!(frame(&mut scroll, tick), UiScrollSettleDisposition::Applied);
    }

    assert_eq!(
        scroll.accepted_offset(),
        block(CONTENT_TRAVEL_POINTS),
        "the reader arrives where the burst aimed"
    );
    assert_eq!(
        storage(&scroll).services,
        at_rest,
        "and the session holds the owners it launched with and no live work"
    );
    let _ = scroll.world.session.shutdown();
}
