//! Presented hit geometry by truth status.
//!
//! A hit row stands at the geometry its frame committed until a Motion sample
//! an admitted witness displayed moves it. Rows never hold a merely accepted
//! sample: the sample crosses to displayed against the retained witness of the
//! row's binding before it moves the row, so every reader of a row reads
//! either committed or displayed truth and names which.

use crate::mounting::presentation::{UiDisplayedRect, UiPublishedRect, UiScrollPoseShift};
use worth_ui_host_contract::{UiMountedCanonicalBox, UiMountedCoordinateSpace};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiPresentedHitRect {
    /// The row stands where its frame committed it.
    Published(UiPublishedRect),
    /// A displayed Motion sample moved the row here, in the layout it was
    /// drawn in. `layout_shift` is how far committed Scroll poses have moved
    /// that layout since; no witness proved the moved rect, so the shift is
    /// applied only at the platform-event and index edges.
    Displayed {
        rect: UiDisplayedRect,
        layout_shift: UiScrollPoseShift,
    },
}

impl UiPresentedHitRect {
    pub(crate) fn coordinate_space(self) -> UiMountedCoordinateSpace {
        match self {
            Self::Published(rect) => rect.coordinate_space(),
            Self::Displayed { rect, .. } => rect.coordinate_space(),
        }
    }

    /// A row a displayed sample moved, before any later pose moves it.
    pub(in crate::mounting) const fn displayed(rect: UiDisplayedRect) -> Self {
        Self::Displayed {
            rect,
            layout_shift: UiScrollPoseShift::none(),
        }
    }

    /// The rect an owner commits from this row: committed geometry as it
    /// stands, and displayed geometry through the named adoption, then moved
    /// as published truth by the poses committed since.
    pub(crate) fn adopted(self) -> UiPublishedRect {
        match self {
            Self::Published(rect) => rect,
            Self::Displayed { rect, layout_shift } => {
                UiPublishedRect::adopting(rect).following_pose(layout_shift)
            }
        }
    }

    /// Whether both rects put the row in the same place, whatever proved it.
    pub(in crate::mounting) fn occupies_same_rect(self, other: Self) -> bool {
        self.index_box() == other.index_box()
    }

    /// Whether the point a platform event reports lands in this rect: the
    /// platform-event edge, where a raw point meets typed geometry.
    pub(crate) fn admits_platform_point(
        self,
        point: crate::mounting::presentation::UiPlatformPoint,
    ) -> bool {
        match self {
            Self::Published(rect) => rect.admits_platform_point(point),
            Self::Displayed { rect, layout_shift } => {
                rect.admits_platform_point_after(layout_shift, point)
            }
        }
    }

    /// The row in the layout a committed Scroll pose moved. Committed
    /// geometry follows the pose; displayed geometry stays as its witness
    /// proved it and records the shift.
    pub(in crate::mounting) fn following_pose(self, shift: UiScrollPoseShift) -> Self {
        match self {
            Self::Published(rect) => Self::Published(rect.following_pose(shift)),
            Self::Displayed { rect, layout_shift } => Self::Displayed {
                rect,
                layout_shift: layout_shift.then(shift),
            },
        }
    }

    /// The box the spatial index files the row under. The index is an
    /// acceleration edge: it answers which rows a point may land in, and every
    /// answer is read back as the row's own typed geometry.
    pub(in crate::mounting) fn index_box(self) -> UiMountedCanonicalBox {
        match self {
            Self::Published(rect) => rect.canonical_box(),
            Self::Displayed { rect, layout_shift } => rect.index_box_after(layout_shift),
        }
    }

    /// The box a test aims a synthetic platform event at: the platform-event edge.
    #[cfg(test)]
    pub(crate) fn platform_box(self) -> UiMountedCanonicalBox {
        self.index_box()
    }
}
