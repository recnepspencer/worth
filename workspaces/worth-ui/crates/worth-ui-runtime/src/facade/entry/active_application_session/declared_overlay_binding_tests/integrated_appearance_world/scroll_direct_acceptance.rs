//! Direct intent crosses the ordinary physical frame boundary, not the Motion
//! sampler. Candidate geometry is deliberately not an accepted-output oracle.
use super::scroll_pose_authority::{block, ScrollWorld};
use super::scroll_settle_commit::{one_notch_up, smooth_scroll, ONE_NOTCH};
use super::scroll_settle_frame::settle_frame;
use super::World;
use crate::mounting::UiMountedFrameOutcome;
use crate::runtime::scroll::{UiHostScrollObservationOutcome, UiScrollDeltaCause};
use worth_ui_host_contract::*;

fn nested_hit(scroll: &ScrollWorld) -> UiMountedCanonicalBox {
    scroll
        .world
        .session
        .mounted
        .interaction_hit_test_basis(scroll.presentation())
        .unwrap()
        .rows()
        .iter()
        .find(|row| row.mounted_instance() == scroll.world.instances[2])
        .unwrap()
        .bounds()
}

fn thumb(scroll: &ScrollWorld) -> [f32; 4] {
    let bounds = scroll
        .world
        .session
        .presented_scroll_chrome_facts(scroll.surface())
        .iter()
        .find(|region| region.owner() == scroll.owner)
        .unwrap()
        .facts()
        .axis(crate::runtime::scroll::chrome::UiScrollChromeAxis::Block)
        .unwrap()
        .thumb();
    // Both admitted coordinate postures share logical client units; the
    // retained path spells Viewport explicitly, while layout spells HostSurface.
    [bounds.x(), bounds.y(), bounds.width(), bounds.height()]
}

#[test]
fn direct_thumb_refusal_retains_offset_hit_chrome_and_track_until_retry_acceptance() {
    let mut declared = smooth_scroll(true);
    declared.region = declared
        .region
        .clone()
        .with_scroll_chrome(super::scroll_chrome_fixture::contract());
    let mut scroll = ScrollWorld::publish_with_nested_content(World::launch_with_scroll(declared));
    assert!(matches!(
        scroll.wheel(ONE_NOTCH, one_notch_up(), 5),
        UiHostScrollObservationOutcome::Applied(_)
    ));
    settle_frame(&mut scroll, 6);
    settle_frame(&mut scroll, 7);
    let target = super::super::super::scroll_direct_control::scroll_content_motion_target(
        scroll.owner,
        scroll.target(),
    );
    let track = scroll
        .world
        .session
        .motion
        .as_ref()
        .unwrap()
        .committed_track(target)
        .unwrap();
    let previous = (
        scroll.accepted_offset(),
        scroll.presentation(),
        nested_hit(&scroll),
        thumb(&scroll),
    );
    scroll
        .world
        .session
        .place_scroll_chrome_offset(
            scroll.owner,
            scroll.incarnation,
            scroll.target(),
            0,
            block(10),
            UiScrollDeltaCause::ChromeThumbDrag,
        )
        .unwrap();
    assert_eq!(scroll.accepted_offset(), previous.0);
    assert_eq!(nested_hit(&scroll), previous.2);
    assert_eq!(thumb(&scroll), previous.3);
    let frame = scroll.world.prepare_surface(scroll.surface());
    let calls = scroll.world.host.presentation_calls();
    scroll.world.host.push_rejected();
    let outcome = scroll
        .world
        .session
        .present_prepared_mounted_frame_internal(
            frame,
            UiPresentationDeadline::at_tick(u64::MAX),
            8,
        );
    match outcome {
        UiMountedFrameOutcome::RejectedBeforeEffects(_) => {}
        UiMountedFrameOutcome::AdmissionDenied(denial) => {
            panic!("direct frame missed host: {:?}", denial.denial())
        }
        other => panic!(
            "expected host refusal: {:?}",
            std::mem::discriminant(&other)
        ),
    }
    assert_eq!(scroll.world.host.presentation_calls(), calls + 1);
    assert_eq!(
        (
            scroll.accepted_offset(),
            scroll.presentation(),
            nested_hit(&scroll),
            thumb(&scroll)
        ),
        previous
    );
    assert_eq!(
        scroll
            .world
            .session
            .motion
            .as_ref()
            .unwrap()
            .committed_track(target),
        Some(track)
    );
    scroll.publish_direct(9);
    assert_eq!(scroll.accepted_offset(), block(10));
    assert_ne!(scroll.presentation(), previous.1);
    assert_ne!(nested_hit(&scroll), previous.2);
    assert_ne!(thumb(&scroll), previous.3);
    assert!(scroll
        .world
        .session
        .motion
        .as_ref()
        .unwrap()
        .committed_track(target)
        .is_none());
    assert!(!scroll
        .world
        .session
        .mounted
        .has_pending_direct_scroll(scroll.surface()));
    let _ = scroll.world.session.shutdown();
}

