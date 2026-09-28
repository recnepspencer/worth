//! What one host scroll delta asks to travel, in the unit the host counted it.
//!
//! A host delta is a pair of integers whose precision says what they count: a
//! distance in subpixels, or lines or pages at the host's subpixel scale. Only
//! a distance moves an offset. A count becomes one against the owner the
//! gesture latched to -- a line by the extent that owner declares, a page by
//! the viewport it shows -- so the three stay apart until that owner is known.

use std::cmp::Ordering;

/// Offset-direction travel: a positive component moves the offset forward.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiScrollHostTravel {
    /// A distance, in subpixels.
    Distance(super::UiScrollDelta),
    /// Lines, in thousandths.
    Lines(super::transition::UiScrollWheelLineDelta),
    /// Pages, in thousandths.
    Pages {
        inline_milli_pages: i64,
        block_milli_pages: i64,
    },
}

/// Which way a gesture asks each axis to travel, whatever it travels in.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiScrollHeading {
    inline: Ordering,
    block: Ordering,
}

impl UiScrollHostTravel {
    /// The travel a host delta of `host` units carries under `precision`. The
    /// host signs a delta the way the reader pushed the content; an offset
    /// moves the other way, so the sign turns here, once. `None` when a
    /// component has no counterpart in offset direction.
    pub(crate) fn from_host(
        precision: worth_ui_host_contract::UiHostScrollDeltaPrecision,
        [x, y]: [i64; 2],
    ) -> Option<Self> {
        let (inline, block) = (x.checked_neg()?, y.checked_neg()?);
        let count = crate::units::host_count_thousandths;
        Some(match precision {
            worth_ui_host_contract::UiHostScrollDeltaPrecision::Pixel => {
                Self::Distance(super::UiScrollDelta::new(inline, block))
            }
            worth_ui_host_contract::UiHostScrollDeltaPrecision::Line { .. } => Self::Lines(
                super::transition::UiScrollWheelLineDelta::new(count(inline)?, count(block)?),
            ),
            worth_ui_host_contract::UiHostScrollDeltaPrecision::Page => Self::Pages {
                inline_milli_pages: count(inline)?,
                block_milli_pages: count(block)?,
            },
        })
    }

    pub(crate) fn heading(self) -> UiScrollHeading {
        match self {
            Self::Distance(delta) => {
                UiScrollHeading::new(delta.inline_subpixels(), delta.block_subpixels())
            }
            Self::Lines(lines) => lines.heading(),
            Self::Pages {
                inline_milli_pages,
                block_milli_pages,
            } => UiScrollHeading::new(inline_milli_pages, block_milli_pages),
        }
    }
}

impl UiScrollHeading {
    pub(crate) fn new(inline: i64, block: i64) -> Self {
        Self {
            inline: inline.cmp(&0),
            block: block.cmp(&0),
        }
    }

    pub(crate) const fn inline(self) -> Ordering {
        self.inline
    }

    pub(crate) const fn block(self) -> Ordering {
        self.block
    }
}

/// Pages against an owner whose page step on each axis is `page_step`
/// subpixels. `None` when the product leaves the representable range.
pub(crate) fn page_travel(
    [inline_milli_pages, block_milli_pages]: [i64; 2],
    [inline_step, block_step]: [i64; 2],
) -> Option<super::UiScrollDelta> {
    let axis = |step: i64, milli_pages: i64| {
        crate::units::UiSubpixels::new(step)
            .thousandths(milli_pages)
            .map(crate::units::UiSubpixels::count)
    };
    Some(super::UiScrollDelta::new(
        axis(inline_step, inline_milli_pages)?,
        axis(block_step, block_milli_pages)?,
    ))
}

#[cfg(test)]
#[path = "host_travel_tests.rs"]
mod tests;
