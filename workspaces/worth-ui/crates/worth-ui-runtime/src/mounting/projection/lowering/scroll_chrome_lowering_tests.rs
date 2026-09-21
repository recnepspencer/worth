//! What one region's chrome lowers to, at Pulse's declared RecentActivity
//! extents.
//!
//! The rectangles are checked against the same plain arithmetic the chrome
//! tests use — a 12-point gutter, a 6-point bar centred in it, a shared corner
//! surrendered by both tracks — rather than against the derivation that
//! produced them.

use super::{lower_scroll_chrome, UiScrollChromeLoweringInput};
use crate::runtime::scroll::chrome::{
    UiScrollChromeAxis, UiScrollChromeAxisSupport, UiScrollChromeDragPosture, UiScrollChromeFacts,
    UiScrollChromeMetrics, UiScrollChromePart,
};
use crate::runtime::scroll::UiScrollPresentationDeviceScale;

const GUTTER: f32 = 12.0;
const VIEWPORT_WIDTH: f32 = 768.0;
const VIEWPORT_HEIGHT: f32 = 269.0;
const CONTENT_WIDTH: f32 = 1_560.0;
const CONTENT_HEIGHT: f32 = 672.0;
const SUBPIXELS_PER_POINT: f32 = 1_000.0;

const TRACK_ROLE: &str = "platform.pulse.appearance.scroll_track";
const THUMB_ROLE: &str = "platform.pulse.appearance.scroll_thumb";

fn canonical(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> worth_ui_host_contract::UiMountedCanonicalBox {
    worth_ui_host_contract::UiMountedCanonicalBox::canonicalize(
        worth_ui_host_contract::UiMountedCanonicalBoxInput {
            x,
            y,
            width,
            height,
            coordinate_space: worth_ui_host_contract::UiMountedCoordinateSpace::Viewport,
        },
    )
    .expect("canonical test box")
}

fn role(name: &str) -> worth_ui_dsl::UiAppearanceRoleIdentity {
    worth_ui_dsl::UiAppearanceRoleIdentity::new(name).expect("declared Pulse chrome role")
}

fn facts(block_offset_points: f32) -> UiScrollChromeFacts {
    UiScrollChromeFacts::derive(
        canonical(0.0, 0.0, VIEWPORT_WIDTH, VIEWPORT_HEIGHT),
        crate::runtime::scroll::UiScrollBounds::new(
            ((CONTENT_WIDTH - VIEWPORT_WIDTH) * SUBPIXELS_PER_POINT) as i64,
            ((CONTENT_HEIGHT - VIEWPORT_HEIGHT) * SUBPIXELS_PER_POINT) as i64,
        )
        .expect("both axes overflow"),
        crate::runtime::scroll::UiScrollOffset::new(
            0,
            (block_offset_points * SUBPIXELS_PER_POINT) as i64,
        )
        .expect("non-negative offset"),
        &crate::runtime::scroll::chrome::UiScrollAdmittedChrome::admit_declared_chrome(
            UiScrollChromeAxisSupport::Both,
            role(TRACK_ROLE),
            role(THUMB_ROLE),
            UiScrollChromeMetrics::declared(),
        )
        .expect("the declared Pulse roles differ"),
    )
    .expect("RecentActivity overflows on both axes")
}

fn input<'input>(
    facts: &'input UiScrollChromeFacts,
    track_role: &'input worth_ui_dsl::UiAppearanceRoleIdentity,
    thumb_role: &'input worth_ui_dsl::UiAppearanceRoleIdentity,
    clip: worth_ui_host_contract::UiMountedCanonicalBox,
    device_scale_milli: u32,
) -> UiScrollChromeLoweringInput<'input> {
    UiScrollChromeLoweringInput {
        owner_instance: worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound()
            .expect("test instance"),
        facts,
        track_role,
        thumb_role,
        clip,
        device_scale: UiScrollPresentationDeviceScale::admit(device_scale_milli)
            .expect("a positive scale names a grid"),
        hovered: None,
        drag: None,
    }
}

