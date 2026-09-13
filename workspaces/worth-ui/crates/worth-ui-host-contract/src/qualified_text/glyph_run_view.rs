//! Borrowed glyph-run draw attribution.
//!
//! Each run names the mounted mechanic, layout, paint span, and original
//! range together with the exact raster key. Attribution is not a cache key.

use super::{UiGlyphRasterKey, UiQualifiedTextLayoutIdentity, UiTextOriginalRange};
use crate::UiMountedRgba8;
use crate::{UiMountedCanonicalBox, UiMountedPaintCommandIdentity, UiMountedTextPaintSpanIdentity};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UiGlyphRunView {
    mechanic: UiMountedPaintCommandIdentity,
    layout: UiQualifiedTextLayoutIdentity,
    paint_span: UiMountedTextPaintSpanIdentity,
    original_range: UiTextOriginalRange,
    foreground: UiMountedRgba8,
    raster_key: UiGlyphRasterKey,
    origin_x_millipoints: i64,
    origin_y_millipoints: i64,
    line_index: u32,
    visual_run_index: u32,
    clip_bounds: UiMountedCanonicalBox,
    layer_semantic_order: u32,
}

#[doc(hidden)]
#[derive(Clone, Copy)]
pub struct UiGlyphRunViewInput {
    pub mechanic: UiMountedPaintCommandIdentity,
    pub layout: UiQualifiedTextLayoutIdentity,
    pub paint_span: UiMountedTextPaintSpanIdentity,
    pub original_range: UiTextOriginalRange,
    pub foreground: UiMountedRgba8,
    pub raster_key: UiGlyphRasterKey,
    pub origin_x_millipoints: i64,
    pub origin_y_millipoints: i64,
    pub line_index: u32,
    pub visual_run_index: u32,
    pub clip_bounds: UiMountedCanonicalBox,
    pub layer_semantic_order: u32,
}

impl UiGlyphRunView {
    #[doc(hidden)]
    pub const fn from_text_mechanics(input: UiGlyphRunViewInput) -> Self {
        Self {
            mechanic: input.mechanic,
            layout: input.layout,
            paint_span: input.paint_span,
            original_range: input.original_range,
            foreground: input.foreground,
            raster_key: input.raster_key,
            origin_x_millipoints: input.origin_x_millipoints,
            origin_y_millipoints: input.origin_y_millipoints,
            line_index: input.line_index,
            visual_run_index: input.visual_run_index,
            clip_bounds: input.clip_bounds,
            layer_semantic_order: input.layer_semantic_order,
        }
    }

    pub const fn mechanic(self) -> UiMountedPaintCommandIdentity {
        self.mechanic
    }

    pub const fn layout_identity(self) -> UiQualifiedTextLayoutIdentity {
        self.layout
    }

    pub const fn paint_span(self) -> UiMountedTextPaintSpanIdentity {
        self.paint_span
    }

    pub const fn original_range(self) -> UiTextOriginalRange {
        self.original_range
    }

    pub const fn foreground(self) -> UiMountedRgba8 {
        self.foreground
    }

    pub const fn raster_key(self) -> UiGlyphRasterKey {
        self.raster_key
    }

    pub const fn origin_x_millipoints(self) -> i64 {
        self.origin_x_millipoints
    }

    pub const fn origin_y_millipoints(self) -> i64 {
        self.origin_y_millipoints
    }

    pub const fn line_index(self) -> u32 {
        self.line_index
    }

    pub const fn visual_run_index(self) -> u32 {
        self.visual_run_index
    }

    pub const fn clip_bounds(self) -> UiMountedCanonicalBox {
        self.clip_bounds
    }

    pub const fn layer_semantic_order(self) -> u32 {
        self.layer_semantic_order
    }

