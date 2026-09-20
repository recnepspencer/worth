#[path = "content_geometry.rs"]
mod content_geometry;
use super::{UiHeadlessAppearanceFrameTranscript, UiHeadlessAppearanceMechanic};
use worth_ui_host_contract::{
    compose_source_over, UiMountedAppearanceColor, UiMountedSurfaceAppearanceMechanic,
    UiMountedSurfaceBorderSide, UiMountedSurfacePaint, UiOverlayParticipantIdentity,
};

struct SurfaceSampleGeometry {
    bounds: [i64; 4],
    radii: [i64; 4],
    x: i64,
    y: i64,
}

impl UiHeadlessAppearanceFrameTranscript {
    /// Samples one ordinary mounted Surface by its exact occurrence identity.
    pub fn reference_surface_at(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        x: i64,
        y: i64,
    ) -> Option<UiMountedAppearanceColor> {
        let surface = self.mechanics.iter().find_map(|mechanic| match mechanic {
            UiHeadlessAppearanceMechanic::Surface(surface)
                if surface.node_receipt().mounted_instance() == instance =>
            {
                Some(surface)
            }
            _ => None,
        })?;
        let mut layers = Vec::new();
        append_surface(&mut layers, surface, x, y);
        Some(compose_source_over(layers))
    }

    /// Samples the issued overlay stack at a logical subpixel point. Extents,
    /// clips, rounded contours and inward borders participate in this reference;
    /// native pixel-edge antialiasing and glyph coverage have separate proofs.
    /// Ordinary content is outside this overlay-only sample.
    pub fn reference_overlay_at(&self, x: i64, y: i64) -> UiMountedAppearanceColor {
        let mut layers = Vec::new();
        for participant in self.overlay_order.bottom_to_top() {
            let mechanic = self
                .mechanics
                .iter()
                .find(|mechanic| match (participant, mechanic) {
                    (
                        UiOverlayParticipantIdentity::Portal(instance),
                        UiHeadlessAppearanceMechanic::PortalSurface(surface),
                    ) => surface.portal_instance() == *instance,
                    (
                        UiOverlayParticipantIdentity::Backdrop(identity),
                        UiHeadlessAppearanceMechanic::Backdrop(backdrop),
                    ) => backdrop.identity() == identity,
                    _ => false,
                });
            match mechanic {
                Some(UiHeadlessAppearanceMechanic::PortalSurface(portal)) => {
                    append_surface(&mut layers, portal.surface(), x, y);
                }
                Some(UiHeadlessAppearanceMechanic::Backdrop(backdrop)) => {
                    let extent = backdrop.extent();
                    let clip = backdrop.clip();
                    if contains(
                        [
                            i64::from(extent.x()),
                            i64::from(extent.y()),
                            i64::from(extent.width()),
                            i64::from(extent.height()),
                        ],
                        x,
                        y,
                    ) && contains(
                        [
                            i64::from(clip.x()),
                            i64::from(clip.y()),
                            i64::from(clip.width()),
                            i64::from(clip.height()),
                        ],
                        x,
                        y,
                    ) {
                        layers.push((backdrop.background(), backdrop.opacity()));
                    }
                }
                // Translation already validates every Backdrop. Structural
                // Portals may own an order position without surface paint.
                _ => {}
            }
        }
        compose_source_over(layers)
    }
}

