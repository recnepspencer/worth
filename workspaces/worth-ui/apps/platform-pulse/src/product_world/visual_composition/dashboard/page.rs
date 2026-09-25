//! The dashboard page: a fixed sidebar, a masthead that fills the width beside
//! it, and the main content laid out in flexible tracks.
//!
//! The main content keeps a 24-point gutter from the sidebar, the masthead,
//! and the viewport edges. It stacks the greeting, the summary card row, and
//! two rows of panels, 20 points apart. The panels sit in two columns that
//! share the remaining width 2:1, and never narrower than 480 and 320 points.
use worth_ui::facade::declaration::{
    ComponentViewportAxisPlacement, ComponentViewportRegion, MosaicLayoutCell,
    MosaicLayoutContract, MosaicTrack,
};

use super::frame::Frame;
use super::{ContainerTracks, DashboardLayoutCell, DashboardPlacement};

/// The sidebar's width, which every screen width keeps.
pub const PLATFORM_PULSE_SIDEBAR_WIDTH: u16 = 236;
/// The masthead's height, down to and including its bottom rule.
pub const PLATFORM_PULSE_MASTHEAD_HEIGHT: u16 = 57;
const GUTTER: u16 = 24;
/// The gap between panels, and between summary cards.
pub(super) const GAP: u16 = 20;
const GREETING_HEIGHT: u16 = 66;
/// The summary card row's height.
pub(super) const CARD_ROW_HEIGHT: u16 = 98;
const PAGE: &str = "page";

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

    /// The page cell the panel takes: charts and lists on the left, their
    /// companions on the right.
    const fn cell(self) -> MosaicLayoutCell {
        match self {
            Self::Chart => MosaicLayoutCell::at(0, 2),
            Self::ServiceHealth => MosaicLayoutCell::at(1, 2),
            Self::RecentActivity => MosaicLayoutCell::at(0, 3),
            Self::Deployments => MosaicLayoutCell::at(1, 3),
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

/// The page and the four panel frames.
pub(super) fn containers() -> Vec<ContainerTracks> {
    let track = |track: Result<MosaicTrack, _>| track.expect("the page declares valid tracks");
    let page = ContainerTracks {
        id: PAGE,
        placement: DashboardPlacement::Viewport(ComponentViewportRegion::new(
            ComponentViewportAxisPlacement::stretch_between(
                PLATFORM_PULSE_SIDEBAR_WIDTH + GUTTER,
                GUTTER,
            ),
            ComponentViewportAxisPlacement::stretch_between(
                PLATFORM_PULSE_MASTHEAD_HEIGHT + GUTTER,
                GUTTER,
            ),
        )),
        tracks: MosaicLayoutContract::grid(
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
        .expect("the page declares its tracks")
        .with_gaps(GAP, GAP),
        scroll_panel: None,
    };
    let panels = Panel::ALL.map(|panel| ContainerTracks {
        id: panel.container(),
        placement: DashboardPlacement::Cell(DashboardLayoutCell {
            container: PAGE,
            cell: panel.cell(),
            region: fill(),
        }),
        tracks: MosaicLayoutContract::frame().expect("a panel frames its content"),
        scroll_panel: None,
    });
    std::iter::once(page).chain(panels).collect()
}
