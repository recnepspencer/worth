//! Which Scroll region a gesture over a region owner addresses.
//!
//! Scrolled content is laid out relative to its region's owner rather than
//! mounted beneath it, so a gesture over content traveling with a region
//! would address that region. A region owner keeps the gesture for its own
//! region instead, even when it is in turn content of an outer one. Every
//! component in this World owns a region, so each hit here is an owner's.

use super::scroll_pose_authority::ScrollWorld;

#[test]
fn a_region_owner_keeps_a_gesture_even_as_another_regions_content() {
    let ScrollWorld { world, .. } = ScrollWorld::launch_published();
    // The third component and the child occurrence are both laid out inside
    // the first component's region, and each owns a region of its own.
    let [primary, _, nested, _, child] = world.instances;
    let addressed = |instance| world.session.mounted.addressed_scroll_owner(instance);
    assert_eq!(
        addressed(primary),
        Some(primary),
        "a region owner addresses its own region"
    );
    for content in [nested, child] {
        assert_eq!(
            addressed(content),
            Some(content),
            "a region owner laid out as another region's content keeps its own region"
        );
    }
}