fn append_surface(
    layers: &mut Vec<(
        UiMountedAppearanceColor,
        worth_ui_host_contract::UiMountedPresentationOpacity,
    )>,
    surface: &UiMountedSurfaceAppearanceMechanic,
    x: i64,
    y: i64,
) {
    let bounds = surface.bounds();
    let clip = surface.clip();
    let bounds = [
        i64::from(bounds.x()),
        i64::from(bounds.y()),
        i64::from(bounds.width()),
        i64::from(bounds.height()),
    ];
    let radii = surface.radii().corners().map(i64::from);
    let shape_coverage = content_geometry::coverage(surface.geometry(), bounds, [x, y]);
    if !contains(
        [
            i64::from(clip.x()),
            i64::from(clip.y()),
            i64::from(clip.width()),
            i64::from(clip.height()),
        ],
        x,
        y,
    ) || if shape_coverage.is_some() {
        !contains(bounds, x, y)
    } else {
        !inside_contour(bounds, radii, x, y)
    } {
        return;
    }
    let (fill, border) = match surface.paint() {
        UiMountedSurfacePaint::Fill(fill) => (Some(*fill), None),
        UiMountedSurfacePaint::Border {
            color,
            inward_width,
        } => (None, Some((*color, i64::from(inward_width.subpixels())))),
        UiMountedSurfacePaint::FillAndBorder {
            fill,
            border,
            inward_width,
        } => (
            Some(*fill),
            Some((*border, i64::from(inward_width.subpixels()))),
        ),
    };
    if let Some(fill) = fill {
        let opacity = shape_coverage.map_or(surface.opacity(), |coverage| {
            worth_ui_host_contract::UiMountedPresentationOpacity::from_runtime_composition(
                (f64::from(surface.opacity().units()) * coverage).round() as u16,
            )
        });
        let [left, top, width, height] = bounds;
        let fill = fill.sample(
            [
                left as f64,
                top as f64,
                (left + width) as f64,
                (top + height) as f64,
            ],
            [x as f64, y as f64],
        );
        layers.push((fill, opacity));
    }
    if let Some((border, width)) = border {
        if samples_border(
            surface,
            width,
            SurfaceSampleGeometry {
                bounds,
                radii,
                x,
                y,
            },
        ) {
            layers.push((border, surface.opacity()));
        }
    }
}

fn samples_border(
    surface: &UiMountedSurfaceAppearanceMechanic,
    width: i64,
    sample: SurfaceSampleGeometry,
) -> bool {
    let [left, top, width_outer, height_outer] = sample.bounds;
    let inner = [
        left + width,
        top + width,
        width_outer - 2 * width,
        height_outer - 2 * width,
    ];
    let edge_enabled = (sample.y < inner[1]
        && border_side_enabled(surface, UiMountedSurfaceBorderSide::Top, sample.x - left))
        || (sample.x >= inner[0] + inner[2]
            && border_side_enabled(surface, UiMountedSurfaceBorderSide::Right, sample.y - top))
        || (sample.y >= inner[1] + inner[3]
            && border_side_enabled(surface, UiMountedSurfaceBorderSide::Bottom, sample.x - left))
        || (sample.x < inner[0]
            && border_side_enabled(surface, UiMountedSurfaceBorderSide::Left, sample.y - top));
    edge_enabled
        && !inside_contour(
            inner,
            sample.radii.map(|radius| (radius - width).max(0)),
            sample.x,
            sample.y,
        )
}

fn border_side_enabled(
    surface: &UiMountedSurfaceAppearanceMechanic,
    side: UiMountedSurfaceBorderSide,
    offset: i64,
) -> bool {
    let edges = surface.border_edges();
    let enabled = match side {
        UiMountedSurfaceBorderSide::Top => edges.top(),
        UiMountedSurfaceBorderSide::Right => edges.right(),
        UiMountedSurfaceBorderSide::Bottom => edges.bottom(),
        UiMountedSurfaceBorderSide::Left => edges.left(),
    };
    enabled
        && !surface.border_omissions().iter().any(|omission| {
            omission.side() == side
                && i64::from(omission.start()) <= offset
                && offset < i64::from(omission.end())
        })
}

fn contains([left, top, width, height]: [i64; 4], x: i64, y: i64) -> bool {
    x >= left && y >= top && x < left + width && y < top + height
}

fn inside_contour(bounds: [i64; 4], radii: [i64; 4], x: i64, y: i64) -> bool {
    if !contains(bounds, x, y) {
        return false;
    }
    let [left, top, width, height] = bounds;
    let right = left + width;
    let bottom = top + height;
    for (radius, horizontal, vertical) in [
        (radii[0], x - left, y - top),
        (radii[1], right - x, y - top),
        (radii[2], right - x, bottom - y),
        (radii[3], x - left, bottom - y),
    ] {
        if horizontal < radius && vertical < radius {
            let dx = i128::from(radius - horizontal);
            let dy = i128::from(radius - vertical);
            return dx * dx + dy * dy <= i128::from(radius).pow(2);
        }
    }
    true
}

