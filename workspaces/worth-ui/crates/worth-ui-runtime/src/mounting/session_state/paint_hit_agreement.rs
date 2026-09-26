//! Paint and hit testing agree on where each instance stands.
//!
//! Interaction moves a hit row by the on-screen Motion sample of its
//! instance or its Portal, then by the Scroll poses committed since its frame
//! published it. The host shows the instance's text and surface through the
//! accepted change composed from every Motion moving them. The two lanes are
//! computed apart, so this compares them: the change showing a painted
//! command, applied to its row's committed rect, lands on the rect
//! interaction reads, interaction reads no part of the row outside the clip
//! the host shows the command through, and a row the host shows inside that
//! clip is one a pointer can reach.
//!
//! Scroll paints each move on the device grid while hit testing stands exactly
//! at the offset, so the lanes may place an edge up to one device pixel apart:
//! half a pixel of rounding where the move starts and half where it lands.

use worth_ui_host_contract::{
    UiMountedCanonicalBox, UiMountedInstanceIdentity, UiMountedPaintCommandIdentity,
    UiMountedPresentationTransform, UiSemanticSurfaceIdentity,
};

use super::WorthUiMountedSessionState;
use crate::mounting::UiPresentedFrameBasisDenial;

/// How far two lanes may place one edge apart beyond the device grid: the
/// rounding of composing the same maps in another order.
const ROUNDING: f32 = 1.0 / 256.0;

/// Where paint and hit testing part on a displayed presentation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum UiPaintHitParting {
    /// The host shows `command` through `transform`, moving the row committed
    /// at `committed` to `shown`, while interaction reads it at `hit`. `shown`
    /// is `None` when the transform does not map the space the row is drawn in.
    Moved {
        command: UiMountedPaintCommandIdentity,
        transform: Option<UiMountedPresentationTransform>,
        committed: UiMountedCanonicalBox,
        shown: Option<UiMountedCanonicalBox>,
        hit: UiMountedCanonicalBox,
    },
    /// Interaction reads `hit` where the host clips `command`, shown through
    /// `transform`, to `clip`.
    Unclipped {
        command: UiMountedPaintCommandIdentity,
        transform: Option<UiMountedPresentationTransform>,
        clip: UiMountedCanonicalBox,
        hit: UiMountedCanonicalBox,
    },
    /// The host shows `command`, through `transform`, over `visible`, while
    /// a pointer reaches no part of its row.
    Unreached {
        command: UiMountedPaintCommandIdentity,
        transform: Option<UiMountedPresentationTransform>,
        visible: UiMountedCanonicalBox,
    },
    /// The presentation state holds a frame other than the displayed one.
    Unpresented(UiMountedInstanceIdentity),
    /// The surface's device scale is not one Scroll paints on, so no slack
    /// between the lanes is known.
    Unscaled,
    /// A row a pointer cannot reach is shown on a surface with no current
    /// layout viewport, so whether the host shows any of it is unknown.
    Unbounded,
    /// Interaction cannot read the frame the host displays.
    Unread(UiPresentedFrameBasisDenial),
}

