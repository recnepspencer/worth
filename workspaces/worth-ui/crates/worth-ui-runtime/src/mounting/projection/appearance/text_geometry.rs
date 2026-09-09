//! Qualified candidate correspondence for foreground physical equivalence.
//! This is carried mounting evidence, not raster coverage or text authority.

use std::sync::Arc;
use worth_ui_host_contract::{
    UiFontCollectionGeneration, UiMountedCanonicalBox, UiMountedCoordinateSpace,
    UiMountedGeometryPosture, UiMountedPaintCommandIdentity, UiMountedSemanticTextMechanic,
    UiMountedTextPaintSpanIdentity, UiQualifiedTextLayoutIdentity, UiQualifiedTextLayoutWidthBasis,
    UiSemanticSurfaceIdentity, UiSurfaceBindingGeneration, UiTextOriginalRange,
    UiTextProfileGeneration, UiTextScaleGeneration,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiMountedAppearanceTextSpanInput {
    identity: UiMountedTextPaintSpanIdentity,
    geometry: Arc<[UiMountedAppearanceTextGeometry]>,
}

impl UiMountedAppearanceTextSpanInput {
    pub(in crate::mounting::projection) fn from_candidates(
        identity: UiMountedTextPaintSpanIdentity,
        mut geometry: Vec<UiMountedAppearanceTextGeometry>,
    ) -> Self {
        // Every candidate is from one exact mounted instance. Semantic command
        // correspondence supplies canonical order independently of receipt churn.
        geometry.sort_by_key(|row| row.command.semantic_text_identity_parts());
        Self {
            identity,
            geometry: geometry.into(),
        }
    }

    pub(crate) const fn identity(&self) -> UiMountedTextPaintSpanIdentity {
        self.identity
    }

    pub(super) fn geometry(&self) -> Arc<[UiMountedAppearanceTextGeometry]> {
        Arc::clone(&self.geometry)
    }

    pub(super) fn commands(&self) -> impl Iterator<Item = UiMountedPaintCommandIdentity> + '_ {
        self.geometry.iter().map(|geometry| geometry.command)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiMountedAppearanceTextGeometry {
    command: UiMountedPaintCommandIdentity,
    surface: UiSemanticSurfaceIdentity,
    // Runtime registration/rebind mints a fresh generation for its immutable
    // profile, including DPI. Qualification scale alone is not that evidence.
    binding: UiSurfaceBindingGeneration,
    layout: UiQualifiedTextLayoutIdentity,
    profile: UiTextProfileGeneration,
    fonts: UiFontCollectionGeneration,
    scale: UiTextScaleGeneration,
    width: UiQualifiedTextLayoutWidthBasis,
    range: UiTextOriginalRange,
    origin_bits: [u32; 2],
    clip: UiMountedTextClipBasis,
    layer: u32,
}

impl UiMountedAppearanceTextGeometry {
    pub(in crate::mounting::projection) fn from_candidate(
        candidate: &UiMountedSemanticTextMechanic,
        range: UiTextOriginalRange,
    ) -> Self {
        Self {
            command: UiMountedPaintCommandIdentity::semantic_text(candidate),
            surface: candidate.surface(),
            binding: candidate.binding(),
            layout: candidate.qualified_layout_identity(),
            profile: candidate.qualified_layout_profile(),
            fonts: candidate.qualified_layout_fonts(),
            scale: candidate.qualified_layout_scale(),
            width: candidate.qualified_layout_width(),
            range,
            origin_bits: [
                candidate.origin_x().to_bits(),
                candidate.origin_y().to_bits(),
            ],
            clip: UiMountedTextClipBasis::from_box(candidate.clip_bounds()),
            layer: candidate.layer_semantic_order(),
        }
    }

    pub(super) const fn command(&self) -> UiMountedPaintCommandIdentity {
        self.command
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct UiMountedTextClipBasis {
    rectangle_bits: [u32; 4],
    space: UiMountedCoordinateSpace,
    posture: UiMountedGeometryPosture,
}

impl UiMountedTextClipBasis {
    fn from_box(bounds: UiMountedCanonicalBox) -> Self {
        Self {
            rectangle_bits: [
                bounds.x().to_bits(),
                bounds.y().to_bits(),
                bounds.width().to_bits(),
                bounds.height().to_bits(),
            ],
            space: bounds.coordinate_space(),
            posture: bounds.posture(),
        }
    }
}
