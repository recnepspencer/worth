//! Where dashboard content stands relative to the box that holds it.
//!
//! Every element keeps the rectangle the checked-in 1536 x 1024 concept gives
//! it. A frame is the concept rectangle of the box the element stands in: the
//! viewport, a layout container's cell, or a panel. The element's insets from
//! that frame are read off the concept once, and each axis says which frame
//! edge the element keeps its inset from as the frame resizes.
use worth_ui::facade::declaration::{
    ComponentViewportAxisPlacement, ComponentViewportRegion, MosaicLayoutCell,
};

use super::{DashboardElement, DashboardLayoutCell, DashboardPlacement};

/// Which frame edge an element keeps its concept inset from along one axis.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Edge {
    /// Keeps its start inset and its extent.
    Start,
    /// Keeps both insets; its extent follows the frame.
    Both,
    /// Keeps its end inset and its extent.
    End,
}

/// The concept rectangle of the box an element stands in, and the layout cell
/// that box is, if any.
#[derive(Clone, Copy, Debug)]
pub(super) struct Frame {
    cell: Option<(&'static str, MosaicLayoutCell)>,
    concept: [u16; 4],
}

/// The concept's viewport.
pub(super) const VIEWPORT: Frame = Frame {
    cell: None,
    concept: [0, 0, 1536, 1024],
};

impl Frame {
    /// A cell of `container` whose box the concept draws at `concept`.
    pub(super) const fn cell(
        container: &'static str,
        cell: MosaicLayoutCell,
        concept: [u16; 4],
    ) -> Self {
        Self {
            cell: Some((container, cell)),
            concept,
        }
    }

    /// Places `element` in this frame, keeping its concept insets from the
    /// chosen edges.
    pub(super) fn place(
        self,
        mut element: DashboardElement,
        horizontal: Edge,
        vertical: Edge,
    ) -> DashboardElement {
        element.placement = self.placement(element.rect, horizontal, vertical);
        element
    }

    /// The placement of a box the concept draws at `rect` within this frame.
    pub(super) fn placement(
        self,
        rect: [u16; 4],
        horizontal: Edge,
        vertical: Edge,
    ) -> DashboardPlacement {
        let region = self.region(rect, horizontal, vertical);
        match self.cell {
            Some((container, cell)) => DashboardPlacement::Cell(DashboardLayoutCell {
                container,
                cell,
                region,
            }),
            None => DashboardPlacement::Viewport(region),
        }
    }

    /// The region a box the concept draws at `rect` takes within this frame.
    pub(super) fn region(
        self,
        rect: [u16; 4],
        horizontal: Edge,
        vertical: Edge,
    ) -> ComponentViewportRegion {
        let [x, y, width, height] = self.concept;
        ComponentViewportRegion::new(
            axis(x, width, rect[0], rect[2], horizontal),
            axis(y, height, rect[1], rect[3], vertical),
        )
    }
}

/// One axis of a box drawn from `start` over `extent`, inside a frame drawn
/// from `frame_start` over `frame_extent`.
fn axis(
    frame_start: u16,
    frame_extent: u16,
    start: u16,
    extent: u16,
    edge: Edge,
) -> ComponentViewportAxisPlacement {
    let before = start
        .checked_sub(frame_start)
        .expect("a framed box starts inside its frame");
    let after = || {
        (frame_start + frame_extent)
            .checked_sub(start + extent)
            .expect("a box kept from its frame's end ends inside the frame")
    };
    match edge {
        Edge::Start => ComponentViewportAxisPlacement::fixed_from_start(before, extent),
        Edge::Both => Some(ComponentViewportAxisPlacement::stretch_between(
            before,
            after(),
        )),
        Edge::End => ComponentViewportAxisPlacement::fixed_from_end(after(), extent),
    }
    .expect("a framed box has extent")
}
