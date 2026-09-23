//! The Recent activity card: its fixed chrome and the content that travels
//! with the Recent activity scroll owner.
mod columns;
mod rows;

use super::{surface, text, DashboardElement, DashboardScrollPanel};

pub(super) fn elements() -> Vec<DashboardElement> {
    let mut elements = vec![card(), heading(), rail()];
    elements.extend(rows::activity_rows());
    elements.extend(columns::activity_columns());
    elements
}

/// Marks one element as travelling with the Recent activity content. Fixed
/// chrome never carries this; it stays put while the rows move beneath it.
const fn scrolled(mut element: DashboardElement) -> DashboardElement {
    element.scroll_panel = Some(DashboardScrollPanel::RecentActivity);
    element
}

/// The card behind the list. Fixed: the region clips the content against it.
fn card() -> DashboardElement {
    surface(
        "activity_card",
        [266, 636, 812, 346],
        "raised_surface",
        12,
        true,
        2,
    )
}

/// The panel header. Fixed, as neighbouring panel headers are.
fn heading() -> DashboardElement {
    text(
        "activity_heading",
        "Recent activity",
        [291, 653, 360, 35],
        18,
        true,
        "primary_text",
    )
}

/// Inset of the rail from the first and last row band edges.
const RAIL_BAND_INSET: u16 = 14;
/// Content-local inline position of the rail, centred under the row dots.
const RAIL_LOCAL_X: u16 = 9;

/// The vertical rule the row dots sit on. It travels with the rows, so its
/// extent is the row table's, not a patched constant.
fn rail() -> DashboardElement {
    let top = rows::row_band_top(0) + RAIL_BAND_INSET;
    let bottom = rows::row_band_top(rows::PLATFORM_PULSE_ACTIVITY_ROW_COUNT as u16);
    scrolled(surface(
        "activity_rail",
        [RAIL_LOCAL_X, top, 1, bottom - top - RAIL_BAND_INSET],
        "structural_rule",
        0,
        false,
        3,
    ))
}
