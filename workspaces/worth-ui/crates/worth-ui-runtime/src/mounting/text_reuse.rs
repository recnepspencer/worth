//! Exact text foreground reuse evidence shared by mounting and presentation.

use std::sync::Arc;

use worth_ui_host_contract::{
    UiGlyphRasterPinRequest, UiHostPresentationLineageIdentity, UiHostSurfaceBaselineIdentity,
    UiHostSurfaceIdentity, UiMountedAllocationBasis, UiMountedCanonicalBox,
    UiMountedPaintCommandIdentity, UiMountedPresentationWorkView, UiMountedSemanticTextMechanic,
    UiMountedTextForegroundSpan, UiQualifiedTextLayoutIdentity,
    UiQualifiedTextLayoutRequestIdentity, UiQualifiedTextLayoutWidthBasis,
    UiSemanticSurfaceIdentity, UiSemanticTextProfile, UiSurfaceBindingGeneration,
    UiTextProfileGeneration, UiTextScaleGeneration, WorthUiHostCapabilityObservationGeneration,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UiMountedTextForegroundPresentationBasis {
    predecessor: Option<worth_ui_host_contract::UiMountedFrameIdentity>,
    successor: worth_ui_host_contract::UiMountedFrameIdentity,
    surface: UiSemanticSurfaceIdentity,
    binding: UiSurfaceBindingGeneration,
    host_surface: UiHostSurfaceIdentity,
    baseline: UiHostSurfaceBaselineIdentity,
    capability_generation: WorthUiHostCapabilityObservationGeneration,
    capability_profile_digest: u64,
    presentation_mode: worth_ui_host_contract::UiHostSurfacePresentationMode,
    dpi_milli: u32,
    host_lineage: Option<UiHostPresentationLineageIdentity>,
    damage_digest: [u8; 32],
}

impl UiMountedTextForegroundPresentationBasis {
    pub(crate) fn from_work(
        work: UiMountedPresentationWorkView<'_>,
        requirement: worth_ui_host_contract::UiMountedSurfaceBindingRequirement,
        dpi_milli: u32,
        host_lineage: Option<UiHostPresentationLineageIdentity>,
        damage_digest: [u8; 32],
    ) -> Self {
        let affinity = work.affinity();
        Self {
            predecessor: affinity.predecessor(),
            successor: affinity.successor(),
            surface: affinity.surface(),
            binding: affinity.binding(),
            host_surface: requirement.host_surface(),
            baseline: affinity.baseline(),
            capability_generation: requirement.capability_generation(),
            capability_profile_digest: requirement.capability_profile_digest(),
            presentation_mode: requirement.presentation_mode(),
            dpi_milli,
            host_lineage,
            damage_digest,
        }
    }

    pub(crate) fn admits_successor(self, candidate: Self) -> bool {
        candidate.predecessor == Some(self.successor)
            && candidate.surface == self.surface
            && candidate.binding == self.binding
            && candidate.host_surface == self.host_surface
            && candidate.baseline == self.baseline
            && candidate.capability_generation == self.capability_generation
            && candidate.capability_profile_digest == self.capability_profile_digest
            && candidate.presentation_mode == self.presentation_mode
            && candidate.dpi_milli == self.dpi_milli
            && candidate.host_lineage == self.host_lineage
            && candidate.damage_digest == self.damage_digest
    }

    pub(crate) const fn binding(self) -> UiSurfaceBindingGeneration {
        self.binding
    }

    pub(crate) const fn successor(self) -> worth_ui_host_contract::UiMountedFrameIdentity {
        self.successor
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct UiMountedTextForegroundReuseMechanic {
    command: UiMountedPaintCommandIdentity,
    surface: UiSemanticSurfaceIdentity,
    binding: UiSurfaceBindingGeneration,
    mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    layout: UiQualifiedTextLayoutIdentity,
    layout_request: UiQualifiedTextLayoutRequestIdentity,
    layout_profile: UiTextProfileGeneration,
    layout_fonts: worth_ui_host_contract::UiFontCollectionGeneration,
    layout_scale: UiTextScaleGeneration,
    layout_width: UiQualifiedTextLayoutWidthBasis,
    text: Arc<str>,
    slot: worth_ui_host_contract::UiSemanticTextSlot,
    collection_row: Option<worth_ui_host_contract::UiMountedCollectionRowCorrelation>,
    foregrounds: Box<[UiMountedTextForegroundSpan]>,
    profile: UiSemanticTextProfile,
    layer_semantic_order: u32,
    capability_generation: WorthUiHostCapabilityObservationGeneration,
    capability_profile_digest: u64,
    allocation_basis: UiMountedAllocationBasis,
    bounds: UiMountedCanonicalBoxBasis,
    clip_bounds: UiMountedCanonicalBoxBasis,
    origin_x_bits: u32,
    origin_y_bits: u32,
}

impl UiMountedTextForegroundReuseMechanic {
    fn from_mechanic(
        command: UiMountedPaintCommandIdentity,
        mechanic: &UiMountedSemanticTextMechanic,
    ) -> Self {
        Self {
            command,
            surface: mechanic.surface(),
            binding: mechanic.binding(),
            mounted_instance: mechanic.mounted_instance(),
            layout: mechanic.qualified_layout_identity(),
            layout_request: mechanic.qualified_layout_request(),
            layout_profile: mechanic.qualified_layout_profile(),
            layout_fonts: mechanic.qualified_layout_fonts(),
            layout_scale: mechanic.qualified_layout_scale(),
            layout_width: mechanic.qualified_layout_width(),
            text: Arc::from(mechanic.text()),
            slot: mechanic.slot(),
            collection_row: mechanic.collection_row().copied(),
            foregrounds: mechanic.foregrounds().to_vec().into_boxed_slice(),
            profile: mechanic.profile(),
            layer_semantic_order: mechanic.layer_semantic_order(),
            capability_generation: mechanic.capability_generation(),
            capability_profile_digest: mechanic.capability_profile_digest(),
            allocation_basis: mechanic.allocation_basis(),
            bounds: UiMountedCanonicalBoxBasis::from_box(mechanic.bounds()),
            clip_bounds: UiMountedCanonicalBoxBasis::from_box(mechanic.clip_bounds()),
            origin_x_bits: mechanic.origin_x().to_bits(),
            origin_y_bits: mechanic.origin_y().to_bits(),
        }
    }

    fn accepts_successor(
        &self,
        command: UiMountedPaintCommandIdentity,
        mechanic: &UiMountedSemanticTextMechanic,
        successor_frame: worth_ui_host_contract::UiMountedFrameIdentity,
    ) -> bool {
        self.command == command
            && mechanic.frame() == successor_frame
            && self.surface == mechanic.surface()
            && self.binding == mechanic.binding()
            && self.mounted_instance == mechanic.mounted_instance()
            && self.layout == mechanic.qualified_layout_identity()
            && self.layout_request == mechanic.qualified_layout_request()
            && self.layout_profile == mechanic.qualified_layout_profile()
            && self.layout_fonts == mechanic.qualified_layout_fonts()
            && self.layout_scale == mechanic.qualified_layout_scale()
            && self.layout_width == mechanic.qualified_layout_width()
            && self.text.as_ref() == mechanic.text()
            && self.slot == mechanic.slot()
            && self.collection_row.as_ref() == mechanic.collection_row()
            && same_foreground_shape(&self.foregrounds, mechanic.foregrounds())
            && self.profile == mechanic.profile()
            && self.layer_semantic_order == mechanic.layer_semantic_order()
            && self.capability_generation == mechanic.capability_generation()
            && self.capability_profile_digest == mechanic.capability_profile_digest()
            && self.allocation_basis == mechanic.allocation_basis()
            && self.bounds == UiMountedCanonicalBoxBasis::from_box(mechanic.bounds())
            && self.clip_bounds == UiMountedCanonicalBoxBasis::from_box(mechanic.clip_bounds())
            && self.origin_x_bits == mechanic.origin_x().to_bits()
            && self.origin_y_bits == mechanic.origin_y().to_bits()
            && mechanic.performed_layout_cost().is_none()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct UiMountedCanonicalBoxBasis {
    x_bits: u32,
    y_bits: u32,
    width_bits: u32,
    height_bits: u32,
    coordinate_space: worth_ui_host_contract::UiMountedCoordinateSpace,
    posture: worth_ui_host_contract::UiMountedGeometryPosture,
}

impl UiMountedCanonicalBoxBasis {
    fn from_box(bounds: UiMountedCanonicalBox) -> Self {
        Self {
            x_bits: bounds.x().to_bits(),
            y_bits: bounds.y().to_bits(),
            width_bits: bounds.width().to_bits(),
            height_bits: bounds.height().to_bits(),
            coordinate_space: bounds.coordinate_space(),
            posture: bounds.posture(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiMountedTextForegroundReuseReceipt {
    mechanic: UiMountedTextForegroundReuseMechanic,
    demand: worth_ui_text::UiGlyphRasterDemandBatch,
    binding_pins: Box<[UiGlyphRasterPinRequest]>,
    basis: UiMountedTextForegroundPresentationBasis,
}

impl UiMountedTextForegroundReuseReceipt {
    pub(crate) fn from_prepared(
        command: UiMountedPaintCommandIdentity,
        mechanic: &UiMountedSemanticTextMechanic,
        demand: &worth_ui_text::UiGlyphRasterDemandBatch,
        binding_pins: &[UiGlyphRasterPinRequest],
        basis: UiMountedTextForegroundPresentationBasis,
    ) -> Self {
        Self {
            mechanic: UiMountedTextForegroundReuseMechanic::from_mechanic(command, mechanic),
            demand: demand.clone(),
            binding_pins: binding_pins.to_vec().into_boxed_slice(),
            basis,
        }
    }

    pub(crate) fn accepts_successor(
        &self,
        command: UiMountedPaintCommandIdentity,
        mechanic: &UiMountedSemanticTextMechanic,
        basis: UiMountedTextForegroundPresentationBasis,
    ) -> bool {
        self.basis.admits_successor(basis)
            && self
                .mechanic
                .accepts_successor(command, mechanic, basis.successor())
            && self.raster_demand_matches(mechanic)
    }

    fn raster_demand_matches(&self, mechanic: &UiMountedSemanticTextMechanic) -> bool {
        let Some(placement) = worth_ui_text::UiGlyphRasterPlacement::from_mounted_logical(
            mechanic.origin_x(),
            mechanic.origin_y(),
        ) else {
            return false;
        };
        self.demand.layout_identity() == mechanic.qualified_layout_identity()
            && self.demand.scale().text_scale_generation() == mechanic.qualified_layout_scale()
            && self.demand.placement() == placement
            && self.demand.lane() == worth_ui_host_contract::UiGlyphRasterLane::Ordinary
            && self
                .demand
                .records()
                .iter()
                .all(|record| record.attribution().layout() == self.demand.layout_identity())
    }

    pub(crate) fn raster_keys_cached(&self, cache: &worth_ui_text::UiGlyphRasterCache) -> bool {
        self.demand
            .records()
            .iter()
            .all(|record| cache.contains_key(record.key()))
    }

    pub(crate) fn pins_are_continuous(&self, binding_pins: &[UiGlyphRasterPinRequest]) -> bool {
        same_pin_set(&self.binding_pins, binding_pins)
    }

    pub(crate) fn demand(&self) -> &worth_ui_text::UiGlyphRasterDemandBatch {
        &self.demand
    }

    pub(crate) fn basis(&self) -> UiMountedTextForegroundPresentationBasis {
        self.basis
    }

    pub(crate) const fn command(&self) -> UiMountedPaintCommandIdentity {
        self.mechanic.command
    }
}

fn same_foreground_shape(
    left: &[UiMountedTextForegroundSpan],
    right: &[UiMountedTextForegroundSpan],
) -> bool {
    left.len() == right.len()
        && left.iter().zip(right).all(|(left, right)| {
            left.original_range() == right.original_range() && left.identity() == right.identity()
        })
}

fn same_pin_set(left: &[UiGlyphRasterPinRequest], right: &[UiGlyphRasterPinRequest]) -> bool {
    left.len() == right.len() && left.iter().all(|pin| right.contains(pin))
}
