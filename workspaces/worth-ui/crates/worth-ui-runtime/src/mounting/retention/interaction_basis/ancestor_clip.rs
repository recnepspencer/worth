//! The Scroll and Mosaic clips a hit row sits inside.
//!
//! Paint clips an occurrence by every ancestor Scroll and Mosaic region, so
//! content a region has scrolled out of view shows nothing there. A hit row
//! reaches only where its ancestors let paint show; otherwise a pointer over
//! a region's frame would land on content scrolled beyond it.
//!
//! The row's own clip is its allocation and travels with it. Ancestor clips do
//! not: a Scroll pose moves the content and leaves the region's viewport where
//! it is. So a row holds its ancestors apart, relative to where its frame
//! committed the row, and every pose hands the row the ancestors it leaves.

use worth_ui_host_contract::{UiAppearanceClip, UiMountedCanonicalBox};

use super::UiPresentedHitRect;
use crate::mounting::presentation::{
    UiDisplayedRect, UiDisplayedSurfaceBasis, UiPlatformPoint, UiPublishedMap, UiPublishedRect,
    UiPublishedToAcceptedMap, UiScrollPoseShift,
};
use crate::mounting::projection::UiMountedAppearanceClip;

/// The coverage a row's ancestors leave it, relative to the row's committed
/// origin.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiHitAncestorClip {
    /// No ancestor clips the row.
    Unclipped,
    /// The ancestors share no coverage, so nothing of the row shows.
    Suppressed,
    /// The ancestors' shared coverage, offset from the row's committed origin.
    Within(UiHitAncestorOffset),
}

/// A rectangle offset from a row's committed origin, in points, every
/// component finite and the extent non-negative. It names no space of its
/// own; it stands in whatever space the row it is read against stands in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct UiHitAncestorOffset {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

// Every component is finite, so none is NaN.
impl Eq for UiHitAncestorOffset {}

impl UiHitAncestorClip {
    /// `clip` read against the row committed at `row`. A clip paint cannot
    /// resolve suppresses nothing, as a Scroll pose reads it: paint lowers no
    /// frame over it, and a Portal child's coverage is its row's own clip.
    pub(crate) fn relative_to(clip: UiMountedAppearanceClip, row: UiMountedCanonicalBox) -> Self {
        match clip {
            UiMountedAppearanceClip::Unclipped | UiMountedAppearanceClip::Unresolved(_) => {
                Self::Unclipped
            }
            UiMountedAppearanceClip::Suppressed => Self::Suppressed,
            UiMountedAppearanceClip::Ancestor(clip) => {
                relative_rect(clip, row).map_or(Self::Suppressed, Self::Within)
            }
        }
    }
}

/// Logical subpixels back to points, offset from the row's origin. `None`
/// when the offset leaves finite geometry.
fn relative_rect(
    clip: UiAppearanceClip,
    row: UiMountedCanonicalBox,
) -> Option<UiHitAncestorOffset> {
    let unit = f64::from(worth_ui_host_contract::UI_APPEARANCE_LOGICAL_SUBPIXELS_PER_POINT);
    #[expect(
        clippy::cast_possible_truncation,
        reason = "clip subpixels come from finite f32 points, so points fit f32"
    )]
    let points = |subpixels: f64, origin: f32| (subpixels / unit - f64::from(origin)) as f32;
    let offset = UiHitAncestorOffset {
        x: points(f64::from(clip.x()), row.x()),
        y: points(f64::from(clip.y()), row.y()),
        width: points(f64::from(clip.width()), 0.0),
        height: points(f64::from(clip.height()), 0.0),
    };
    [offset.x, offset.y, offset.width, offset.height]
        .iter()
        .all(|component| component.is_finite())
        .then_some(offset)
}

/// How a committed Scroll pose moves one hit row: how far it carries the row,
/// and the ancestor clips it leaves over the row.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiHitScrollMove {
    shift: UiScrollPoseShift,
    ancestor: Option<UiHitAncestorClip>,
}

impl UiHitScrollMove {
    /// No committed pose has moved the row.
    pub(crate) const fn none() -> Self {
        Self {
            shift: UiScrollPoseShift::none(),
            ancestor: None,
        }
    }

