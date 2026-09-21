//! Facade-level proof that a settle ends at the boundary that took away what
//! it was settling.
//!
//! A smooth notch leaves three things alive at once: a semantic target in
//! Scroll, an accepted sample in the sampler, and a committed Motion track
//! walking one toward the other. Each scenario here removes the thing that
//! made the settle meaningful and then asks whether all three are gone.
//!
//! The question matters because none of these boundaries is reported to the
//! settle. Nothing tells a Motion track that the occurrence it moves was
//! unmounted, that the binding whose rectangles it was crossing has been given
//! back, or that the room it was moving content through has closed up. A settle that survives one of them goes on writing offsets into
//! a region nobody is watching, and hands the next reader content that moved
//! while they were away.
//!
//! Every scenario pays frames before the boundary until the content has
//! visibly started travelling, so the settle being ended is a live one rather
//! than one that had nothing left to do. The first frame after a notch samples
//! the rest pose and moves nothing, so that takes two.

use super::super::super::UiScrollSettleDisposition;
use super::geometry::scrollable::install_scrollable_primary_with_collapsed_content;
use super::scroll_pose_authority::{block, ScrollWorld};
use super::scroll_settle_commit::{one_notch_up, pending_transitions, smooth_world, ONE_NOTCH};
use crate::mounting::{UiSurfaceBindingCoordinatePosture, UiSurfaceBindingProfile};
use crate::runtime::scroll::{UiHostScrollObservationDenial, UiHostScrollObservationOutcome};
use worth_ui_host_contract::*;

const NOTCH_TICK: u64 = 5;

fn presentation(scroll: &ScrollWorld) -> UiHostObservationPresentationBasis {
    scroll
        .world
        .session
        .mounted
        .current_presentation_for_surface(scroll.surface())
        .expect("the first surface is published")
}

/// One Motion frame the way the native shell runs it, for a settle that may
/// already have ended: a tick a refused sampler cannot answer prepares
/// nothing, and there is nothing left for it to move.
fn frame(scroll: &mut ScrollWorld, tick: u64) -> UiScrollSettleDisposition {
    let basis = presentation(scroll);
    if let Ok(prepared) = scroll.world.session.prepare_motion_tick(tick, basis) {
        scroll
            .world
            .session
            .present_prepared_motion_tick(prepared, basis);
    }
    scroll.world.session.settle_accepted_scroll_sample(basis)
}

/// A World with one notch published and two frames of it paid, so the content
/// is on its way and the settle carrying it is live.
fn world_mid_settle() -> ScrollWorld {
    let mut scroll = smooth_world(true);
    assert!(matches!(
        scroll.wheel(ONE_NOTCH, one_notch_up(), NOTCH_TICK),
        UiHostScrollObservationOutcome::Applied(_)
    ));
    assert_eq!(pending_transitions(&scroll), 1);
    assert_eq!(
        frame(&mut scroll, NOTCH_TICK + 1),
        UiScrollSettleDisposition::Applied
    );
    assert_eq!(
        frame(&mut scroll, NOTCH_TICK + 2),
        UiScrollSettleDisposition::Applied
    );
    assert_ne!(
        scroll.accepted_offset(),
        block(0),
        "the paid frames have already moved the content"
    );
    scroll
}

/// Nothing is left settling: no semantic target, no accepted sample, and no
/// frame that finds work to do.
fn nothing_is_settling(scroll: &mut ScrollWorld, tick: u64) {
    assert_eq!(
        pending_transitions(scroll),
        0,
        "the semantic target the settle was walking toward is gone"
    );
    assert!(
        scroll
            .world
            .session
            .mounted
            .accepted_scroll_group_translations()
            .is_empty(),
        "no accepted sample is left for a later frame to commit"
    );
    assert_eq!(
        frame(scroll, tick),
        UiScrollSettleDisposition::Idle,
        "a frame after the boundary has nothing to settle"
    );
}

/// No accepted sample survives the boundary, so no later frame can commit one.
fn no_sample_is_owed(scroll: &ScrollWorld) {
    assert_eq!(
        pending_transitions(scroll),
        0,
        "the semantic target the settle was walking toward is gone"
    );
    assert!(
        scroll
            .world
            .session
            .mounted
            .accepted_scroll_group_translations()
            .is_empty(),
        "no accepted sample is left for a later frame to commit"
    );
}

/// The occurrence a settle was moving is unmounted. Content that is no longer
/// mounted cannot finish arriving, so the settle ends rather than walk a
/// region that is gone.
#[test]
fn unmounting_the_occurrence_ends_the_settle_it_was_carrying() {
    let mut scroll = world_mid_settle();
    let occurrence = scroll.target();
    scroll
        .world
        .session
        .unmount_instance_with_interaction_receipt(occurrence)
        .expect("a mounted occurrence unmounts");
    nothing_is_settling(&mut scroll, NOTCH_TICK + 3);
    let _ = scroll.world.session.shutdown();
}