impl WorthUiMountedSessionState {
    /// Every place paint and hit testing part on what `surface` displays. A
    /// presentation whose published entrance awaits its Motion commit is not
    /// compared: interaction reads the entrance before paint accepts it.
    pub(crate) fn parted_paint_and_hit(
        &self,
        surface: UiSemanticSurfaceIdentity,
    ) -> Vec<UiPaintHitParting> {
        let Some(displayed) = self.current_presentation_for_surface(surface) else {
            return Vec::new();
        };
        let presentation = displayed.basis();
        if self
            .presentation
            .awaits_entrance_commit(presentation.binding())
        {
            return Vec::new();
        }
        let Some(scale) = self.scroll_chrome_device_scale(surface) else {
            return vec![UiPaintHitParting::Unscaled];
        };
        let viewport = self
            .current_surface_viewport(surface)
            .map(|(_, viewport)| viewport);
        let basis = match self.interaction_hit_test_basis(presentation) {
            Ok(basis) => basis,
            Err(denial) => return vec![UiPaintHitParting::Unread(denial)],
        };
        #[expect(
            clippy::cast_possible_truncation,
            reason = "a device pixel in points is a small positive length"
        )]
        let slack = Slack(ROUNDING + scale.pixel_in_points() as f32);
        let mut parted = Vec::new();
        for row in basis.rows() {
            let hit = row.bounds().index_box();
            let Some(shown) = self.presentation.shown_own_paint(
                presentation,
                row.mounted_instance(),
                row.mounted().bounds(),
            ) else {
                parted.push(UiPaintHitParting::Unpresented(row.mounted_instance()));
                continue;
            };
            let reachable = row.reachable_box();
            for paint in shown {
                if !paint
                    .carried
                    .is_some_and(|shown| slack.same_place(shown, hit))
                {
                    parted.push(UiPaintHitParting::Moved {
                        command: paint.command,
                        transform: paint.transform,
                        committed: row.mounted().bounds(),
                        shown: paint.carried,
                        hit,
                    });
                }
                match (paint.clip, reachable) {
                    (Some(clip), Some(reachable)) if !slack.within(reachable, clip) => {
                        parted.push(UiPaintHitParting::Unclipped {
                            command: paint.command,
                            transform: paint.transform,
                            clip,
                            hit: reachable,
                        });
                    }
                    (_, None) if paint.carried.is_some() && viewport.is_none() => {
                        parted.push(UiPaintHitParting::Unbounded);
                    }
                    (clip, None) => {
                        // The host shows nothing of the surface outside its
                        // viewport, whatever else clips the command.
                        let visible = paint
                            .carried
                            .zip(viewport)
                            .and_then(|(shown, viewport)| shown.intersection(viewport))
                            .and_then(|shown| match clip {
                                Some(clip) => shown.intersection(clip),
                                None => Some(shown),
                            });
                        if let Some(visible) = visible.filter(|visible| slack.spanned_by(*visible))
                        {
                            parted.push(UiPaintHitParting::Unreached {
                                command: paint.command,
                                transform: paint.transform,
                                visible,
                            });
                        }
                    }
                    (None, Some(_)) | (Some(_), Some(_)) => {}
                }
            }
        }
        parted
    }
}

/// Where a rect's four edges stand in its space.
#[derive(Clone, Copy)]
struct Edges {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

impl Edges {
    fn of(rect: UiMountedCanonicalBox) -> Self {
        Self {
            left: rect.x(),
            top: rect.y(),
            right: rect.x() + rect.width(),
            bottom: rect.y() + rect.height(),
        }
    }
}

/// How far apart the lanes may place one edge on a surface.
#[derive(Clone, Copy)]
struct Slack(f32);

impl Slack {
    fn same_place(self, a: UiMountedCanonicalBox, b: UiMountedCanonicalBox) -> bool {
        let (a_edges, b_edges) = (Edges::of(a), Edges::of(b));
        let near = |a: f32, b: f32| (a - b).abs() <= self.0;
        a.coordinate_space() == b.coordinate_space()
            && near(a_edges.left, b_edges.left)
            && near(a_edges.top, b_edges.top)
            && near(a_edges.right, b_edges.right)
            && near(a_edges.bottom, b_edges.bottom)
    }

    /// Whether `rect` is wider and taller than the lanes may part, so a row
    /// shown over it is one a pointer reaches.
    fn spanned_by(self, rect: UiMountedCanonicalBox) -> bool {
        rect.width() > self.0 && rect.height() > self.0
    }

    fn within(self, inner: UiMountedCanonicalBox, outer: UiMountedCanonicalBox) -> bool {
        let (inner_edges, outer_edges) = (Edges::of(inner), Edges::of(outer));
        inner.coordinate_space() == outer.coordinate_space()
            && inner_edges.left >= outer_edges.left - self.0
            && inner_edges.top >= outer_edges.top - self.0
            && inner_edges.right <= outer_edges.right + self.0
            && inner_edges.bottom <= outer_edges.bottom + self.0
    }
}
