use std::sync::Arc;

use worth_ui_host_contract::{
    UiGlyphRasterKey, UiGlyphRasterPinRequest, UiHostPresentationLineageIdentity,
    UiHostSurfaceIdentity, UiMountedContentGeneration, UiMountedFrameIdentity,
    UiMountedInstanceIdentity, UiMountedPaintCommandIdentity, UiMountedPresentationAttemptIdentity,
    UiMountedTextForegroundSpan, UiQualifiedTextLayoutIdentity,
    UiQualifiedTextLayoutRequestIdentity, UiQualifiedTextLayoutWidthBasis,
    UiSemanticSurfaceIdentity, UiSurfaceBindingGeneration, UiTextOriginalRange,
    UiTextScaleGeneration,
};

mod admission;
mod identity_parts;
mod raster_key_set;
pub use raster_key_set::WorthUiPresentationRasterKeySetBasis;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorthUiPresentationPaintSpanBasis {
    identity: [u8; 32],
    original_range: UiTextOriginalRange,
    foreground: [u8; 4],
}

impl WorthUiPresentationPaintSpanBasis {
    pub fn from_mounted(span: UiMountedTextForegroundSpan) -> Self {
        Self {
            identity: span.identity().digest(),
            original_range: span.original_range(),
            foreground: span.color().channels(),
        }
    }

