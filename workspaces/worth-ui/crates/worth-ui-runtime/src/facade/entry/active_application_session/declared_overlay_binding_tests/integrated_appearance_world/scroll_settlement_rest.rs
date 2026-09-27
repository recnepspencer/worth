//! An accepted Scroll sample settles to its displacement from where the
//! content rests, wherever layout puts that rest.

use super::super::super::UiAcceptedScrollSettlementDenial;
use super::scroll_pose_authority::{block, ScrollWorld};
use crate::mounting::presentation::displayed_rect_for_test;
use crate::runtime::motion::UiMotionTargetIdentity;
use crate::runtime::scroll::UiScrollChainEntry;

#[test]
fn an_accepted_sample_settles_to_its_displacement_from_a_nonzero_rest() {
    let scroll = ScrollWorld::launch_published();
    let (target, surface) = (scroll.target(), scroll.surface());
    let (owner_instance, region) = scroll
        .world
        .session
        .mounted
        .scroll_region_geometry(target, 0)
        .expect("the scrollable region has geometry");
    let content = region.in_layout_space().content();
    assert_eq!(owner_instance, target);
    assert!(
        content.x() > 0.0 && content.y() > 0.0,
        "the content box rests off the origin: {content:?}"
    );
    let key =
        super::super::super::scroll_transition_preparation::scroll_motion_owner_key(scroll.owner);
    let motion_target = UiMotionTargetIdentity::from_scroll_region_owner(surface, target, key);
    let owner = scroll
        .world
        .session
        .scroll_settlement_reading()
        .owner(motion_target, surface)
        .expect("this surface's Scroll content resolves")
        .expect("this surface's Scroll content is not foreign");
    assert_eq!(owner.owner_instance, target);
    assert_eq!(owner.region_instance, target);
    assert_eq!(owner.slot, 0);
    assert_eq!(
        owner.entry,
        UiScrollChainEntry::new(scroll.owner, scroll.incarnation)
    );
    let settle = |dy: f32| {
        let [x, y, width, height] = [content.x(), content.y(), content.width(), content.height()];
        owner.settle(displayed_rect_for_test(
            [x, y + dy, width, height],
            content.coordinate_space(),
        ))
    };

    // Content displayed twelve points above rest is content scrolled by twelve
    // points, whatever absolute position rest happens to be.
    let scrolled = settle(-12.0).expect("a sample above rest settles");
    assert_eq!(scrolled.offset.settled(), block(12));
    assert_eq!(scrolled.owner.owner_instance, target);
    let at_rest = settle(0.0).expect("a sample at rest settles");
    assert_eq!(at_rest.offset.settled(), block(0));
    assert!(
        matches!(
            settle(12.0),
            Err(UiAcceptedScrollSettlementDenial::SampleBeforeRest)
        ),
        "content displayed below rest names no offset"
    );
    assert!(
        matches!(
            scroll.world.session.scroll_settlement_reading().owner(
                UiMotionTargetIdentity::from_mounted_owner(surface, target, key),
                surface,
            ),
            Ok(None)
        ),
        "an ordinary Motion target is not Scroll content"
    );
    let _ = scroll.world.session.shutdown();
}