/// Both axes lower, each as its track and then its thumb, and each part takes
/// the role its own declaration named.
#[test]
fn every_overflowing_axis_lowers_its_track_under_its_thumb() {
    let facts = facts(0.0);
    let track_role = role(TRACK_ROLE);
    let thumb_role = role(THUMB_ROLE);
    let nodes = lower_scroll_chrome(input(
        &facts,
        &track_role,
        &thumb_role,
        canonical(0.0, 0.0, VIEWPORT_WIDTH, VIEWPORT_HEIGHT),
        UiScrollPresentationDeviceScale::UNSCALED_MILLI,
    ))
    .expect("the declared extents lower");

    assert_eq!(
        nodes
            .iter()
            .map(|node| (node.axis(), node.part()))
            .collect::<Vec<_>>(),
        vec![
            (UiScrollChromeAxis::Inline, UiScrollChromePart::Track),
            (UiScrollChromeAxis::Inline, UiScrollChromePart::Thumb),
            (UiScrollChromeAxis::Block, UiScrollChromePart::Track),
            (UiScrollChromeAxis::Block, UiScrollChromePart::Thumb),
        ]
    );
    for node in &nodes {
        let expected = match node.part() {
            UiScrollChromePart::Track => &track_role,
            UiScrollChromePart::Thumb => &thumb_role,
        };
        assert_eq!(node.role(), expected);
    }
    let block_track = nodes
        .iter()
        .find(|node| {
            node.axis() == UiScrollChromeAxis::Block && node.part() == UiScrollChromePart::Track
        })
        .expect("the block axis lowered");
    assert_eq!(block_track.rect().x(), VIEWPORT_WIDTH - GUTTER);
    assert_eq!(block_track.rect().width(), GUTTER);
    assert_eq!(
        block_track.rect().height(),
        VIEWPORT_HEIGHT - GUTTER,
        "the block track surrenders the shared corner"
    );
}

/// Every painted edge lands on the device grid of the frame's own scale, so a
/// thumb that derived to a fraction of a point still paints sharp.
#[test]
fn painted_parts_land_on_the_device_grid() {
    let facts = facts(137.0);
    let track_role = role(TRACK_ROLE);
    let thumb_role = role(THUMB_ROLE);
    for (scale, pixels_per_point) in [(1_000_u32, 1.0_f32), (2_000, 2.0), (1_500, 1.5)] {
        let nodes = lower_scroll_chrome(input(
            &facts,
            &track_role,
            &thumb_role,
            canonical(0.0, 0.0, VIEWPORT_WIDTH, VIEWPORT_HEIGHT),
            scale,
        ))
        .expect("the declared extents lower");
        for node in nodes {
            let rect = node.rect();
            for edge in [
                rect.x(),
                rect.y(),
                rect.x() + rect.width(),
                rect.y() + rect.height(),
            ] {
                let pixels = edge * pixels_per_point;
                assert!(
                    (pixels - pixels.round()).abs() < 0.01,
                    "{:?} {:?} edge {edge} is off the {scale} grid",
                    node.axis(),
                    node.part()
                );
            }
        }
    }
}

/// Chrome is clipped to the region that reserved it. A viewport too short to
/// show its own inline gutter paints no inline chrome rather than painting it
/// past its own edge.
#[test]
fn a_part_the_region_cannot_show_is_not_painted() {
    let facts = facts(0.0);
    let track_role = role(TRACK_ROLE);
    let thumb_role = role(THUMB_ROLE);
    let nodes = lower_scroll_chrome(input(
        &facts,
        &track_role,
        &thumb_role,
        canonical(0.0, 0.0, VIEWPORT_WIDTH, VIEWPORT_HEIGHT - GUTTER),
        UiScrollPresentationDeviceScale::UNSCALED_MILLI,
    ))
    .expect("the declared extents lower");

    assert!(
        nodes
            .iter()
            .all(|node| node.axis() == UiScrollChromeAxis::Block),
        "the inline gutter sits outside the clip"
    );
    for node in &nodes {
        assert!(
            node.clip().y() + node.clip().height() <= VIEWPORT_HEIGHT - GUTTER + 0.01,
            "a painted part reaches past the region"
        );
    }
}

