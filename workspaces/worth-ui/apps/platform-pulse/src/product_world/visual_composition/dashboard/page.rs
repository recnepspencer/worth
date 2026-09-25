//! The dashboard page: a fixed sidebar, a masthead that fills the width beside
//! it, and the main content laid out in flexible tracks.
//!
//! The page fills the stage beside the sidebar and below the masthead, and
//! keeps its content a 24-point gutter inside it. The content stacks the
//! greeting, the summary card row, and the panels, 20 points apart. From
//! the breakpoint up, the panels sit in two rows of two columns that share
//! the width 2:1, and never narrower than 480 and 320 points; below it, they
//! stack in one column in source order. When the rows' minimums outgrow the
//! stage, the page keeps them and scrolls.
use worth_ui::facade::declaration::{
    ComponentViewportAxisPlacement, ComponentViewportRegion, MosaicLayoutCell,
    MosaicLayoutContract, MosaicTrack,
};

use super::frame::Frame;
use super::{
    ContainerTracks, DashboardLayoutCell, DashboardPlacement, DashboardScrollOwner, StackedTracks,
};

/// The sidebar's width, which every screen width keeps.
pub const PLATFORM_PULSE_SIDEBAR_WIDTH: u16 = 236;
/// The masthead's height, down to and including its bottom rule.
pub const PLATFORM_PULSE_MASTHEAD_HEIGHT: u16 = 57;
/// The narrowest viewport, in logical points, that lays the panels and the
/// summary cards out side by side.
pub(super) const BREAKPOINT: u16 = 1200;
const GUTTER: u16 = 24;
/// The gap between panels, and between summary cards.
pub(super) const GAP: u16 = 20;
const GREETING_HEIGHT: u16 = 66;
/// The summary card row's height.
pub(super) const CARD_ROW_HEIGHT: u16 = 98;
/// The page row the first panel takes; the greeting and the card row come
/// before it.
const FIRST_PANEL_ROW: u16 = 2;
pub(super) const PAGE: &str = "page";

/// One of the four dashboard panels, each a one-cell container its content
/// is placed in.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Panel {
    Chart,
    ServiceHealth,
    RecentActivity,
    Deployments,
}

impl Panel {
    const ALL: [Self; 4] = [
        Self::Chart,
        Self::ServiceHealth,
        Self::RecentActivity,
        Self::Deployments,
    ];

    const fn container(self) -> &'static str {
        match self {
            Self::Chart => "chart_panel",
            Self::ServiceHealth => "health_panel",
            Self::RecentActivity => "activity_panel",
            Self::Deployments => "deployments_panel",
        }
    }

    /// The page cell the panel takes from the breakpoint up: charts and
    /// lists on the left, their companions on the right.
    const fn cell(self) -> MosaicLayoutCell {
        match self {
            Self::Chart => MosaicLayoutCell::at(0, FIRST_PANEL_ROW),
            Self::ServiceHealth => MosaicLayoutCell::at(1, FIRST_PANEL_ROW),
            Self::RecentActivity => MosaicLayoutCell::at(0, FIRST_PANEL_ROW + 1),
            Self::Deployments => MosaicLayoutCell::at(1, FIRST_PANEL_ROW + 1),
        }
    }

    /// Where the concept draws the panel, which its content's insets are
    /// read against.
    const fn concept(self) -> [u16; 4] {
        match self {
            Self::Chart => [266, 269, 812, 348],
            Self::ServiceHealth => [1095, 269, 411, 348],
            Self::RecentActivity => [266, 636, 812, 346],
            Self::Deployments => [1095, 636, 411, 346],
        }
    }

    /// The panel's own box, which its content is placed in.
    pub(super) const fn frame(self) -> Frame {
        Frame::cell(self.container(), MosaicLayoutCell::at(0, 0), self.concept())
    }
}

/// The greeting row, across both panel columns.
pub(super) fn greeting() -> Frame {
    Frame::cell(PAGE, across(0), [266, 81, 1240, 66])
}

