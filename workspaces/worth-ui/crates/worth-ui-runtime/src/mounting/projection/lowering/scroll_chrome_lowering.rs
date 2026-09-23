//! Lowering one region's derived scroll chrome into painted parts.
//!
//! Chrome is not a mounted node: it has no graph node identity, no plan index
//! and no mounted instance of its own, so it cannot travel the node lowering
//! path. What it does have is everything a painted rectangle needs — a declared
//! appearance role, a rectangle derived from the accepted offset, the region's
//! clip, and the Hover/Pressed classes its role partitions on. This file
//! assembles exactly that, once, so every consumer paints the same rectangles
//! the pointer is hit-tested against.
//!
//! Chrome is snapped here, from its own derived rectangles. The rectangles
//! that leave this file sit on the device grid; the rectangles the pointer is
//! resolved against stay unsnapped, because hit testing answers from the
//! accepted offset rather than from the frame that offset was painted in. The
//! content easing underneath is snapped on the same grid where it is painted,
//! from the same accepted sample, so the thumb and the rows it measures move by
//! whole pixels together.

use crate::runtime::scroll::chrome::{
    UiScrollChromeAppearanceState, UiScrollChromeAxis, UiScrollChromeDragPosture,
    UiScrollChromeFacts, UiScrollChromePart,
};
use crate::runtime::scroll::{
    snap_to_device_grid, UiScrollPresentationDeviceScale, UiScrollPresentationSnappingDenial,
};

/// One painted chrome rectangle, ready for a host paint mechanic.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct UiMountedScrollChromeNode {
    owner_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    axis: UiScrollChromeAxis,
    part: UiScrollChromePart,
    role: worth_ui_dsl::UiAppearanceRoleIdentity,
    rect: worth_ui_host_contract::UiMountedCanonicalBox,
    clip: worth_ui_host_contract::UiMountedCanonicalBox,
    appearance: UiScrollChromeAppearanceState,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiScrollChromeLoweringDenial {
    /// The device scale named no grid, or a snapped rectangle left the range
    /// canonical mounted geometry admits.
    Snapping(UiScrollPresentationSnappingDenial),
}

/// What one region hands to lowering: its derived chrome, the roles that paint
/// it, the box it is clipped to, and the interaction posture its parts present.
pub(crate) struct UiScrollChromeLoweringInput<'input> {
    pub(crate) owner_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    pub(crate) facts: &'input UiScrollChromeFacts,
    pub(crate) track_role: &'input worth_ui_dsl::UiAppearanceRoleIdentity,
    pub(crate) thumb_role: &'input worth_ui_dsl::UiAppearanceRoleIdentity,
    /// The region's viewport box. Chrome sits inside the reserved gutter, so
    /// this normally contains it outright; it still clips, because a viewport
    /// narrower than its own gutter must not paint chrome outside itself.
    pub(crate) clip: worth_ui_host_contract::UiMountedCanonicalBox,
    pub(crate) device_scale: UiScrollPresentationDeviceScale,
    /// The part the pointer is over, if it is over one of this region's parts.
    pub(crate) hovered: Option<(UiScrollChromeAxis, UiScrollChromePart)>,
    /// The axis this region has a thumb drag on, and where that drag's pointer
    /// is.
    pub(crate) drag: Option<(UiScrollChromeAxis, UiScrollChromeDragPosture)>,
}

impl UiMountedScrollChromeNode {
    pub(crate) const fn owner_instance(&self) -> worth_ui_host_contract::UiMountedInstanceIdentity {
        self.owner_instance
    }

    pub(crate) const fn axis(&self) -> UiScrollChromeAxis {
        self.axis
    }

    pub(crate) const fn part(&self) -> UiScrollChromePart {
        self.part
    }

    pub(crate) const fn role(&self) -> &worth_ui_dsl::UiAppearanceRoleIdentity {
        &self.role
    }

    /// The rectangle to paint, already on the device grid.
    pub(crate) const fn rect(&self) -> worth_ui_host_contract::UiMountedCanonicalBox {
        self.rect
    }

    pub(crate) const fn clip(&self) -> worth_ui_host_contract::UiMountedCanonicalBox {
        self.clip
    }

    pub(crate) const fn appearance(&self) -> UiScrollChromeAppearanceState {
        self.appearance
    }
}

/// Every painted part of one region's chrome, in paint order.
///
/// An axis without travel contributes nothing at all: WP3's derivation already
/// refuses an axis whose maximum offset is zero, so a region that fits its
/// content presents no track and no thumb rather than an empty gutter.
pub(crate) fn lower_scroll_chrome(
    input: UiScrollChromeLoweringInput<'_>,
) -> Result<Vec<UiMountedScrollChromeNode>, UiScrollChromeLoweringDenial> {
    let mut nodes = Vec::new();
    for axis in [UiScrollChromeAxis::Inline, UiScrollChromeAxis::Block] {
        let Some(facts) = input.facts.axis(axis) else {
            continue;
        };
        for part in UiScrollChromePart::PAINT_ORDER {
            let rect = snap_to_device_grid(part.rect(facts), input.device_scale)
                .map_err(UiScrollChromeLoweringDenial::Snapping)?;
            let Some(_) = intersection(rect, input.clip) else {
                continue;
            };
            nodes.push(UiMountedScrollChromeNode {
                owner_instance: input.owner_instance,
                axis,
                part,
                role: part_role(part, input.track_role, input.thumb_role).clone(),
                rect,
                // Retain the stationary viewport, not the currently occupied
                // rectangle: a later accepted sample moves the thumb inside it.
                clip: input.clip,
                appearance: UiScrollChromeAppearanceState::resolve(
                    part,
                    input
                        .hovered
                        .filter(|(hovered_axis, _)| *hovered_axis == axis)
                        .map(|(_, hovered_part)| hovered_part),
                    input
                        .drag
                        .filter(|(drag_axis, _)| *drag_axis == axis)
                        .map(|(_, posture)| posture),
                ),
            });
        }
    }
    Ok(nodes)
}

const fn part_role<'role>(
    part: UiScrollChromePart,
    track_role: &'role worth_ui_dsl::UiAppearanceRoleIdentity,
    thumb_role: &'role worth_ui_dsl::UiAppearanceRoleIdentity,
) -> &'role worth_ui_dsl::UiAppearanceRoleIdentity {
    match part {
        UiScrollChromePart::Track => track_role,
        UiScrollChromePart::Thumb => thumb_role,
    }
}

/// The part of `rect` the region actually shows. `None` when the clip leaves
/// nothing, which is how a viewport too small for its own gutter paints no
/// chrome instead of painting it outside itself.
fn intersection(
    rect: worth_ui_host_contract::UiMountedCanonicalBox,
    clip: worth_ui_host_contract::UiMountedCanonicalBox,
) -> Option<worth_ui_host_contract::UiMountedCanonicalBox> {
    let left = rect.x().max(clip.x());
    let top = rect.y().max(clip.y());
    let right = (rect.x() + rect.width()).min(clip.x() + clip.width());
    let bottom = (rect.y() + rect.height()).min(clip.y() + clip.height());
    if right <= left || bottom <= top {
        return None;
    }
    worth_ui_host_contract::UiMountedCanonicalBox::canonicalize(
        worth_ui_host_contract::UiMountedCanonicalBoxInput {
            x: left,
            y: top,
            width: right - left,
            height: bottom - top,
            coordinate_space: rect.coordinate_space(),
        },
    )
    .ok()
}