    /// A pose that carries the row by `shift` and leaves `ancestor` over it,
    /// relative to where the pose leaves the row.
    pub(crate) const fn new(shift: UiScrollPoseShift, ancestor: UiHitAncestorClip) -> Self {
        Self {
            shift,
            ancestor: Some(ancestor),
        }
    }

    /// This move followed by `later`: the shifts add, and the later pose
    /// names the ancestors that stand now.
    pub(crate) fn then(self, later: Self) -> Self {
        Self {
            shift: self.shift.then(later.shift),
            ancestor: later.ancestor.or(self.ancestor),
        }
    }

    pub(crate) const fn shift(self) -> UiScrollPoseShift {
        self.shift
    }

    pub(crate) const fn ancestor(self) -> Option<UiHitAncestorClip> {
        self.ancestor
    }
}

/// What places a row's ancestors where the host shows the row. A row's own
/// Motion moves it inside its ancestors, which stay committed; an entrance or
/// a Portal moves the ancestors with the row.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum UiHitAncestorPlacement {
    Committed,
    Entrance(UiPublishedMap),
    Portal {
        map: UiPublishedToAcceptedMap,
        displayed: UiDisplayedSurfaceBasis,
    },
}

impl UiHitAncestorPlacement {
    /// Where `ancestor`, relative to a row committed at `row`, reaches once
    /// this placement and the poses since, `shift`, have moved it.
    pub(super) fn reach(
        self,
        ancestor: UiHitAncestorClip,
        row: UiMountedCanonicalBox,
        shift: UiScrollPoseShift,
    ) -> UiHitAncestorReach {
        let UiHitAncestorOffset {
            x,
            y,
            width,
            height,
        } = match ancestor {
            UiHitAncestorClip::Unclipped => return UiHitAncestorReach::Anywhere,
            UiHitAncestorClip::Suppressed => return UiHitAncestorReach::Nowhere,
            UiHitAncestorClip::Within(offset) => offset,
        };
        let Ok(committed) = UiPublishedRect::from_committed_components(
            [row.x() + x, row.y() + y, width, height],
            row.coordinate_space(),
        ) else {
            return UiHitAncestorReach::Nowhere;
        };
        let placed = match self {
            Self::Committed => Some(UiPresentedHitRect::Published(committed)),
            Self::Entrance(map) => map.apply(committed).map(UiPresentedHitRect::Published),
            // The row's own Portal sample was displayed at `displayed`, and
            // the same map places the ancestors it carries. The crossing to
            // `displayed` reads only the surface and binding `map` accepted
            // on, and the row's own bounds already crossed it through `map`
            // when this placement was made, so it cannot be denied here.
            Self::Portal { map, displayed } => map.apply(committed).map(|accepted| {
                UiPresentedHitRect::displayed(
                    UiDisplayedRect::displayed(accepted, displayed)
                        .expect("a displayed Portal sample displays what it carries"),
                )
            }),
        };
        // A placement beyond finite geometry reaches nowhere.
        placed.map_or(UiHitAncestorReach::Nowhere, |rect| {
            UiHitAncestorReach::Within(rect.following_pose(shift))
        })
    }
}

/// Where a row's ancestors let it be reached.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UiHitAncestorReach {
    Anywhere,
    Nowhere,
    Within(UiPresentedHitRect),
}

impl UiHitAncestorReach {
    /// Whether the point a platform event reports lands where the row's
    /// ancestors let it be reached.
    pub(crate) fn admits_platform_point(self, point: UiPlatformPoint) -> bool {
        match self {
            Self::Anywhere => true,
            Self::Nowhere => false,
            Self::Within(rect) => rect.admits_platform_point(point),
        }
    }

    /// Whether both let the row be reached in the same place, whatever
    /// proved it.
    pub(in crate::mounting) fn occupies_same_reach(self, other: Self) -> bool {
        match (self, other) {
            (Self::Within(left), Self::Within(right)) => left.occupies_same_rect(right),
            (left, right) => left == right,
        }
    }

    /// The box the spatial index narrows a row to: `None` when the ancestors
    /// do not narrow it.
    pub(in crate::mounting) fn index_box(self) -> Option<UiMountedCanonicalBox> {
        match self {
            Self::Within(rect) => Some(rect.index_box()),
            Self::Anywhere | Self::Nowhere => None,
        }
    }
}