impl UiHeadlessAppearanceFrameTranscript {
    /// Samples the scroll chrome of one mounted occurrence at a logical
    /// subpixel point, in paint order: the track first, then the thumb on top.
    ///
    /// This is the headless evidence read for a scrollbar. It honours the
    /// region's clip and the thumb's rounded contour, so a point inside the
    /// gutter but outside the thumb reports the track, and a point in a cut
    /// corner reports what lies beneath it.
    pub fn reference_scroll_chrome_at(
        &self,
        owner: worth_ui_host_contract::UiMountedInstanceIdentity,
        x: i64,
        y: i64,
    ) -> UiMountedAppearanceColor {
        let mut parts = self
            .mechanics
            .iter()
            .filter_map(|mechanic| match mechanic {
                UiHeadlessAppearanceMechanic::ScrollChrome(chrome)
                    if chrome.identity().owner_instance() == owner =>
                {
                    Some(chrome)
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        parts.sort_by_key(|chrome| chrome.identity().part().paint_ordinal());
        let mut layers = Vec::new();
        for chrome in parts {
            let rect = chrome.rect();
            let clip = chrome.clip();
            let bounds = [
                i64::from(rect.x()),
                i64::from(rect.y()),
                i64::from(rect.width()),
                i64::from(rect.height()),
            ];
            if !contains(
                [
                    i64::from(clip.x()),
                    i64::from(clip.y()),
                    i64::from(clip.width()),
                    i64::from(clip.height()),
                ],
                x,
                y,
            ) || !inside_contour(bounds, chrome.radii().corners().map(i64::from), x, y)
            {
                continue;
            }
            layers.push((chrome.background(), chrome.opacity()));
        }
        compose_source_over(layers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use worth_ui_host_contract::{
        UiAppearanceAllocationBounds, UiAppearanceClip, UiAppearanceLogicalLength,
        UiAppearanceNormalizedLogicalRadii, UiMountedInstanceIdentity, UiMountedNodeReceiptIssuer,
        UiMountedPresentationOpacity, UiMountedSurfaceAppearanceCompletionInput,
        UiMountedSurfaceBorderEdges,
    };

    #[test]
    fn reference_surface_omits_a_shared_edge_without_losing_exterior_edges() {
        let frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
        let issuer = UiMountedNodeReceiptIssuer::mint_for(frame).unwrap();
        let bounds = UiAppearanceAllocationBounds::new(0, 0, 10_000, 10_000).unwrap();
        let surface = UiMountedSurfaceAppearanceMechanic::complete_from_runtime_mounting(
            UiMountedSurfaceAppearanceCompletionInput {
            geometry: Default::default(),
                issuer,
                node_receipt: issuer
                    .receipt_for(UiMountedInstanceIdentity::mint_unbound().unwrap()),
                bounds,
                clip: UiAppearanceClip::new(0, 0, 10_000, 10_000).unwrap(),
                surface_paint_order: 0,
                portal_group: None,
                radii: UiAppearanceNormalizedLogicalRadii::normalize(
                    bounds,
                    [UiAppearanceLogicalLength::ZERO; 4],
                ),
                border_edges: UiMountedSurfaceBorderEdges::from_runtime_mosaic(
                    true, true, true, false,
                ),
                border_omissions: Box::new([]),
                paint: UiMountedSurfacePaint::Border {
                    color: UiMountedAppearanceColor::from_straight_srgba([20, 40, 60, 255]),
                    inward_width: UiAppearanceLogicalLength::new(1_000).unwrap(),
                },
                opacity: UiMountedPresentationOpacity::from_runtime_composition(u16::MAX),
                projection: worth_ui_host_contract::UiMountedNodeAppearanceAttribution::from_runtime_mounting(
                    issuer, 1, 1,
                )
                .unwrap(),
            },
        )
        .unwrap();
        let mut shared_edge = Vec::new();
        append_surface(&mut shared_edge, &surface, 0, 5_000);
        assert!(shared_edge.is_empty());
        let mut exterior_edge = Vec::new();
        append_surface(&mut exterior_edge, &surface, 5_000, 0);
        assert_eq!(exterior_edge.len(), 1);
    }
}