/// Where the summary card row stands: filling its page row.
pub(super) fn card_row() -> DashboardPlacement {
    DashboardPlacement::Cell(DashboardLayoutCell {
        container: PAGE,
        cell: across(1),
        region: fill(),
    })
}

/// A page row spanning both panel columns.
fn across(row: u16) -> MosaicLayoutCell {
    MosaicLayoutCell::spanning(0, row, 2, 1).expect("a page row spans both columns")
}

fn fill() -> ComponentViewportRegion {
    ComponentViewportRegion::new(
        ComponentViewportAxisPlacement::stretch_between(0, 0),
        ComponentViewportAxisPlacement::stretch_between(0, 0),
    )
}

/// The stage beside the sidebar and below the masthead: where the page
/// stands, and the region it scrolls its content through.
pub(super) fn stage() -> ComponentViewportRegion {
    ComponentViewportRegion::new(
        ComponentViewportAxisPlacement::stretch_between(PLATFORM_PULSE_SIDEBAR_WIDTH, 0),
        ComponentViewportAxisPlacement::stretch_between(PLATFORM_PULSE_MASTHEAD_HEIGHT, 0),
    )
}

/// Where a member of the page stands below the breakpoint: the greeting and
/// the card row keep their rows in the one column, and the panels follow in
/// source order, one per row.
fn stacked_cell(cell: MosaicLayoutCell) -> MosaicLayoutCell {
    match Panel::ALL.iter().position(|panel| panel.cell() == cell) {
        Some(order) => MosaicLayoutCell::at(0, FIRST_PANEL_ROW + order as u16),
        None => MosaicLayoutCell::at(0, cell.row()),
    }
}

/// The page and the four panel frames.
pub(super) fn containers() -> Vec<ContainerTracks> {
    let track = |track: Result<MosaicTrack, _>| track.expect("the page declares valid tracks");
    let wide = MosaicLayoutContract::grid(
        [
            track(MosaicTrack::flex(2, 480)),
            track(MosaicTrack::flex(1, 320)),
        ],
        [
            track(MosaicTrack::fixed(GREETING_HEIGHT)),
            track(MosaicTrack::fixed(CARD_ROW_HEIGHT)),
            track(MosaicTrack::flex(1, 348)),
            track(MosaicTrack::flex(1, 346)),
        ],
    )
    .expect("the page declares its tracks");
    let stacked = MosaicLayoutContract::grid(
        [track(MosaicTrack::flex(1, 480))],
        [
            track(MosaicTrack::fixed(GREETING_HEIGHT)),
            track(MosaicTrack::fixed(2 * CARD_ROW_HEIGHT + GAP)),
            track(MosaicTrack::flex(1, 348)),
            track(MosaicTrack::flex(1, 348)),
            track(MosaicTrack::flex(1, 346)),
            track(MosaicTrack::flex(1, 346)),
        ],
    )
    .expect("the stacked page declares its tracks");
    let page = ContainerTracks {
        id: PAGE,
        placement: DashboardPlacement::Viewport(stage()),
        tracks: wide.with_gaps(GAP, GAP).with_padding(GUTTER, GUTTER),
        stacked: Some(StackedTracks {
            tracks: stacked.with_gaps(GAP, GAP).with_padding(GUTTER, GUTTER),
            cell: stacked_cell,
        }),
        scroll_owner: Some(DashboardScrollOwner::Page),
    };
    let panels = Panel::ALL.map(|panel| ContainerTracks {
        id: panel.container(),
        placement: DashboardPlacement::Cell(DashboardLayoutCell {
            container: PAGE,
            cell: panel.cell(),
            region: fill(),
        }),
        tracks: MosaicLayoutContract::frame().expect("a panel frames its content"),
        stacked: None,
        scroll_owner: None,
    });
    std::iter::once(page).chain(panels).collect()
}
