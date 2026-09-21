//! Facade-level proof that content moving under a still pointer re-resolves
//! what that pointer is over, on the frame the content moves.
//!
//! The World here lays the third component inside the first component's
//! scrollable region: twelve points below that region's top edge and eight
//! points tall, and in front of the region's owner in the authored hit order.
//! A pointer resting five points below the top edge is therefore over the
//! owner at rest and over the nested component once ten points of content have
//! passed under it. The pointer never moves, and the host never says it did.
//!
//! Both routes that move scrolled content are driven through their own
//! production entry: an immediate wheel, which places its whole delta on the
//! frame it arrives, and a published settle, which pays an accepted sample out
//! over several frames. Neither is given a synthetic pointer event. What each
//! asserts is the reader's own test of whether hover is honest, because the
//! row the pointer is shown over is the row a click would reach.

use super::scroll_pose_authority::{block, ScrollWorld};
use super::scroll_settle_commit::{one_notch_up, smooth_scroll, ONE_NOTCH, SETTLE_TICKS};
use super::World;
use crate::runtime::scroll::UiHostScrollObservationOutcome;
use worth_ui_host_contract::{
    UiHostObservationPresentationBasis, UiHostPointerIdentity, UiHostScrollDeltaPhase,
    UiHostScrollDeltaPrecision, UiHostSurfacePosition, UiMountedInstanceIdentity,
    UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT,
};

/// Where the pointer rests: inside the first component and five points below
/// the top edge its region shares with it, which is seven points above the
/// nested component at rest.
const RESTING_POINT: [i64; 2] = [
    150 * UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT,
    55 * UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT,
];

/// How far the content travels in these scenarios: enough to carry the nested
/// component from below the resting point to over it.
const TRAVEL_POINTS: i64 = 10;

/// A host block delta that pushes content toward the top of the viewport by
/// `TRAVEL_POINTS`, which moves the accepted offset the other way.
fn wheel_travel() -> i64 {
    -TRAVEL_POINTS * UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT
}

fn presentation(scroll: &ScrollWorld) -> UiHostObservationPresentationBasis {
    scroll
        .world
        .session
        .mounted
        .current_presentation_for_surface(scroll.surface())
        .expect("the first surface is published")
}

/// Put one pointer at the resting point and leave it there. Every later
/// assertion reads the same pointer without the host reporting it again.
fn rest_pointer_on_the_component(scroll: &mut ScrollWorld) {
    let basis = presentation(scroll);
    let batch = super::stationary_motion::inputs::pointer_batch(
        scroll.world.session.host_session.identity().as_u64(),
        basis,
        1,
        UiHostPointerIdentity::new(1),
        UiHostSurfacePosition::viewport_logical(RESTING_POINT[0], RESTING_POINT[1]),
        None,
        false,
    );
    let ingress = scroll.world.session.admit_host_interaction_batch(batch);
    let crate::facade::interaction::UiHostInteractionIngressOutcome::Applied(receipt) = ingress
    else {
        panic!("a pointer over the scrollable component must be admitted: {ingress:?}");
    };
    assert!(receipt.pointer_presence_denials().is_empty());
}

/// What the resting pointer is currently over.
fn hovered(scroll: &ScrollWorld) -> Option<UiMountedInstanceIdentity> {
    scroll
        .world
        .session
        .interaction
        .pointer_presence_appearance_snapshot()
        .expect("an admitted pointer leaves a presence owner")
        .postures()
        .iter()
        .find(|posture| posture.pointer() == UiHostPointerIdentity::new(1))
        .expect("the resting pointer keeps its posture")
        .target()
}

/// One Motion frame the way the native shell runs it.
fn settle_frame(scroll: &mut ScrollWorld, tick: u64) {
    let basis = presentation(scroll);
    let prepared = scroll
        .world
        .session
        .prepare_motion_tick(tick, basis)
        .expect("an armed settle prepares its tick");
    scroll
        .world
        .session
        .present_prepared_motion_tick(prepared, basis);
    scroll.world.session.settle_accepted_scroll_sample(basis);
}

/// A wheel that places its whole delta at once moves the content and the rows
/// a pointer lands on in the same breath. The pointer is not reported again,
/// so if hover were left to the next pointer event it would still name the
/// region's owner.
#[test]
fn an_immediate_wheel_moves_what_the_resting_pointer_is_over() {
    let mut scroll = ScrollWorld::publish_with_nested_content(World::launch());
    rest_pointer_on_the_component(&mut scroll);
    let owner = scroll.world.instances[0];
    let nested = scroll.world.instances[2];
    assert_eq!(
        hovered(&scroll),
        Some(owner),
        "at rest the nested component begins below the resting point"
    );

    let target = scroll.pointer_target();
    assert!(matches!(
        scroll.targeted_wheel(
            UiHostScrollDeltaPhase::Updated,
            target,
            UiHostScrollDeltaPrecision::Pixel,
            wheel_travel(),
            5,
        ),
        UiHostScrollObservationOutcome::Applied(_)
    ));
    assert_eq!(scroll.accepted_offset(), block(TRAVEL_POINTS));
    assert_eq!(
        hovered(&scroll),
        Some(nested),
        "the content travelled under the pointer, so the pointer is over it"
    );
    assert_eq!(
        scroll
            .world
            .session
            .last_scroll_hit_index_work()
            .scroll_rows_displaced(),
        1,
        "the pose displaced exactly the row it moved"
    );
    let _ = scroll.world.session.shutdown();
}

/// A published settle pays its accepted sample out over several frames. Hover
/// follows the accepted sample, never the target the content is travelling
/// toward: the owner keeps the pointer until the content has actually arrived
/// under it.
#[test]
fn a_settle_hands_the_pointer_over_only_once_the_content_has_arrived() {
    let mut scroll =
        ScrollWorld::publish_with_nested_content(World::launch_with_scroll(smooth_scroll(true)));
    rest_pointer_on_the_component(&mut scroll);
    let owner = scroll.world.instances[0];
    let nested = scroll.world.instances[2];

    assert!(matches!(
        scroll.wheel(ONE_NOTCH, one_notch_up(), 5),
        UiHostScrollObservationOutcome::Applied(_)
    ));
    assert_eq!(
        scroll.accepted_offset(),
        block(0),
        "a smooth notch stages a target and moves nothing"
    );
    assert_eq!(
        hovered(&scroll),
        Some(owner),
        "hover never resolves against the offset the content is heading for"
    );

    settle_frame(&mut scroll, 6);
    assert_ne!(
        scroll.accepted_offset(),
        block(TRAVEL_POINTS),
        "one frame of a multi-frame settle is not the whole travel"
    );
    assert_eq!(
        hovered(&scroll),
        Some(owner),
        "content still on its way has not arrived under the pointer"
    );

    for elapsed in 2..=u64::from(SETTLE_TICKS) + 1 {
        settle_frame(&mut scroll, 5 + elapsed);
    }
    assert_eq!(scroll.accepted_offset(), block(TRAVEL_POINTS));
    assert_eq!(
        hovered(&scroll),
        Some(nested),
        "the settled content is what the pointer is over"
    );
    let _ = scroll.world.session.shutdown();
}
