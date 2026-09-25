//! The Recent activity card: its fixed chrome and the content that travels
//! with the Recent activity scroll owner.
mod columns;
mod rows;

use super::frame::Edge;
use super::page::Panel;
use super::{surface, text, DashboardElement, DashboardScrollPanel};

pub(super) fn elements() -> Vec<DashboardElement> {
    let mut elements = vec![card(), heading(), rail()];
    elements.extend(rows::activity_rows());
    elements.extend(columns::activity_columns());
    elements
}

/// Makes one element, authored from the content origin, travel with the
/// Recent activity content. Fixed chrome never does; it stays put while the
/// rows move beneath it.
fn scrolled(element: DashboardElement) -> DashboardElement {
    DashboardScrollPanel::RecentActivity.carry(element, Edge::Start)
}

/// The card behind the list, filling its panel. The region clips the content
/// against it.
fn card() -> DashboardElement {
    Panel::RecentActivity.frame().place(
        surface(
            "activity_card",
            [266, 636, 812, 346],
            "raised_surface",
            12,
            true,
            2,
        ),
        Edge::Both,
        Edge::Both,
    )
}

/// The panel header, kept at the panel's top left as neighboring panel
/// headers are.
fn heading() -> DashboardElement {
    Panel::RecentActivity.frame().place(
        text(
            "activity_heading",
            "Recent activity",
            [291, 653, 360, 35],
            18,
            true,
            "primary_text",
        ),
        Edge::Start,
        Edge::Start,
    )
}

/// Inset of the rail from the first and last row band edges.
const RAIL_BAND_INSET: u16 = 14;
/// Content-local inline position of the rail, centered under the row dots.
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
