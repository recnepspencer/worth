//! Which Scroll region a gesture over a mounted occurrence addresses.
//!
//! Scrolled content is laid out relative to its region's owner rather than
//! mounted beneath it, so a gesture over content that owns no region
//! addresses that region. A region owner keeps the gesture for its own
//! region instead, even when it is in turn content of an outer one. The
//! answer names which of the two it is, so owning a region and traveling as
//! one's content are never read as each other.

use super::scroll_pose_authority::ScrollWorld;
use super::session::World;
use crate::mounting::UiAddressedScrollOwner::{ContentOf, OwnsRegion};

#[test]
fn a_region_owner_keeps_a_gesture_even_as_another_regions_content() {
    let ScrollWorld { world, .. } = ScrollWorld::launch_published();
    // The third component and the child occurrence are both laid out inside
    // the first component's region, and each owns a region of its own.
    let [primary, _, nested, _, child] = world.instances;
    let addressed = |instance| world.session.mounted.addressed_scroll_owner(instance);
    assert_eq!(
        addressed(primary),
        Some(OwnsRegion(primary)),
        "a region owner addresses its own region"
    );
    for content in [nested, child] {
        assert_eq!(
            addressed(content),
            Some(OwnsRegion(content)),
            "a region owner laid out as another region's content keeps its own region"
        );
    }
}

/// The authored source with the component at `index` declaring no region.
fn without_region(source: String, index: usize) -> String {
    let component = super::authored::COMPONENTS[index];
    let declared = format!(
        "component {component} {{ appearance {{ role overlay.content{index} }} region workspace.region.primary {{ sizing workspace.sizing.mosaic_support; }} }}"
    );
    assert!(
        source.contains(&declared),
        "the authored source no longer declares {component}'s region this way"
    );
    source.replace(
        &declared,
        &format!("component {component} {{ appearance {{ role overlay.content{index} }} }}"),
    )
}

#[test]
fn content_that_owns_no_region_addresses_the_region_it_travels_with() {
    // The third component owns no region and is laid out inside the first's,
    // so it is only that region's content. The second owns none either and
    // is laid out against the surface, so no region addresses it.
    let source = without_region(without_region(super::authored::source(), 2), 1);
    let ScrollWorld { world, .. } =
        ScrollWorld::publish_with_nested_content(World::launch_with_source(false, false, source));
    let [primary, peer, content, _, _] = world.instances;
    let addressed = |instance| world.session.mounted.addressed_scroll_owner(instance);
    assert_eq!(
        addressed(content),
        Some(ContentOf(primary)),
        "content that owns no region travels with the region it is laid out in"
    );
    assert_eq!(addressed(primary), Some(OwnsRegion(primary)));
    assert_eq!(
        addressed(peer),
        None,
        "an occurrence that neither owns a region nor travels with one addresses none"
    );
}
