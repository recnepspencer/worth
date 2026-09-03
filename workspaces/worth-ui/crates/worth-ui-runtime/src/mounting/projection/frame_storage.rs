use worth_ui_host_contract::{
    UiMountedLayerRow, UiMountedPaintBatchRow, UiMountedPaintPrimitiveKind,
    UiMountedRealtimeBatchRow, UiMountedResourceEntry, UiMountedSpatialBatchRow,
};

use super::UiMountedProjectionDenial;

pub(in crate::mounting) mod diagnostic_source;
mod drawable_order;
mod lane_recording;
mod layout_reconstruction;
mod mechanic_source;
#[cfg(test)]
pub(crate) mod mechanic_source_tests;
mod node_changes;
mod portal_child_view;
mod portal_mechanic_view;
mod portal_overlay_view;
mod presentation_effects;
pub(crate) mod presentation_sources;
mod presentation_view;
mod rebind;
mod semantic_mechanics;
mod semantic_projection;
mod semantic_text_view;
mod table_recording;
mod view;

use diagnostic_source::UiMountedDiagnosticSource;
use lane_recording::require_once;
use mechanic_source::{UiMountedMechanicCompletion, UiMountedMechanicSource};
use presentation_effects::{
    UiMountedPresentationEffectCompletion, UiMountedPresentationEffectSource,
};
pub(in crate::mounting) use semantic_mechanics::UiMountedSemanticMechanicSource;
pub(in crate::mounting) use semantic_projection::UiMountedSemanticProjection;
pub(super) use semantic_projection::{UiMountedProjectionNodeRecord, UiMountedProjectionSurface};
use view::{UiMountedOrdinaryPaintSelector, UiMountedPlanIndexPaintSelector};

pub(crate) struct UiMountedAppearanceNodeInputContext {
    pub(crate) frame: worth_ui_host_contract::UiMountedFrameIdentity,
    pub(crate) semantic_surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    pub(crate) mounted_instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    pub(crate) graph_node: crate::graph::UiGraphNodeIdentity,
    pub(crate) incarnation: worth_ui_host_contract::UiMountIncarnation,
    pub(crate) node_receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
    issuer: worth_ui_host_contract::UiMountedNodeReceiptIssuer,
    plan_digest: u64,
    allocation: worth_ui_host_contract::UiMountedAllocationProjection,
    static_paint: Option<super::static_paint::UiMountedStaticPaintSeed>,
}

impl UiMountedAppearanceNodeInputContext {
    pub(crate) fn lower(
        &self,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
    ) -> Result<super::UiMountedAppearanceLoweringInput, super::UiMountedAppearanceLoweringDenial>
    {
        let input_node = super::appearance::UiMountedAppearanceNodeInput::from_runtime_mounting(
            self.issuer,
            self.semantic_surface,
            self.node_receipt,
            self.graph_node,
            self.plan_digest,
            self.allocation,
            self.static_paint,
        )?;
        Ok(super::UiMountedAppearanceLoweringInput::for_single_node(
            self.frame,
            self.semantic_surface,
            presentation,
            input_node,
        ))
    }
}

const TABLE_LIMIT: usize = 2_048;
const RESOURCE_LIMIT: usize = 1_024;

