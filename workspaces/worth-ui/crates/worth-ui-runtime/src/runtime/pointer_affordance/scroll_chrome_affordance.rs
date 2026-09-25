//! Which scroll chrome a pointer is over, and which region owns the answer.
//!
//! Scroll chrome is painted from derived facts rather than from mounted nodes,
//! so the presented hit index cannot answer for it. This file is the one place
//! a surface point becomes a chrome answer, and every consumer — pressing,
//! hovering, and routing a wheel over the gutter — asks it rather than
//! re-deriving rectangles of its own.
//!
//! The answer always names the owning region, including for the reserved
//! corner. A corner press reaches no axis, but a wheel there still belongs to
//! the region the gutter was reserved out of.

use crate::runtime::scroll::chrome::{UiScrollChromeAxis, UiScrollChromeFacts, UiScrollChromePart};

/// One region occurrence's chrome offered to pointer resolution.
#[derive(Clone, Copy, Debug)]
pub(crate) struct UiScrollChromeRegionTarget<'a> {
    pub(crate) owner: crate::runtime::scroll::UiScrollOwnerIdentity,
    pub(crate) facts: &'a UiScrollChromeFacts,
}

/// The part of one axis's chrome a press landed on.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiScrollChromePartAnswer {
    axis: UiScrollChromeAxis,
    part: UiScrollChromePart,
}

/// What the pointer found. `Corner` carries an owner but no axis, which is how
/// a corner press reaches neither scrollbar while a corner wheel still reaches
/// the region.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiScrollChromePointerAnswer {
    owner: crate::runtime::scroll::UiScrollOwnerIdentity,
    part: Option<UiScrollChromePartAnswer>,
}

impl UiScrollChromePartAnswer {
    pub(crate) const fn axis(self) -> UiScrollChromeAxis {
        self.axis
    }

    pub(crate) const fn part(self) -> UiScrollChromePart {
        self.part
    }
}

impl UiScrollChromePointerAnswer {
    /// The region whose chrome the pointer is over. A wheel here scrolls this
    /// region rather than whatever content lies behind the gutter.
    pub(crate) const fn owner(self) -> crate::runtime::scroll::UiScrollOwnerIdentity {
        self.owner
    }

    /// `None` at the reserved corner, which belongs to no axis.
    pub(crate) const fn part(self) -> Option<UiScrollChromePartAnswer> {
        self.part
    }
}

/// The chrome answer for one surface point.
///
/// Regions arrive in paint order, so the last one that claims the point is the
/// one painted on top; a nested region's scrollbar therefore wins over the
/// scrollbar of the region containing it.
pub(crate) fn resolve_scroll_chrome_pointer(
    point: crate::mounting::presentation::UiPlatformPoint,
    regions: &[UiScrollChromeRegionTarget<'_>],
) -> Option<UiScrollChromePointerAnswer> {
    regions.iter().rev().find_map(|region| {
        region_answer(point, region.facts).map(|part| UiScrollChromePointerAnswer {
            owner: region.owner,
            part,
        })
    })
}

/// `None` when the point is outside this region's chrome; `Some(None)` when it
/// is on the reserved corner.
fn region_answer(
    point: crate::mounting::presentation::UiPlatformPoint,
    facts: &UiScrollChromeFacts,
) -> Option<Option<UiScrollChromePartAnswer>> {
    if let Some(axis) = facts.pointer_axis(point) {
        return Some(Some(axis_answer(point, axis, facts)?));
    }
    facts
        .corner()
        .filter(|corner| crate::runtime::scroll::chrome::rect_contains(*corner, point))
        .map(|_| None)
}

/// Within one axis, the effective thumb target claims the point first: it fills
/// the whole gutter alongside the thumb, so a press just beside the drawn bar
/// still grabs it rather than paging past it.
fn axis_answer(
    point: crate::mounting::presentation::UiPlatformPoint,
    axis: UiScrollChromeAxis,
    facts: &UiScrollChromeFacts,
) -> Option<UiScrollChromePartAnswer> {
    let axis_facts = facts.axis(axis)?;
    let on_thumb = axis_facts
        .effective_thumb_pointer_rect(axis)
        .is_some_and(|rect| crate::runtime::scroll::chrome::rect_contains(rect, point));
    Some(UiScrollChromePartAnswer {
        axis,
        part: if on_thumb {
            UiScrollChromePart::Thumb
        } else {
            UiScrollChromePart::Track
        },
    })
}