#[test]
fn older_prepared_frame_cannot_acknowledge_newer_accumulated_direct_input() {
    let mut scroll = ScrollWorld::publish_with_nested_content(World::launch_without_motion());
    assert!(
        scroll.world.session.motion.as_ref().is_none(),
        "direct publication has no Motion authority to borrow"
    );
    assert!(matches!(
        scroll.wheel(UiHostScrollDeltaPrecision::Pixel, -5_000, 5),
        UiHostScrollObservationOutcome::Applied(_)
    ));
    let stale = scroll.world.prepare_surface(scroll.surface());
    assert!(matches!(
        scroll.wheel(UiHostScrollDeltaPrecision::Pixel, -5_000, 6),
        UiHostScrollObservationOutcome::Applied(_)
    ));
    let calls = scroll.world.host.presentation_calls();
    let outcome = scroll
        .world
        .session
        .present_prepared_mounted_frame_internal(
            stale,
            UiPresentationDeadline::at_tick(u64::MAX),
            7,
        );
    assert!(matches!(outcome, UiMountedFrameOutcome::AdmissionDenied(_)));
    assert_eq!(scroll.world.host.presentation_calls(), calls);
    assert_eq!(scroll.accepted_offset(), block(0));
    scroll.publish_direct(8);
    assert_eq!(scroll.accepted_offset(), block(10));
    let _ = scroll.world.session.shutdown();
}

#[test]
fn a_direct_frame_in_flight_keeps_predecessor_hits_until_matching_completion() {
    let mut scroll = ScrollWorld::publish_with_nested_content(World::launch_without_motion());
    let previous = (
        scroll.accepted_offset(),
        scroll.presentation(),
        nested_hit(&scroll),
    );
    assert!(matches!(
        scroll.wheel(UiHostScrollDeltaPrecision::Pixel, -10_000, 5),
        UiHostScrollObservationOutcome::Applied(_)
    ));
    let pending = scroll.hold_presentation_open(6);
    assert_eq!(
        (
            scroll.accepted_offset(),
            scroll.presentation(),
            nested_hit(&scroll)
        ),
        previous
    );
    assert!(matches!(
        scroll.wheel(UiHostScrollDeltaPrecision::Pixel, -5_000, 7),
        UiHostScrollObservationOutcome::Denied(
            crate::runtime::scroll::UiHostScrollObservationDenial::Geometry(
                crate::mounting::UiMountedOccurrenceGeometryDenial::PresentationInFlight
            )
        )
    ));
    scroll.complete(pending, 8);
    assert_eq!(scroll.accepted_offset(), block(10));
    assert_ne!(scroll.presentation(), previous.1);
    assert_ne!(nested_hit(&scroll), previous.2);
    let _ = scroll.world.session.shutdown();
}

#[test]
fn unmount_retires_unpublished_direct_intent_and_its_mounted_guard() {
    let mut scroll = ScrollWorld::publish_with_nested_content(World::launch_without_motion());
    assert!(matches!(
        scroll.wheel(UiHostScrollDeltaPrecision::Pixel, -10_000, 5),
        UiHostScrollObservationOutcome::Applied(_)
    ));
    assert!(scroll
        .world
        .session
        .mounted
        .has_pending_direct_scroll(scroll.surface()));
    assert!(scroll
        .world
        .session
        .scroll
        .as_ref()
        .unwrap()
        .has_pending_direct(scroll.surface()));
    scroll
        .world
        .session
        .unmount_instance(scroll.target())
        .unwrap();
    assert!(!scroll
        .world
        .session
        .mounted
        .has_pending_direct_scroll(scroll.surface()));
    assert!(!scroll
        .world
        .session
        .scroll
        .as_ref()
        .unwrap()
        .has_pending_direct(scroll.surface()));
    let _ = scroll.world.session.shutdown();
}

#[test]
fn pending_direct_geometry_defers_keyboard_focus_before_semantic_mutation() {
    let mut scroll = ScrollWorld::launch_published();
    let before = scroll
        .world
        .session
        .focus
        .as_ref()
        .unwrap()
        .current_semantic_focus();
    assert!(matches!(
        scroll.wheel(UiHostScrollDeltaPrecision::Pixel, -10_000, 5),
        UiHostScrollObservationOutcome::Applied(_)
    ));
    let mut publications = Vec::new();
    let presentation = scroll.presentation();
    assert!(scroll.world.session.observe_focus_navigation_report(
        &UiHostObservationPayload::Keyboard {
            logical_key: UiHostKey::Tab,
            physical_key: None,
            modifiers: UiHostKeyboardModifiers::default(),
            transition: UiHostKeyTransition::Pressed { repeat: false },
        },
        presentation,
        &mut publications,
    ));
    assert!(matches!(
        publications.as_slice(),
        [Err(
            crate::facade::entry::UiFocusPlacementExecutionDenial::UnpublishedScrollGeometry
        )]
    ));
    assert_eq!(
        scroll
            .world
            .session
            .focus
            .as_ref()
            .unwrap()
            .current_semantic_focus(),
        before
    );
    assert_eq!(scroll.accepted_offset(), block(0));
    scroll.publish_direct(6);
    assert_eq!(scroll.accepted_offset(), block(10));
    let _ = scroll.world.session.shutdown();
}

#[test]
fn a_direct_region_without_painted_children_accepts_only_its_no_paint_completion() {
    let mut scroll = ScrollWorld::publish(World::launch_without_motion());
    assert!(matches!(
        scroll.wheel(UiHostScrollDeltaPrecision::Pixel, -10_000, 5),
        UiHostScrollObservationOutcome::Applied(_)
    ));
    assert_eq!(scroll.accepted_offset(), block(0));
    let calls = scroll.world.host.presentation_calls();
    let frame = scroll.world.prepare_surface(scroll.surface());
    scroll.world.publish(frame, 6, false);
    assert_eq!(scroll.world.host.presentation_calls(), calls + 1);
    assert_eq!(scroll.accepted_offset(), block(10));
    assert!(!scroll
        .world
        .session
        .mounted
        .has_pending_direct_scroll(scroll.surface()));
    let _ = scroll.world.session.shutdown();
}