#[derive(Clone)]
pub struct UiMountedProjectionFrame {
    frame: worth_ui_host_contract::UiMountedFrameIdentity,
    content_generation: worth_ui_host_contract::UiMountedContentGeneration,
    receipt_basis: super::super::UiMountedNodeReceiptBasis,
    plan_digest: u64,
    semantic: UiMountedSemanticProjection,
    mechanics: UiMountedMechanicSource,
    presentation_effects: UiMountedPresentationEffectSource,
    diagnostics: UiMountedDiagnosticSource,
    changed_instances: std::rc::Rc<[worth_ui_host_contract::UiMountedInstanceIdentity]>,
    precise_command_instances: std::rc::Rc<[worth_ui_host_contract::UiMountedInstanceIdentity]>,
    presentation_command_changes:
        std::rc::Rc<[worth_ui_host_contract::UiMountedPaintCommandChange]>,
    paint_batches: Vec<UiMountedPaintBatchRow>,
    layers: Vec<UiMountedLayerRow>,
    spatial_batches: Vec<UiMountedSpatialBatchRow>,
    realtime_batches: Vec<UiMountedRealtimeBatchRow>,
    resources: Vec<UiMountedResourceEntry>,
    ordinary_recorded: bool,
    virtualized_recorded: bool,
    canvas_recorded: bool,
    realtime_recorded: bool,
    ordinary_paint_selector: Option<UiMountedOrdinaryPaintSelector>,
    plan_index_paint_selectors: Vec<UiMountedPlanIndexPaintSelector>,
    preview: Option<super::lowering::UiMountedPreviewProjectionInput>,
    visual_overlay: Option<super::super::UiMountedVisualOverlayProjectionInput>,
    portal_overlays: std::rc::Rc<[super::super::UiMountedPortalOverlayProjectionInput]>,
    portal_overlays_changed: bool,
    counters: super::super::UiMountStageCounters,
    capability_generation: worth_ui_host_contract::WorthUiHostCapabilityObservationGeneration,
    capability_profile_digest: u64,
    font_collection: std::sync::Arc<worth_ui_text::UiGlobalFontCollection>,
    text_profile_generation: worth_ui_host_contract::UiTextProfileGeneration,
    materialized_projection_rows: std::rc::Rc<std::cell::Cell<u64>>,
}

pub(super) struct UiMountedProjectionFrameInput {
    pub frame: worth_ui_host_contract::UiMountedFrameIdentity,
    pub content_generation: worth_ui_host_contract::UiMountedContentGeneration,
    pub receipt_basis: super::super::UiMountedNodeReceiptBasis,
    pub plan_digest: u64,
    pub semantic: UiMountedSemanticProjection,
    pub counters: super::super::UiMountStageCounters,
    pub capability_generation: worth_ui_host_contract::WorthUiHostCapabilityObservationGeneration,
    pub capability_profile_digest: u64,
    pub font_collection: std::sync::Arc<worth_ui_text::UiGlobalFontCollection>,
    pub mechanics: UiMountedMechanicSource,
    pub presentation_effects: UiMountedPresentationEffectSource,
    pub diagnostics: UiMountedDiagnosticSource,
    pub portal_overlays: std::rc::Rc<[super::super::UiMountedPortalOverlayProjectionInput]>,
    pub portal_overlays_changed: bool,
    pub changed_instances: std::rc::Rc<[worth_ui_host_contract::UiMountedInstanceIdentity]>,
}

impl UiMountedProjectionFrame {
    pub(super) fn new(input: UiMountedProjectionFrameInput) -> Self {
        Self {
            frame: input.frame,
            content_generation: input.content_generation,
            receipt_basis: input.receipt_basis,
            plan_digest: input.plan_digest,
            semantic: input.semantic,
            mechanics: input.mechanics,
            presentation_effects: input.presentation_effects,
            diagnostics: input.diagnostics,
            changed_instances: input.changed_instances,
            precise_command_instances: std::rc::Rc::from([]),
            presentation_command_changes: std::rc::Rc::from([]),
            paint_batches: Vec::new(),
            layers: Vec::new(),
            spatial_batches: Vec::new(),
            realtime_batches: Vec::new(),
            resources: Vec::new(),
            ordinary_recorded: false,
            virtualized_recorded: false,
            canvas_recorded: false,
            realtime_recorded: false,
            ordinary_paint_selector: None,
            plan_index_paint_selectors: Vec::new(),
            preview: None,
            visual_overlay: None,
            portal_overlays: input.portal_overlays,
            portal_overlays_changed: input.portal_overlays_changed,
            counters: input.counters,
            capability_generation: input.capability_generation,
            capability_profile_digest: input.capability_profile_digest,
            font_collection: input.font_collection,
            text_profile_generation: super::semantic_text::current_text_profile_generation(),
            materialized_projection_rows: std::rc::Rc::new(std::cell::Cell::new(0)),
        }
    }