/// A drag owns both parts of its own axis and none of the other's, and a
/// pointer dragged off the gutter still reports the press held.
#[test]
fn a_drag_holds_its_own_axis_and_leaves_the_other_idle() {
    let facts = facts(0.0);
    let track_role = role(TRACK_ROLE);
    let thumb_role = role(THUMB_ROLE);
    let mut lowering = input(
        &facts,
        &track_role,
        &thumb_role,
        canonical(0.0, 0.0, VIEWPORT_WIDTH, VIEWPORT_HEIGHT),
        UiScrollPresentationDeviceScale::UNSCALED_MILLI,
    );
    lowering.drag = Some((
        UiScrollChromeAxis::Block,
        UiScrollChromeDragPosture::OutsideGutter,
    ));
    lowering.hovered = Some((UiScrollChromeAxis::Inline, UiScrollChromePart::Thumb));
    let nodes = lower_scroll_chrome(lowering).expect("the declared extents lower");

    for node in &nodes {
        let expected = match (node.axis(), node.part()) {
            (UiScrollChromeAxis::Block, _) => (
                worth_ui_dsl::UiAppearanceAxisClass::HoverOutside,
                worth_ui_dsl::UiAppearanceAxisClass::PressedCapturedOutside,
            ),
            (UiScrollChromeAxis::Inline, UiScrollChromePart::Thumb) => (
                worth_ui_dsl::UiAppearanceAxisClass::Hovered,
                worth_ui_dsl::UiAppearanceAxisClass::PressedIdle,
            ),
            (UiScrollChromeAxis::Inline, UiScrollChromePart::Track) => (
                worth_ui_dsl::UiAppearanceAxisClass::HoverOutside,
                worth_ui_dsl::UiAppearanceAxisClass::PressedIdle,
            ),
        };
        assert_eq!(
            (node.appearance().hover(), node.appearance().pressed()),
            expected,
            "{:?} {:?}",
            node.axis(),
            node.part()
        );
    }
}

/// The block thumb sits where plain proportion puts it for the accepted
/// offset: one Pulse notch (three 20-point lines) moves it its share of the
/// travel, and the far end of the content leaves it flush with the end of its
/// own track rather than past it.
#[test]
fn the_block_thumb_follows_the_accepted_offset_along_its_track() {
    let track_role = role(TRACK_ROLE);
    let thumb_role = role(THUMB_ROLE);
    let max_offset = CONTENT_HEIGHT - VIEWPORT_HEIGHT;
    let track_length = VIEWPORT_HEIGHT - GUTTER;
    let thumb_length = (track_length * VIEWPORT_HEIGHT / CONTENT_HEIGHT).max(24.0);
    let travel = track_length - thumb_length;
    for offset in [60.0_f32, max_offset] {
        let facts = facts(offset);
        let nodes = lower_scroll_chrome(input(
            &facts,
            &track_role,
            &thumb_role,
            canonical(0.0, 0.0, VIEWPORT_WIDTH, VIEWPORT_HEIGHT),
            UiScrollPresentationDeviceScale::UNSCALED_MILLI,
        ))
        .expect("the declared extents lower");
        let thumb = nodes
            .iter()
            .find(|node| {
                node.axis() == UiScrollChromeAxis::Block && node.part() == UiScrollChromePart::Thumb
            })
            .expect("the block thumb lowered");
        let expected_top = travel * offset / max_offset;
        assert!(
            (thumb.rect().y() - expected_top).abs() <= 0.5,
            "offset {offset}: thumb top {} is not the snapped {expected_top}",
            thumb.rect().y()
        );
        assert!(
            (thumb.rect().height() - thumb_length).abs() <= 1.0,
            "offset {offset}: thumb length {} is not the snapped {thumb_length}",
            thumb.rect().height()
        );
        let bottom = thumb.rect().y() + thumb.rect().height();
        assert!(
            bottom <= track_length + 0.001,
            "offset {offset}: thumb bottom {bottom} leaves the {track_length} track"
        );
        if offset == max_offset {
            assert!(
                (bottom - track_length).abs() <= 0.5,
                "at the far end the thumb is flush with the track end: {bottom}"
            );
        }
    }
}
