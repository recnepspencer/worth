use super::geometry::scrollable::install_scrollable_primary_with_shorter_content;
use super::scroll_pose_authority::ScrollWorld;
use super::scroll_settle_commit::smooth_scroll;
use super::World;
use crate::facade::entry::active_application_session::scroll_chrome_interaction::{
    UiScrollChromeInteractionDenial, UiScrollChromePressOutcome,
};
use crate::runtime::scroll::chrome::{UiScrollChromeAxis, UiScrollChromePart};
use worth_ui_host_contract::*;

#[test]
fn rejected_extent_keeps_presented_chrome_hit_geometry_and_defers_control_until_retry() {
    let mut declared = smooth_scroll(true);
    declared.region = declared
        .region
        .clone()
        .with_scroll_chrome(super::scroll_chrome_fixture::contract());
    let mut scroll = ScrollWorld::publish_with_nested_content(World::launch_with_scroll(declared));
    let surface = scroll.surface();
    let previous = scroll.presentation();
    let regions = scroll.world.session.scroll_chrome_facts(surface);
    let original = regions
        .iter()
        .find(|region| region.owner() == scroll.owner)
        .unwrap();
    let thumb = original
        .facts()
        .axis(UiScrollChromeAxis::Block)
        .unwrap()
        .thumb();
    let capture = UiHostPointerCaptureEpoch::new(7);
    let pointer = UiHostPointerIdentity::new(1);
    let grabbed = [
        thumb.x() + thumb.width() / 2.0,
        thumb.y() + thumb.height() / 2.0,
    ];
    assert!(matches!(
        scroll
            .world
            .session
            .press_scroll_chrome(surface, grabbed, pointer, capture, previous,),
        Ok(UiScrollChromePressOutcome::ThumbCaptured(_))
    ));
    let before = scroll.accepted_offset();
    install_scrollable_primary_with_shorter_content(
        &mut scroll.world.session,
        scroll.world.surfaces,
        scroll.world.instances,
    );
    let candidate = scroll.world.session.scroll_chrome_facts(surface);
    let larger = candidate
        .iter()
        .find(|region| region.owner() == scroll.owner)
        .unwrap()
        .facts()
        .axis(UiScrollChromeAxis::Block)
        .unwrap()
        .thumb();
    assert!(larger.height() > thumb.height());
    let point = [
        grabbed[0],
        thumb.y() + (thumb.height() + larger.height()) / 2.0,
    ];
    assert_eq!(
        scroll
            .world
            .session
            .prepared_scroll_chrome_under_pointer(surface, point)
            .unwrap()
            .part()
            .unwrap()
            .part(),
        UiScrollChromePart::Thumb,
        "candidate appearance hover is prepared for the new thumb, without authorizing input there"
    );
    let frame = scroll.world.prepare_surface(surface);
    scroll.world.host.push_rejected();
    let outcome = scroll
        .world
        .session
        .present_prepared_mounted_frame_internal(
            frame,
            UiPresentationDeadline::at_tick(u64::MAX),
            4,
        );
    match outcome {
        crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(_) => {}
        crate::mounting::UiMountedFrameOutcome::AdmissionDenied(denial) => {
            panic!("extent did not reach the host: {:?}", denial.denial())
        }
        other => panic!(
            "expected host refusal, got {:?}",
            std::mem::discriminant(&other)
        ),
    }
    assert_eq!(scroll.presentation(), previous);
    let answer = scroll
        .world
        .session
        .scroll_chrome_under_pointer(surface, point)
        .unwrap();
    assert_eq!(
        answer.part().unwrap().part(),
        UiScrollChromePart::Track,
        "the rejected candidate's larger thumb cannot claim a point on the displayed track"
    );
    assert_eq!(
        scroll
            .world
            .session
            .press_scroll_chrome(surface, point, pointer, capture, previous),
        Err(UiScrollChromeInteractionDenial::UnpresentedLayout)
    );
    assert_eq!(
        scroll
            .world
            .session
            .drag_scroll_chrome([grabbed[0], grabbed[1] + 2.0], pointer, capture),
        Err(UiScrollChromeInteractionDenial::UnpresentedLayout)
    );
    assert_eq!(scroll.accepted_offset(), before);
    assert!(
        scroll
            .world
            .session
            .release_scroll_chrome(pointer, capture)
            .is_ok(),
        "pending layout never traps an existing capture"
    );

    let frame = scroll.world.prepare_surface(surface);
    scroll.world.publish(frame, 5, false);
    let accepted = scroll.presentation();
    assert_ne!(accepted, previous);
    let answer = scroll
        .world
        .session
        .scroll_chrome_under_pointer(surface, point)
        .unwrap();
    assert_eq!(answer.part().unwrap().part(), UiScrollChromePart::Thumb);
    assert!(matches!(
        scroll
            .world
            .session
            .press_scroll_chrome(surface, point, pointer, capture, accepted,),
        Ok(UiScrollChromePressOutcome::ThumbCaptured(_))
    ));
    assert!(scroll
        .world
        .session
        .release_scroll_chrome(pointer, capture)
        .is_ok());
    let _ = scroll.world.session.shutdown();
}