    pub const fn identity(self) -> [u8; 32] {
        self.identity
    }
    pub const fn original_range(self) -> UiTextOriginalRange {
        self.original_range
    }
    pub const fn foreground(self) -> [u8; 4] {
        self.foreground
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiPresentationMechanicBasis {
    mounted_instance: UiMountedInstanceIdentity,
    mechanic: UiMountedPaintCommandIdentity,
    content_generation: UiMountedContentGeneration,
    content: Arc<str>,
    layout: UiQualifiedTextLayoutIdentity,
    layout_request: UiQualifiedTextLayoutRequestIdentity,
    layout_width: UiQualifiedTextLayoutWidthBasis,
    paint_spans: Box<[WorthUiPresentationPaintSpanBasis]>,
    raster_key_set: WorthUiPresentationRasterKeySetBasis,
    text_scale: UiTextScaleGeneration,
}

#[doc(hidden)]
#[derive(Clone)]
pub struct WorthUiPresentationMechanicBasisInput {
    pub mounted_instance: UiMountedInstanceIdentity,
    pub mechanic: UiMountedPaintCommandIdentity,
    pub content_generation: UiMountedContentGeneration,
    pub content: Arc<str>,
    pub layout: UiQualifiedTextLayoutIdentity,
    pub layout_request: UiQualifiedTextLayoutRequestIdentity,
    pub layout_width: UiQualifiedTextLayoutWidthBasis,
    pub paint_spans: Box<[WorthUiPresentationPaintSpanBasis]>,
    pub raster_keys: Box<[UiGlyphRasterKey]>,
    pub text_scale: UiTextScaleGeneration,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct WorthUiPresentationPinBasis {
    pin: UiGlyphRasterPinRequest,
}

impl WorthUiPresentationPinBasis {
    #[doc(hidden)]
    pub const fn from_runtime(pin: UiGlyphRasterPinRequest) -> Self {
        Self { pin }
    }
    pub const fn layout(self) -> UiQualifiedTextLayoutIdentity {
        self.pin.layout_identity()
    }
    pub const fn key(self) -> UiGlyphRasterKey {
        self.pin.key()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiPresentationRequestBasis {
    mounted_frame: UiMountedFrameIdentity,
    semantic_surface: UiSemanticSurfaceIdentity,
    host_surface: UiHostSurfaceIdentity,
    binding: UiSurfaceBindingGeneration,
    complete: bool,
    mechanics: Box<[WorthUiPresentationMechanicBasis]>,
    removed_mechanics: Box<[UiMountedPaintCommandIdentity]>,
    binding_pins: Box<[WorthUiPresentationPinBasis]>,
    pin_additions: Box<[WorthUiPresentationPinBasis]>,
    pin_releases: Box<[WorthUiPresentationPinBasis]>,
    dpi_milli: u32,
    attempt: UiMountedPresentationAttemptIdentity,
    predecessor: Option<UiMountedFrameIdentity>,
    host_lineage: UiHostPresentationLineageIdentity,
}

#[doc(hidden)]
#[derive(Clone)]
pub struct WorthUiPresentationRequestBasisInput {
    pub mounted_frame: UiMountedFrameIdentity,
    pub semantic_surface: UiSemanticSurfaceIdentity,
    pub host_surface: UiHostSurfaceIdentity,
    pub binding: UiSurfaceBindingGeneration,
    pub complete: bool,
    pub mechanics: Box<[WorthUiPresentationMechanicBasisInput]>,
    pub removed_mechanics: Box<[UiMountedPaintCommandIdentity]>,
    pub binding_pins: Box<[WorthUiPresentationPinBasis]>,
    pub pin_additions: Box<[WorthUiPresentationPinBasis]>,
    pub pin_releases: Box<[WorthUiPresentationPinBasis]>,
    pub dpi_milli: u32,
    pub attempt: UiMountedPresentationAttemptIdentity,
    pub predecessor: Option<UiMountedFrameIdentity>,
    pub host_lineage: UiHostPresentationLineageIdentity,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthUiPresentationRequestBasisDenial {
    ZeroDpi,
    EmptyPaintSpan,
    DuplicatePaintSpan,
    OverlappingPaintSpan,
    DuplicateMechanic,
    DuplicateRemovedMechanic,
    MechanicAlsoRemoved,
    DuplicatePinAddition,
    DuplicatePinRelease,
    DuplicateBindingPin,
    NonTextMechanic,
    MechanicMountedInstanceMismatch,
}

impl WorthUiPresentationRequestBasis {
    pub fn mechanics(&self) -> &[WorthUiPresentationMechanicBasis] {
        &self.mechanics
    }
    pub fn removed_mechanics(&self) -> &[UiMountedPaintCommandIdentity] {
        &self.removed_mechanics
    }
    pub fn pin_additions(&self) -> &[WorthUiPresentationPinBasis] {
        &self.pin_additions
    }
    pub fn binding_pins(&self) -> &[WorthUiPresentationPinBasis] {
        &self.binding_pins
    }
    pub fn pin_releases(&self) -> &[WorthUiPresentationPinBasis] {
        &self.pin_releases
    }
    pub const fn dpi_milli(&self) -> u32 {
        self.dpi_milli
    }
    pub const fn attempt(&self) -> UiMountedPresentationAttemptIdentity {
        self.attempt
    }
    pub const fn semantic_surface(&self) -> UiSemanticSurfaceIdentity {
        self.semantic_surface
    }
    pub const fn host_surface(&self) -> UiHostSurfaceIdentity {
        self.host_surface
    }
    pub const fn binding(&self) -> UiSurfaceBindingGeneration {
        self.binding
    }
    pub const fn complete(&self) -> bool {
        self.complete
    }
    pub const fn host_lineage(&self) -> UiHostPresentationLineageIdentity {
        self.host_lineage
    }
    pub const fn mounted_frame(&self) -> UiMountedFrameIdentity {
        self.mounted_frame
    }
    pub const fn predecessor(&self) -> Option<UiMountedFrameIdentity> {
        self.predecessor
    }
}

impl WorthUiPresentationMechanicBasis {
    pub const fn mounted_instance(&self) -> UiMountedInstanceIdentity {
        self.mounted_instance
    }
    pub const fn mechanic(&self) -> UiMountedPaintCommandIdentity {
        self.mechanic
    }
    pub const fn layout(&self) -> UiQualifiedTextLayoutIdentity {
        self.layout
    }
    pub const fn content_generation(&self) -> UiMountedContentGeneration {
        self.content_generation
    }
    pub fn content(&self) -> &str {
        &self.content
    }
    pub const fn layout_request(&self) -> UiQualifiedTextLayoutRequestIdentity {
        self.layout_request
    }
    pub const fn layout_width(&self) -> UiQualifiedTextLayoutWidthBasis {
        self.layout_width
    }
    pub fn paint_spans(&self) -> &[WorthUiPresentationPaintSpanBasis] {
        &self.paint_spans
    }
    pub fn raster_key_set(&self) -> &WorthUiPresentationRasterKeySetBasis {
        &self.raster_key_set
    }
    pub const fn text_scale(&self) -> UiTextScaleGeneration {
        self.text_scale
    }
}

#[cfg(test)]
#[path = "request_basis_tests.rs"]
mod tests;