    /// Canonical headless transcript row shared by Runtime and native
    /// presentation evidence. The mounted owner identities remain the truth;
    /// these bytes only make their exact agreement independently comparable.
    pub fn canonical_transcript_bytes(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(320);
        bytes.extend_from_slice(
            &self
                .mechanic()
                .mounted_instance()
                .diagnostic_value()
                .to_le_bytes(),
        );
        let (slot, row) = self
            .mechanic()
            .semantic_text_identity_parts()
            .expect("glyph-run transcript remains semantic text");
        bytes.extend_from_slice(&slot.to_le_bytes());
        bytes.push(u8::from(row.is_some()));
        if let Some(row) = row {
            bytes.extend_from_slice(&row);
        }
        bytes.extend_from_slice(&self.layout_identity().digest());
        bytes.extend_from_slice(&self.paint_span().digest());
        bytes.extend_from_slice(&self.original_range().start().to_le_bytes());
        bytes.extend_from_slice(&self.original_range().end().to_le_bytes());
        bytes.extend_from_slice(&self.foreground().channels());
        bytes.extend_from_slice(&self.raster_key().canonical_evidence_bytes());
        bytes.extend_from_slice(&self.origin_x_millipoints().to_le_bytes());
        bytes.extend_from_slice(&self.origin_y_millipoints().to_le_bytes());
        bytes.extend_from_slice(&self.line_index().to_le_bytes());
        bytes.extend_from_slice(&self.visual_run_index().to_le_bytes());
        let clip = self.clip_bounds();
        bytes.extend_from_slice(&clip.x().to_bits().to_le_bytes());
        bytes.extend_from_slice(&clip.y().to_bits().to_le_bytes());
        bytes.extend_from_slice(&clip.width().to_bits().to_le_bytes());
        bytes.extend_from_slice(&clip.height().to_bits().to_le_bytes());
        bytes.extend_from_slice(&self.layer_semantic_order().to_le_bytes());
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::qualified_text::raster_key::{
        UiGlyphRasterFractionalOrigin, UiGlyphRasterKeyInput, UiGlyphRasterPalette,
        UiGlyphRasterSize, UiGlyphRasterSource, UiGlyphVariationCoordinates,
    };
    use crate::{
        UiFontCollectionGeneration, UiFontCollectionLineageIdentity,
        UiHostObservationPresentationBasis, UiHostPresentationEpoch, UiHostSurfaceIdentity,
        UiMountedCanonicalBox, UiMountedCanonicalBoxInput, UiMountedCoordinateSpace,
        UiMountedFrameIdentity, UiMountedInstanceIdentity, UiMountedNodeReceiptIssuer,
        UiMountedPortalInputShielding, UiMountedPortalOverlayCompletionInput,
        UiMountedPortalOverlayLifecyclePosture, UiMountedPortalOverlayMechanic, UiMountedRgba8,
        UiQualifiedFontFaceIdentity, UiSemanticSurfaceIdentity, UiSurfaceBindingGeneration,
        UiTextProfileGeneration,
    };

    fn sample_key() -> UiGlyphRasterKey {
        UiGlyphRasterKey::from_text_mechanics(UiGlyphRasterKeyInput {
            font_collection: UiFontCollectionGeneration::new(2).unwrap(),
            font_collection_lineage: UiFontCollectionLineageIdentity::from_text_mechanics([5; 32]),
            profile: UiTextProfileGeneration::new(4).unwrap(),
            face: UiQualifiedFontFaceIdentity::from_text_mechanics([6; 32], 1),
            glyph_id: 21,
            variations: UiGlyphVariationCoordinates::empty(),
            palette: UiGlyphRasterPalette::new(1),
            size: UiGlyphRasterSize::from_millipoints(16_000).unwrap(),
            source: UiGlyphRasterSource::ColorBitmap,
            dpi_milli: 2_000,
            origin: UiGlyphRasterFractionalOrigin::from_sixty_fourths(16, 0),
        })
        .unwrap()
    }

    fn mechanic_identity() -> UiMountedPaintCommandIdentity {
        let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
        let mounted_instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
        let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
        let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
        let bounds = UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
            x: 0.0,
            y: 0.0,
            width: 16.0,
            height: 16.0,
            coordinate_space: UiMountedCoordinateSpace::Viewport,
        })
        .unwrap();
        let mechanic = UiMountedPortalOverlayMechanic::complete_from_runtime_mounting(
            UiMountedPortalOverlayCompletionInput {
                frame,
                surface,
                binding,
                owner: mounted_instance,
                owner_receipt: UiMountedNodeReceiptIssuer::mint_for(frame)
                    .unwrap()
                    .receipt_for(mounted_instance),
                portal_identity: mounted_instance.diagnostic_value(),
                anchor_presentation: UiHostObservationPresentationBasis::new(
                    UiHostSurfaceIdentity::mint_unbound().unwrap(),
                    frame,
                    binding,
                    UiHostPresentationEpoch::issued_by_host(1),
                ),
                anchor_bounds: bounds,
                bounds,
                color: UiMountedRgba8::new(0, 0, 0, 255),
                layer_semantic_order: 0,
                layer_depth: 0,
                clip_bounds: bounds,
                lifecycle: UiMountedPortalOverlayLifecyclePosture::Visible,
                shielding: UiMountedPortalInputShielding::ContentBounds,
            },
        )
        .unwrap();
        UiMountedPaintCommandIdentity::portal_overlay(&mechanic)
    }

    #[test]
    fn glyph_run_view_ties_draw_attribution_to_the_exact_key() {
        let mechanic = mechanic_identity();
        let layout = UiQualifiedTextLayoutIdentity::from_text_mechanics([9; 32]);
        let view = UiGlyphRunView::from_text_mechanics(UiGlyphRunViewInput {
            mechanic,
            layout,
            paint_span: UiMountedTextPaintSpanIdentity::from_runtime_mounting([3; 32]),
            original_range: UiTextOriginalRange::new(4, 8).unwrap(),
            foreground: UiMountedRgba8::new(12, 34, 56, 255),
            raster_key: sample_key(),
            origin_x_millipoints: 12_500,
            origin_y_millipoints: 27_000,
            line_index: 2,
            visual_run_index: 3,
            clip_bounds: mechanic_clip(),
            layer_semantic_order: 4,
        });
        assert_eq!(view.mechanic(), mechanic);
        assert_eq!(view.layout_identity(), layout);
        assert_eq!(view.paint_span().digest(), [3; 32]);
        assert_eq!(view.original_range().start(), 4);
        assert_eq!(view.foreground(), UiMountedRgba8::new(12, 34, 56, 255));
        assert_eq!(view.raster_key().glyph_id(), 21);
        assert_eq!(view.raster_key().source(), UiGlyphRasterSource::ColorBitmap);
        assert_eq!(view.origin_x_millipoints(), 12_500);
        assert_eq!(view.origin_y_millipoints(), 27_000);
        assert_eq!(view.line_index(), 2);
        assert_eq!(view.visual_run_index(), 3);
        assert_eq!(view.layer_semantic_order(), 4);
    }

    fn mechanic_clip() -> UiMountedCanonicalBox {
        UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
            x: 0.0,
            y: 0.0,
            width: 32.0,
            height: 32.0,
            coordinate_space: UiMountedCoordinateSpace::HostSurface,
        })
        .unwrap()
    }
}
