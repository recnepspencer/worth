//! Resolves what the dashboard declares, independently of the runtime.
//!
//! Each container's tracks are allocated here the way the layout spec states
//! them: inside the container's padding, fixed tracks keep their extent,
//! flexible tracks share what is left by weight, and a share below its
//! minimum is held there. A container too small for its tracks' minimums
//! grows to hold them. Tests compare these boxes against closed-form
//! expectations.
use std::collections::BTreeMap;

use worth_ui::facade::declaration::{
    ComponentId, ComponentViewportAxisPlacement as Axis, ComponentViewportRegion, MosaicTrack,
};

use super::{
    dashboard_containers, dashboard_elements, DashboardContainer, DashboardElement,
    DashboardPlacement, DashboardScrollOwner,
};

pub(super) type Rect = [f32; 4];

/// The concept extent, the supported maximum, the two-column floor, a
/// height below the panel rows' minimums, the widest and narrowest stacked
/// extents the spec drags through, and a stacked extent tall enough for its
/// panels to share the height.
pub(super) const EXTENTS: [(f32, f32); 7] = [
    (1536.0, 1024.0),
    (1920.0, 1200.0),
    (1200.0, 800.0),
    (1280.0, 720.0),
    (1120.0, 800.0),
    (800.0, 600.0),
    (1100.0, 2400.0),
];
pub(super) const PANELS: [&str; 4] = [
    "chart_panel",
    "health_panel",
    "activity_panel",
    "deployments_panel",
];

/// Where the declarations put every component at one viewport extent.
pub(super) struct Declared {
    viewport: Rect,
    containers: BTreeMap<&'static str, DashboardContainer>,
    elements: BTreeMap<&'static str, DashboardElement>,
}

impl Declared {
    pub(super) fn at(width: f32, height: f32) -> Self {
        Self {
            viewport: [0.0, 0.0, width, height],
            containers: dashboard_containers()
                .into_iter()
                .map(|container| (container.id, container))
                .collect(),
            elements: dashboard_elements()
                .into_iter()
                .map(|element| (element.id, element))
                .collect(),
        }
    }

    pub(super) fn element(&self, id: &str) -> Rect {
        let element = self.elements[id];
        let placed = self
            .resolve(&element.component(), element.placement, Some(element.rect))
            .0;
        match element.portal_owner {
            Some(owner) => {
                let owner = self.element(owner);
                [
                    owner[0] + placed[0],
                    owner[1] + placed[1],
                    placed[2],
                    placed[3],
                ]
            }
            None => placed,
        }
    }

    /// A container's box: where its placement puts it, grown on each axis
    /// to hold its tracks' minimums, gaps, and padding.
    pub(super) fn container(&self, id: &str) -> Rect {
        let container = &self.containers[id];
        let [x, y, width, height] = self
            .resolve(&container.component(), container.placement, None)
            .0;
        let layout = container.layout.select(self.viewport[2]);
        let across = minimum(
            layout.column_tracks(),
            layout.column_gap_logical_points(),
            layout.inline_padding_logical_points(),
        );
        let down = minimum(
            layout.row_tracks(),
            layout.row_gap_logical_points(),
            layout.block_padding_logical_points(),
        );
        [x, y, width.max(across), height.max(down)]
    }

    /// A scroll owner's region, resolved within the box its content was
    /// placed in.
    pub(super) fn region(&self, owner: DashboardScrollOwner) -> Rect {
        let container = &self.containers[owner.container()];
        let reference = self
            .resolve(&container.component(), container.placement, None)
            .1;
        region_box(owner.region_placement(), reference)
    }

    /// The panel a placement stands in, through any containers between.
    pub(super) fn panel_of(&self, placement: DashboardPlacement) -> Option<&'static str> {
        let cell = placement.layout_cell()?;
        if PANELS.contains(&cell.container) {
            return Some(cell.container);
        }
        self.panel_of(self.containers[cell.container].placement)
    }

    /// The box `component`'s placement resolves to, and the box it resolved
    /// within. A layout member takes the cell its container's layout assigns
    /// it at this viewport width.
    fn resolve(
        &self,
        component: &ComponentId,
        placement: DashboardPlacement,
        authored: Option<[u16; 4]>,
    ) -> (Rect, Rect) {
        match placement {
            DashboardPlacement::Authored => (
                authored.expect("an element is authored").map(f32::from),
                self.viewport,
            ),
            DashboardPlacement::Viewport(region) => {
                (region_box(region, self.viewport), self.viewport)
            }
            DashboardPlacement::Cell(cell) => {
                let [x, y, width, height] = self.container(cell.container);
                let layout = self.containers[cell.container]
                    .layout
                    .select(self.viewport[2]);
                let inline = f32::from(layout.inline_padding_logical_points());
                let block = f32::from(layout.block_padding_logical_points());
                let columns = allocate(
                    layout.column_tracks(),
                    layout.column_gap_logical_points(),
                    width - 2.0 * inline,
                );
                let rows = allocate(
                    layout.row_tracks(),
                    layout.row_gap_logical_points(),
                    height - 2.0 * block,
                );
                let member = layout
                    .member_cell(component)
                    .expect("a member sits in its container's selected layout");
                let (left, across) = span(&columns, member.column(), member.column_span());
                let (top, down) = span(&rows, member.row(), member.row_span());
                let reference = [x + inline + left, y + block + top, across, down];
                (region_box(cell.region, reference), reference)
            }
        }
    }
}

