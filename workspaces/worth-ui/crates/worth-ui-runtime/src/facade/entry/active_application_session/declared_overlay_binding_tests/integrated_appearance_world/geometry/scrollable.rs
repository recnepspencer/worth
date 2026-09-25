//! The World relaid out so its first component's region actually scrolls, and
//! the variants of that layout the scrolling scenarios need.
//!
//! The baseline layout next door gives each region a box the size of the
//! component that owns it, which is a region with nothing to scroll. Travel is
//! the difference between the two: shrink the region and the content its owner
//! carries no longer fits, and how far the reader can go is how much of it is
//! left over. Every layout here states that difference, moves the content
//! across it, or takes it away again.

use super::{install_with_child_region, RegionOverrides, BOXES};
use crate::facade::WorthUiActiveApplicationSession;
use worth_ui_host_contract::{UiMountedInstanceIdentity, UiSemanticSurfaceIdentity};

/// The first component's region in its own local space when the World scrolls:
/// half the owner's height, so its content has thirty points of block travel.
pub(in super::super) const SCROLLABLE_PRIMARY_REGION: [f32; 4] = [0.0, 0.0, 180.0, 30.0];

/// Reinstall the launched geometry with the first component's region smaller
/// than its box, so a wheel over it has somewhere to go.
pub(in super::super) fn install_scrollable_primary(
    session: &mut WorthUiActiveApplicationSession,
    surfaces: [UiSemanticSurfaceIdentity; 2],
    instances: [UiMountedInstanceIdentity; 5],
) {
    install_with_child_region(
        session,
        surfaces,
        instances,
        20,
        BOXES,
        RegionOverrides {
            child: None,
            primary: Some(SCROLLABLE_PRIMARY_REGION),
            nested: None,
            detach_child: false,
        },
        None,
        super::VIEWPORT,
    );
}

/// The third component's box once it is laid out inside the first component's
/// scrollable region, in that component's local space: far enough down that a
/// point five points below the component's top edge is above it at rest, and
/// short enough that ten points of travel carry it over that point and no
/// further than under it.
pub(in super::super) const NESTED_CONTENT_BOX: [f32; 4] = [8.0, 12.0, 160.0, 8.0];

/// The scrollable World with the third component laid out inside the first
/// component's region, so the region owns content a pointer can reach and that
/// content travels when the region scrolls. The third component sits in front
/// of the first in the authored hit order, so a point inside both is the
/// third's.
pub(in super::super) fn install_scrollable_primary_with_nested_content(
    session: &mut WorthUiActiveApplicationSession,
    surfaces: [UiSemanticSurfaceIdentity; 2],
    instances: [UiMountedInstanceIdentity; 5],
) {
    let mut boxes = BOXES;
    boxes[2] = NESTED_CONTENT_BOX;
    install_with_child_region(
        session,
        surfaces,
        instances,
        21,
        boxes,
        RegionOverrides {
            child: None,
            primary: Some(SCROLLABLE_PRIMARY_REGION),
            nested: Some(2),
            detach_child: false,
        },
        None,
        super::VIEWPORT,
    );
}

/// How far the region's content moves when rows are inserted above it. Small
/// enough that the region still has travel left once the offset follows.
pub(in super::super) const INSERTED_POINTS: f32 = 6.0;

/// The nested World relaid out as if rows had been inserted at the top of the
/// region's content: everything that region carries begins `INSERTED_POINTS`
/// further down inside it, and nothing else about the surface changes.
pub(in super::super) fn install_scrollable_primary_with_inserted_content(
    session: &mut WorthUiActiveApplicationSession,
    surfaces: [UiSemanticSurfaceIdentity; 2],
    instances: [UiMountedInstanceIdentity; 5],
) {
    let mut boxes = BOXES;
    boxes[2] = NESTED_CONTENT_BOX;
    boxes[2][1] += INSERTED_POINTS;
    boxes[4][1] += INSERTED_POINTS;
    install_with_child_region(
        session,
        surfaces,
        instances,
        22,
        boxes,
        RegionOverrides {
            child: None,
            primary: Some(SCROLLABLE_PRIMARY_REGION),
            nested: Some(2),
            detach_child: false,
        },
        None,
        super::VIEWPORT,
    );
}

/// The nested World relaid out with the child occurrence laid out against the
/// surface, so the region stops carrying the one occurrence it has held since
/// launch, and with the content that remains moved by the same distance the
/// insertion would have moved it. An owner that rebased against whichever item
/// remained would follow that move; one with nothing left to measure has to
/// clamp instead.
pub(in super::super) fn install_scrollable_primary_without_the_anchored_content(
    session: &mut WorthUiActiveApplicationSession,
    surfaces: [UiSemanticSurfaceIdentity; 2],
    instances: [UiMountedInstanceIdentity; 5],
) {
    let mut boxes = BOXES;
    boxes[2] = NESTED_CONTENT_BOX;
    boxes[2][1] += INSERTED_POINTS;
    install_with_child_region(
        session,
        surfaces,
        instances,
        23,
        boxes,
        RegionOverrides {
            child: None,
            primary: Some(SCROLLABLE_PRIMARY_REGION),
            nested: Some(2),
            detach_child: true,
        },
        None,
        super::VIEWPORT,
    );
}

/// The first component relaid out no taller than the region showing it, so the
/// content that region was scrolling now fits inside it. Travel is what is left
/// over, and nothing is: the region has nowhere left to go, and a settle still
/// walking toward somewhere has nowhere left to arrive.
pub(in super::super) fn install_scrollable_primary_with_collapsed_content(
    session: &mut WorthUiActiveApplicationSession,
    surfaces: [UiSemanticSurfaceIdentity; 2],
    instances: [UiMountedInstanceIdentity; 5],
) {
    let mut boxes = BOXES;
    boxes[0][3] = SCROLLABLE_PRIMARY_REGION[3];
    install_with_child_region(
        session,
        surfaces,
        instances,
        24,
        boxes,
        RegionOverrides {
            child: None,
            primary: Some(SCROLLABLE_PRIMARY_REGION),
            nested: None,
            detach_child: false,
        },
        None,
        super::VIEWPORT,
    );
}

/// Five points of legal travel remain, so a live target above five must be
/// retargeted rather than handled by the empty-extent cancellation path.
pub(in super::super) fn install_scrollable_primary_with_shorter_content(
    session: &mut WorthUiActiveApplicationSession,
    surfaces: [UiSemanticSurfaceIdentity; 2],
    instances: [UiMountedInstanceIdentity; 5],
) {
    install_scrollable_primary_with_travel(session, surfaces, instances, 25, 5.0);
}

/// The nested World relaid out at `revision` with `travel` points of block
/// travel: the first component is exactly that much taller than the region
/// showing it, and the nested component keeps its place inside the region.
pub(in super::super) fn install_scrollable_primary_with_travel(
    session: &mut WorthUiActiveApplicationSession,
    surfaces: [UiSemanticSurfaceIdentity; 2],
    instances: [UiMountedInstanceIdentity; 5],
    revision: u64,
    travel: f32,
) {
    let mut boxes = BOXES;
    boxes[0][3] = SCROLLABLE_PRIMARY_REGION[3] + travel;
    boxes[2] = NESTED_CONTENT_BOX;
    install_with_child_region(
        session,
        surfaces,
        instances,
        revision,
        boxes,
        RegionOverrides {
            child: None,
            primary: Some(SCROLLABLE_PRIMARY_REGION),
            nested: Some(2),
            detach_child: false,
        },
        None,
        super::VIEWPORT,
    );
}
