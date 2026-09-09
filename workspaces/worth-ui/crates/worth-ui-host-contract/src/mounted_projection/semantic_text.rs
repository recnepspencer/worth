use std::sync::Arc;

use crate::{
    UiMountedContentGeneration, UiMountedFrameIdentity, UiMountedInstanceIdentity,
    UiMountedNodeReceiptIdentity, UiSemanticSurfaceIdentity, UiSurfaceBindingGeneration,
    WorthUiHostCapabilityObservationGeneration,
};

mod appearance_clip;
mod foreground;
mod frame_affinity;
mod table;
mod validation;

pub use foreground::{UiMountedTextForegroundSpan, UiMountedTextPaintSpanIdentity};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiMountedTextSchemaVersion(u16);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiSemanticTextProfile {
    BodyDefault,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiSemanticTextWrapPosture {
    Clip,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiSemanticTextBaselinePosture {
    Alphabetic,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum UiSemanticTextSlot {
    Value,
    CollectionValue { selected_field_ordinal: u16 },
    Posture,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct UiMountedCollectionRowCorrelation([u8; 32]);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiMountedSemanticTextReference(u16);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountedSemanticTextCompletionDenial {
    NonAreaGeometry,
    ClipMismatch,
    InvalidTextOrigin,
    NodeReceiptFrameMismatch,
    NodeReceiptInstanceMismatch,
    CollectionIdentityMismatch,
    ContentCapacityExceeded,
    QualifiedLayoutSourceMismatch,
    QualifiedLayoutGenerationMismatch,
    ForegroundSpanMismatch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiMountedSemanticTextTableDenial {
    CapacityExceeded,
    ByteCapacityExceeded,
}

#[doc(hidden)]
pub trait UiMountedQualifiedTextResolver {
    fn resolve(
        &self,
        identity: crate::UiQualifiedTextLayoutIdentity,
    ) -> Option<crate::UiQualifiedTextLayoutView<'_>>;
}

#[doc(hidden)]
impl UiMountedQualifiedTextResolver for () {
    fn resolve(
        &self,
        _identity: crate::UiQualifiedTextLayoutIdentity,
    ) -> Option<crate::UiQualifiedTextLayoutView<'_>> {
        None
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiMountedSemanticTextMechanic {
    schema: UiMountedTextSchemaVersion,
    content_generation: UiMountedContentGeneration,
    frame: UiMountedFrameIdentity,
    surface: UiSemanticSurfaceIdentity,
    binding: UiSurfaceBindingGeneration,
    mounted_instance: UiMountedInstanceIdentity,
    node_receipt: UiMountedNodeReceiptIdentity,
    allocation_basis: super::UiMountedAllocationBasis,
    bounds: super::UiMountedCanonicalBox,
    clip_bounds: super::UiMountedCanonicalBox,
    origin_x: f32,
    origin_y: f32,
    text: Arc<str>,
    layout_identity: crate::UiQualifiedTextLayoutIdentity,
    layout_request: crate::UiQualifiedTextLayoutRequestIdentity,
    layout_profile: crate::UiTextProfileGeneration,
    layout_fonts: crate::UiFontCollectionGeneration,
    layout_scale: crate::UiTextScaleGeneration,
    layout_width: crate::UiQualifiedTextLayoutWidthBasis,
    slot: UiSemanticTextSlot,
    collection_row: Option<UiMountedCollectionRowCorrelation>,
    foregrounds: Arc<[UiMountedTextForegroundSpan]>,
    profile: UiSemanticTextProfile,
    layer_semantic_order: u32,
    capability_generation: WorthUiHostCapabilityObservationGeneration,
    capability_profile_digest: u64,
    performed_layout_cost: Option<crate::UiQualifiedTextCostRecord>,
    semantic_digest: u64,
}

#[doc(hidden)]
#[derive(Clone)]
pub struct UiMountedSemanticTextCompletionInput<'layout> {
    pub content_generation: UiMountedContentGeneration,
    pub frame: UiMountedFrameIdentity,
    pub surface: UiSemanticSurfaceIdentity,
    pub binding: UiSurfaceBindingGeneration,
    pub mounted_instance: UiMountedInstanceIdentity,
    pub node_receipt: UiMountedNodeReceiptIdentity,
    pub allocation_basis: super::UiMountedAllocationBasis,
    pub bounds: super::UiMountedCanonicalBox,
    pub clip_bounds: super::UiMountedCanonicalBox,
    pub origin_x: f32,
    pub origin_y: f32,
    pub text: Arc<str>,
    pub layout: crate::UiQualifiedTextLayoutView<'layout>,
    pub slot: UiSemanticTextSlot,
    pub collection_row: Option<UiMountedCollectionRowCorrelation>,
    pub foregrounds: Arc<[UiMountedTextForegroundSpan]>,
    pub profile: UiSemanticTextProfile,
    pub layer_semantic_order: u32,
    pub capability_generation: WorthUiHostCapabilityObservationGeneration,
    pub capability_profile_digest: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiMountedSemanticTextTable {
    schema: UiMountedTextSchemaVersion,
    rows: std::sync::Arc<[UiMountedSemanticTextMechanic]>,
}

impl UiMountedTextSchemaVersion {
    pub const REQUIRED_MOUNTED_FRAME_REVISION: u16 = 5;

    pub const fn current() -> Self {
        Self(3)
    }

    pub const fn revision(self) -> u16 {
        self.0
    }
}

impl UiSemanticTextProfile {
    pub const fn size_millipoints(self) -> u16 {
        match self {
            Self::BodyDefault => 14_000,
        }
    }

    pub const fn weight(self) -> u16 {
        match self {
            Self::BodyDefault => 400,
        }
    }

    pub const fn wrap(self) -> UiSemanticTextWrapPosture {
        match self {
            Self::BodyDefault => UiSemanticTextWrapPosture::Clip,
        }
    }

    pub const fn baseline(self) -> UiSemanticTextBaselinePosture {
        match self {
            Self::BodyDefault => UiSemanticTextBaselinePosture::Alphabetic,
        }
    }
}

impl UiMountedSemanticTextReference {
    #[doc(hidden)]
    pub const fn from_runtime_mounting(index: u16) -> Self {
        Self(index)
    }

    pub const fn index(self) -> u16 {
        self.0
    }
}

impl UiMountedCollectionRowCorrelation {
    #[doc(hidden)]
    pub const fn from_runtime_mounting(identity: [u8; 32]) -> Self {
        Self(identity)
    }

    pub const fn correlation_digest(self) -> [u8; 32] {
        self.0
    }
}

impl UiMountedSemanticTextMechanic {
    pub const MAX_CONTENT_BYTES: usize = 65_536;

    #[doc(hidden)]
    pub fn complete_from_runtime_mounting(
        input: UiMountedSemanticTextCompletionInput,
    ) -> Result<Self, UiMountedSemanticTextCompletionDenial> {
        Self::complete(input, true)
    }

    #[doc(hidden)]
    pub fn complete_from_runtime_mounting_with_reused_layout(
        input: UiMountedSemanticTextCompletionInput,
    ) -> Result<Self, UiMountedSemanticTextCompletionDenial> {
        Self::complete(input, false)
    }

    fn complete(
        input: UiMountedSemanticTextCompletionInput,
        layout_performed: bool,
    ) -> Result<Self, UiMountedSemanticTextCompletionDenial> {
        validation::validate_completion(&input)?;
        let semantic_digest = validation::semantic_digest(&input);
        let performed_layout_cost = layout_performed.then(|| input.layout.cost());
        Ok(Self {
            schema: UiMountedTextSchemaVersion::current(),
            content_generation: input.content_generation,
            frame: input.frame,
            surface: input.surface,
            binding: input.binding,
            mounted_instance: input.mounted_instance,
            node_receipt: input.node_receipt,
            allocation_basis: input.allocation_basis,
            bounds: input.bounds,
            clip_bounds: input.clip_bounds,
            origin_x: input.origin_x,
            origin_y: input.origin_y,
            text: input.text,
            layout_identity: input.layout.identity(),
            layout_request: input.layout.request_identity(),
            layout_profile: input.layout.profile_generation(),
            layout_fonts: input.layout.font_collection_generation(),
            layout_scale: input.layout.text_scale_generation(),
            layout_width: input.layout.width_basis(),
            slot: input.slot,
            collection_row: input.collection_row,
            foregrounds: input.foregrounds,
            profile: input.profile,
            layer_semantic_order: input.layer_semantic_order,
            capability_generation: input.capability_generation,
            capability_profile_digest: input.capability_profile_digest,
            performed_layout_cost,
            semantic_digest,
        })
    }

    pub const fn schema(&self) -> UiMountedTextSchemaVersion {
        self.schema
    }
    pub const fn content_generation(&self) -> UiMountedContentGeneration {
        self.content_generation
    }
    pub const fn frame(&self) -> UiMountedFrameIdentity {
        self.frame
    }
    pub const fn surface(&self) -> UiSemanticSurfaceIdentity {
        self.surface
    }
    pub const fn binding(&self) -> UiSurfaceBindingGeneration {
        self.binding
    }
    pub const fn mounted_instance(&self) -> UiMountedInstanceIdentity {
        self.mounted_instance
    }
    pub const fn node_receipt(&self) -> UiMountedNodeReceiptIdentity {
        self.node_receipt
    }
    pub const fn allocation_basis(&self) -> super::UiMountedAllocationBasis {
        self.allocation_basis
    }
    pub const fn bounds(&self) -> super::UiMountedCanonicalBox {
        self.bounds
    }
    pub const fn clip_bounds(&self) -> super::UiMountedCanonicalBox {
        self.clip_bounds
    }
    pub fn origin_x(&self) -> f32 {
        self.origin_x
    }
    pub fn origin_y(&self) -> f32 {
        self.origin_y
    }
    pub fn text(&self) -> &str {
        &self.text
    }
    pub const fn qualified_layout_identity(&self) -> crate::UiQualifiedTextLayoutIdentity {
        self.layout_identity
    }
    pub const fn qualified_layout_request(&self) -> crate::UiQualifiedTextLayoutRequestIdentity {
        self.layout_request
    }
    pub const fn qualified_layout_profile(&self) -> crate::UiTextProfileGeneration {
        self.layout_profile
    }
    pub const fn qualified_layout_fonts(&self) -> crate::UiFontCollectionGeneration {
        self.layout_fonts
    }
    pub const fn qualified_layout_scale(&self) -> crate::UiTextScaleGeneration {
        self.layout_scale
    }
    pub const fn qualified_layout_width(&self) -> crate::UiQualifiedTextLayoutWidthBasis {
        self.layout_width
    }
    pub const fn slot(&self) -> UiSemanticTextSlot {
        self.slot
    }
    pub fn collection_row(&self) -> Option<&UiMountedCollectionRowCorrelation> {
        self.collection_row.as_ref()
    }
    pub fn foregrounds(&self) -> &[UiMountedTextForegroundSpan] {
        &self.foregrounds
    }
    pub const fn profile(&self) -> UiSemanticTextProfile {
        self.profile
    }
    pub const fn layer_semantic_order(&self) -> u32 {
        self.layer_semantic_order
    }
    pub const fn capability_generation(&self) -> WorthUiHostCapabilityObservationGeneration {
        self.capability_generation
    }
    pub const fn capability_profile_digest(&self) -> u64 {
        self.capability_profile_digest
    }
    pub const fn performed_layout_cost(&self) -> Option<crate::UiQualifiedTextCostRecord> {
        self.performed_layout_cost
    }
    pub const fn semantic_digest(&self) -> u64 {
        self.semantic_digest
    }

    #[doc(hidden)]
    pub fn presented_within_portal(
        &self,
        portal: super::UiMountedPortalOverlayMechanic,
    ) -> Result<Option<Self>, UiMountedSemanticTextCompletionDenial> {
        let Some(geometry) =
            super::portal_child_geometry::project(self.bounds, self.clip_bounds, portal)
                .map_err(|_| UiMountedSemanticTextCompletionDenial::NonAreaGeometry)?
        else {
            return Ok(None);
        };
        let bounds = geometry.bounds;
        let mut presented = self.clone();
        presented.bounds = bounds;
        presented.clip_bounds = geometry.clip;
        presented.origin_x = self.origin_x + portal.bounds().x() - portal.anchor_bounds().x();
        presented.origin_y = self.origin_y + portal.bounds().y() - portal.anchor_bounds().y();
        presented.layer_semantic_order = portal
            .layer_semantic_order()
            .saturating_add(1 + self.layer_semantic_order.min(1_024));
        let max_x = bounds.x() + bounds.width();
        let max_y = bounds.y() + bounds.height();
        if presented.origin_x < bounds.x()
            || presented.origin_x > max_x
            || presented.origin_y < bounds.y()
            || presented.origin_y > max_y
        {
            return Err(UiMountedSemanticTextCompletionDenial::InvalidTextOrigin);
        }
        presented.semantic_digest = validation::semantic_digest_mechanic(&presented);
        Ok(Some(presented))
    }
}

#[cfg(test)]
#[path = "semantic_text_tests.rs"]
pub(super) mod tests;