/// Every track at its minimum, the gaps between, and the padding on both
/// sides.
fn minimum(tracks: &[MosaicTrack], gap: u16, padding: u16) -> f32 {
    let extents = tracks
        .iter()
        .map(|track| f32::from(track.base_logical_points()))
        .sum::<f32>();
    extents + f32::from(gap) * tracks.len().saturating_sub(1) as f32 + 2.0 * f32::from(padding)
}

/// Fixed tracks keep their extent; flexible tracks share what is left by
/// weight, and a share below its minimum is held there while the others
/// share the rest.
fn allocate(tracks: &[MosaicTrack], gap: u16, available: f32) -> Vec<(f32, f32)> {
    let gaps = f32::from(gap) * tracks.len().saturating_sub(1) as f32;
    let mut held = tracks
        .iter()
        .map(|track| {
            track
                .weight()
                .is_none()
                .then(|| f32::from(track.base_logical_points()))
        })
        .collect::<Vec<_>>();
    loop {
        let used = gaps + held.iter().flatten().sum::<f32>();
        let open = tracks.iter().zip(&held).filter(|(_, held)| held.is_none());
        let weights = open
            .clone()
            .filter_map(|(track, _)| track.weight())
            .map(f32::from)
            .sum::<f32>();
        let per_weight = (available - used) / weights;
        let mut clamped = false;
        for (index, track) in tracks.iter().enumerate() {
            let (None, Some(weight)) = (held[index], track.weight()) else {
                continue;
            };
            if per_weight * f32::from(weight) < f32::from(track.base_logical_points()) {
                held[index] = Some(f32::from(track.base_logical_points()));
                clamped = true;
            }
        }
        if !clamped {
            let extents = tracks.iter().zip(&held).map(|(track, held)| {
                held.unwrap_or_else(|| per_weight * f32::from(track.weight().unwrap_or(0)))
            });
            let mut start = 0.0;
            return extents
                .map(|extent| {
                    let track = (start, extent);
                    start += extent + f32::from(gap);
                    track
                })
                .collect();
        }
    }
}

fn span(tracks: &[(f32, f32)], first: u16, count: u16) -> (f32, f32) {
    let (start, _) = tracks[usize::from(first)];
    let (last_start, last_extent) = tracks[usize::from(first + count - 1)];
    (start, last_start + last_extent - start)
}

fn region_box(region: ComponentViewportRegion, [x, y, width, height]: Rect) -> Rect {
    let (left, across) = axis(region.horizontal(), width);
    let (top, down) = axis(region.vertical(), height);
    [x + left, y + top, across, down]
}

fn axis(axis: Axis, available: f32) -> (f32, f32) {
    match axis {
        Axis::FixedFromStart {
            start_logical_points,
            extent_logical_points,
        } => (
            f32::from(start_logical_points),
            f32::from(extent_logical_points),
        ),
        Axis::StretchBetween {
            start_logical_points,
            end_logical_points,
        } => {
            let start = f32::from(start_logical_points);
            (
                start,
                (available - start - f32::from(end_logical_points)).max(0.0),
            )
        }
        Axis::FixedFromEnd {
            end_logical_points,
            extent_logical_points,
        } => {
            let extent = f32::from(extent_logical_points);
            (
                (available - f32::from(end_logical_points) - extent).max(0.0),
                extent,
            )
        }
    }
}

pub(super) fn assert_near(actual: Rect, expected: Rect, what: &str) {
    let near = actual
        .iter()
        .zip(expected)
        .all(|(actual, expected)| (actual - expected).abs() < 1e-3);
    assert!(near, "{what}: {actual:?} is not {expected:?}");
}
