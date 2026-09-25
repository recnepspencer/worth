use super::displayed::UiDisplayedRect;
use super::published::UiPublishedRect;
use crate::runtime::scroll::UiScrollOffset;
use worth_ui_host_contract::UiMountedCanonicalBox;

/// How far committed Scroll poses have moved one occurrence's layout: a
/// published translation, the difference between committed offsets.
///
/// Published rects follow it and stay published. A displayed rect never does:
/// its witness proved it in the layout the host showed, and nothing proves
/// where it stands after a later pose. A holder keeps the shift beside the
/// displayed rect and applies it only at the platform-event and spatial-index
/// edges, through [`UiDisplayedRect::admits_platform_point_after`] and
/// [`UiDisplayedRect::index_box_after`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiScrollPoseShift([f32; 2]);

// Both components derive from finite committed offsets, so neither is NaN.
impl Eq for UiScrollPoseShift {}

impl UiScrollPoseShift {
    /// No committed pose has moved the layout.
    pub(crate) const fn none() -> Self {
        Self([0.0, 0.0])
    }

    /// How far committing `next` over `previous` moves the content they scroll.
    pub(in crate::mounting) fn between(previous: UiScrollOffset, next: UiScrollOffset) -> Self {
        let unit = worth_ui_host_contract::UI_HOST_SURFACE_POSITION_SUBPIXELS_PER_UNIT as f64;
        let points = |from: i64, to: i64| ((from - to) as f64 / unit) as f32;
        Self([
            points(previous.inline_subpixels(), next.inline_subpixels()),
            points(previous.block_subpixels(), next.block_subpixels()),
        ])
    }

    /// This shift followed by `later`.
    pub(crate) fn then(self, later: Self) -> Self {
        Self([self.0[0] + later.0[0], self.0[1] + later.0[1]])
    }
}

impl UiPublishedRect {
    /// This committed rect in the layout `shift` moved.
    pub(crate) fn following_pose(self, shift: UiScrollPoseShift) -> Self {
        Self::from_rect(self.rect().shifted(shift.0))
    }

    /// A committed mounted box in the layout `shift` moved, for owners that
    /// keep committed layout as host-contract boxes.
    pub(crate) fn box_following_pose(
        bounds: UiMountedCanonicalBox,
        shift: UiScrollPoseShift,
    ) -> UiMountedCanonicalBox {
        Self::from_committed_box(bounds)
            .following_pose(shift)
            .canonical_box()
    }
}

impl UiDisplayedRect {
    /// Whether the point a platform event reports lands where this rect
    /// stands once committed poses have moved the layout by `shift` since the
    /// sample was drawn. The moved rect exists only for this comparison.
    pub(crate) fn admits_platform_point_after(
        self,
        shift: UiScrollPoseShift,
        point: super::UiPlatformPoint,
    ) -> bool {
        self.rect().shifted(shift.0).admits(point)
    }

    /// The box the spatial index files this rect under once committed poses
    /// have moved the layout by `shift`: the acceleration edge, whose answers
    /// are read back through [`Self::admits_platform_point_after`].
    pub(crate) fn index_box_after(self, shift: UiScrollPoseShift) -> UiMountedCanonicalBox {
        self.rect().shifted(shift.0).canonical_box()
    }
}