/// The binding whose rectangles the content was crossing is given back. A
/// settle that outlived it would resume against geometry from a presentation
/// nobody is showing any more.
#[test]
fn rebinding_the_surface_ends_the_settle_that_was_crossing_it() {
    let mut scroll = world_mid_settle();
    let binding = presentation(&scroll).binding();
    scroll
        .world
        .session
        .rebind_host_surface_with_interaction_receipt(
            binding,
            UiHostSurfacePresentationMode::NativeDisplay,
            UiSurfaceBindingProfile::new(
                1_000,
                UiSurfaceBindingCoordinatePosture::LogicalPoints,
                2,
            )
            .expect("a second binding profile"),
        )
        .expect("a presented surface rebinds");
    no_sample_is_owed(&scroll);
    let _ = scroll.world.session.shutdown();
}

/// The window stops being the reader's. Every settle ends where its last
/// accepted sample put the content, and the wheel gesture gives its owner
/// back, so the first notch after the reader returns is theirs to aim.
#[test]
fn losing_the_window_ends_the_settle_and_gives_the_gesture_back() {
    let mut scroll = world_mid_settle();
    let stopped_at = scroll.accepted_offset();
    unfocus(&mut scroll);

    nothing_is_settling(&mut scroll, NOTCH_TICK + 4);
    assert_eq!(
        scroll.accepted_offset(),
        stopped_at,
        "content stops where the last sample the reader saw put it"
    );
    let target = scroll.surface_only_target();
    assert!(
        matches!(
            scroll.targeted_wheel(
                UiHostScrollDeltaPhase::Updated,
                target,
                ONE_NOTCH,
                one_notch_up(),
                NOTCH_TICK + 5,
            ),
            UiHostScrollObservationOutcome::Denied(
                UiHostScrollObservationDenial::PresentedSurfaceFallbackIsAmbiguous
            )
        ),
        "the latch went with the attention, so a surface-only report names nobody"
    );
    let _ = scroll.world.session.shutdown();
}

/// The content a settle was carrying shrinks until it fits the region showing
/// it. Travel is what was left over, and nothing is left over now, so the
/// target has nowhere to arrive: reconciling the new extent retires it, and the
/// same publication ends the sampler entry and the Motion track that were
/// walking toward it. A settle that outlived the room it needed would go on
/// translating content that has none.
#[test]
fn collapsing_the_extent_ends_the_settle_that_had_nowhere_left_to_arrive() {
    let mut scroll = world_mid_settle();
    install_scrollable_primary_with_collapsed_content(
        &mut scroll.world.session,
        scroll.world.surfaces,
        scroll.world.instances,
    );
    let frame = scroll.world.prepare();
    scroll.world.publish(frame, NOTCH_TICK + 3, false);

    nothing_is_settling(&mut scroll, NOTCH_TICK + 4);
    assert_eq!(
        scroll.accepted_offset(),
        block(0),
        "a region with nowhere to travel holds its content at rest"
    );
    let _ = scroll.world.session.shutdown();
}

/// Tell the session the window stopped being the reader's, through the same
/// observation batch the host sends. A wheel report reaches this World without
/// a batch, so this is the first observation sequence the session is owed and
/// the only one these scenarios spend.
const FOCUS_REPORT_SEQUENCE: u64 = 1;

fn unfocus(scroll: &mut ScrollWorld) {
    let presentation = presentation(scroll);
    let UiHostProtocolNegotiation::Compatible(protocol) =
        UiHostProtocolContract::current().negotiate()
    else {
        panic!("the current host protocol negotiates")
    };
    let sequence = UiHostObservationSequence::new(FOCUS_REPORT_SEQUENCE);
    let batch = UiHostObservationBatch::new(UiHostObservationBatchInput {
        protocol,
        host_session: scroll.world.session.host_session.identity().as_u64(),
        presentation,
        sequences: UiHostObservationSequenceRange::new(sequence, sequence),
        loss: UiHostObservationLoss::Complete,
        reports: vec![UiHostObservationReport::new(
            sequence,
            UiHostObservationTimeBasis::HostMonotonicMillis(sequence.value()),
            UiHostObservationPayload::WindowFocus {
                surface: presentation.host_surface(),
                focused: false,
            },
        )],
    })
    .expect("a single focus report is a batch");
    let outcome = scroll.world.session.admit_host_interaction_batch(batch);
    assert!(
        matches!(
            outcome,
            crate::facade::interaction::UiHostInteractionIngressOutcome::Applied(_)
        ),
        "a WindowFocus report reaches the session: {outcome:?}"
    );
}