    pub fn frame_identity(&self) -> worth_ui_host_contract::UiMountedFrameIdentity {
        self.frame
    }
    pub(in crate::mounting) fn content_generation(
        &self,
    ) -> worth_ui_host_contract::UiMountedContentGeneration {
        self.content_generation
    }
    pub fn plan_digest(&self) -> u64 {
        self.plan_digest
    }
    pub(super) fn visual_overlay_target(
        &self,
    ) -> Option<worth_ui_host_contract::UiMountedInstanceIdentity> {
        self.visual_overlay.map(|overlay| overlay.target_instance())
    }
    pub(super) fn portal_overlay_inputs(
        &self,
    ) -> &[super::super::UiMountedPortalOverlayProjectionInput] {
        &self.portal_overlays
    }
    pub(crate) fn mounted_instances(
        &self,
    ) -> impl ExactSizeIterator<Item = worth_ui_host_contract::UiMountedInstanceIdentity> + '_ {
        self.semantic.order.iter().copied()
    }

    pub fn cost_report(&self) -> super::super::UiMountCostReport {
        self.counters.finish()
    }

    pub(in crate::mounting) fn table_range_digest(&self) -> u64 {
        [
            self.semantic.node_count() as u64,
            self.layers.len() as u64,
            self.paint_batches.len() as u64,
            self.spatial_batches.len() as u64,
            self.realtime_batches.len() as u64,
            self.resources.len() as u64,
            self.mechanics.table_digest(),
        ]
        .into_iter()
        .fold(0x7461_626c_6572_616e_u64, |digest, value| {
            digest.rotate_left(11) ^ value
        })
    }

    pub(in crate::mounting) fn identity_trace_basis(
        &self,
        source: crate::facade::prepared_application_authority::WorthUiPreparedVisualTraceSource,
    ) -> super::super::UiMountedIdentityTraceBasis {
        super::super::UiMountedIdentityTraceBasis::new(
            self.receipt_basis.clone(),
            self.semantic.clone(),
            source,
        )
    }

    pub(super) fn record_ordinary(
        &mut self,
        receipt: &crate::runtime::WorthUiOrdinaryLaneFrameReceipt,
    ) -> Result<(), UiMountedProjectionDenial> {
        require_once(&mut self.ordinary_recorded)?;
        let batch = self.push_lane_batch(
            receipt.touch().row_count() as u32,
            0,
            None,
            UiMountedPaintPrimitiveKind::OrdinaryLaneSummary,
        )?;
        self.ordinary_paint_selector =
            Some(UiMountedOrdinaryPaintSelector::new(receipt.clone(), batch));
        Ok(())
    }

    pub(super) fn complete_mechanics(&mut self) -> Result<(), UiMountedProjectionDenial> {
        let mutation = self.mechanics.apply(UiMountedMechanicCompletion {
            frame: self.frame,
            content: self.content_generation,
            receipts: &self.receipt_basis,
            semantic: &self.semantic,
            changed: &self.changed_instances,
            capability_generation: self.capability_generation,
            capability_profile_digest: self.capability_profile_digest,
            font_collection: &self.font_collection,
        })?;
        self.record_rows::<worth_ui_host_contract::UiMountedFilledRectMechanic>(
            mutation.filled_rects,
        )?;
        self.record_rows::<worth_ui_host_contract::UiMountedSemanticTextMechanic>(
            mutation.semantic_text,
        )?;
        self.record_rows::<worth_ui_host_contract::UiMountedHitTestMechanic>(mutation.hit_tests)?;
        self.precise_command_instances = mutation.precise_instances.into();
        self.presentation_command_changes = mutation.command_changes.into();
        Ok(())
    }

    pub(super) fn mechanic_source(&self) -> UiMountedMechanicSource {
        self.mechanics.clone()
    }

    pub(in crate::mounting) fn input_text_profile(
        &self,
    ) -> worth_ui_host_contract::UiTextProfileGeneration {
        self.text_profile_generation
    }

    pub(super) fn presentation_effect_source(&self) -> UiMountedPresentationEffectSource {
        self.presentation_effects.clone()
    }

    pub(super) fn complete_presentation_effects(
        &mut self,
        changed_instances: &[worth_ui_host_contract::UiMountedInstanceIdentity],
    ) {
        self.presentation_effects
            .apply(UiMountedPresentationEffectCompletion {
                semantic: &self.semantic,
                mechanics: &self.mechanics,
                changed: changed_instances,
                preview: self.preview.as_ref(),
                overlay: self.visual_overlay.as_ref(),
                canvas: !self.spatial_batches.is_empty(),
                realtime: !self.realtime_batches.is_empty(),
            });
    }

    pub(super) fn complete_diagnostics(
        &mut self,
        changed_instances: &[worth_ui_host_contract::UiMountedInstanceIdentity],
    ) {
        self.diagnostics.apply(
            &self.semantic,
            changed_instances,
            self.visual_overlay,
            self.frame,
        );
    }

    pub(in crate::mounting) fn materialized_projection_rows(&self) -> u64 {
        self.materialized_projection_rows.get()
    }

    pub(in crate::mounting) fn node_receipt_affinity(
        &self,
    ) -> Option<worth_ui_host_contract::UiMountedNodeReceiptAffinity> {
        self.receipt_basis.affinity()
    }

    pub(in crate::mounting) fn refresh_presentation_lane_auxiliary(
        &self,
        auxiliary: &mut worth_ui_host_contract::UiMountedPresentationAuxiliaryState,
        requirement: worth_ui_host_contract::UiMountedSurfaceBindingRequirement,
    ) {
        auxiliary.refresh_lane_presentation_from_runtime_mounting(
            self.frame,
            requirement.semantic_surface(),
            requirement.binding(),
            self.content_generation,
            worth_ui_host_contract::UiMountedPaintBatchTable::new(self.paint_batches.clone()),
            worth_ui_host_contract::UiMountedSpatialBatchTable::new(self.spatial_batches.clone()),
            worth_ui_host_contract::UiMountedRealtimeBatchTable::new(self.realtime_batches.clone()),
            worth_ui_host_contract::UiMountedResourceTable::new(self.resources.clone()),
        );
    }

    pub(in crate::mounting) fn semantic_projection(&self) -> &UiMountedSemanticProjection {
        &self.semantic
    }

    pub(crate) fn appearance_node_inputs(
        &self,
        selected_graph_nodes: &[crate::graph::UiGraphNodeIdentity],
    ) -> Vec<UiMountedAppearanceNodeInputContext> {
        self.semantic
            .nodes_in_mounted_order()
            .filter(|node| selected_graph_nodes.contains(&node.receipt.graph_node()))
            .map(|node| {
                let receipt = node.receipt();
                let node_receipt = self
                    .receipt_basis
                    .receipt_for(receipt.mounted_instance())
                    .expect("mounted projection node belongs to its receipt basis");
                UiMountedAppearanceNodeInputContext {
                    frame: self.frame,
                    semantic_surface: receipt.semantic_surface(),
                    mounted_instance: receipt.mounted_instance(),
                    graph_node: receipt.graph_node(),
                    incarnation: receipt.incarnation(),
                    node_receipt,
                    issuer: self.receipt_basis.issuer(),
                    plan_digest: receipt.plan_digest(),
                    allocation: receipt.allocation(),
                    static_paint: node.static_paint,
                }
            })
            .collect()
    }
}
