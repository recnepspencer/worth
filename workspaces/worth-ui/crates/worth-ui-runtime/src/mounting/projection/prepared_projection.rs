use super::frame_storage::{
    UiMountedAppearanceFrameState, UiMountedProjectionFrameInput, UiMountedProjectionFrameOwner,
    UiMountedSemanticProjection,
};
use super::{
    UiMountedAppearanceProjectionSelection, UiMountedProjectionDenial, UiMountedProjectionFrame,
};

#[path = "prepared_projection/delta_source.rs"]
mod delta_source;
pub(crate) use delta_source::UiMountedPresentationDeltaSource;

pub(crate) struct UiPreparedMountedProjection {
    plan_digest: u64,
    semantic: UiMountedSemanticProjection,
    ordinary: Option<crate::runtime::WorthUiOrdinaryLaneFrameReceipt>,
    virtualized: Option<crate::runtime::WorthUiVirtualizedDataFrameReceipt>,
    canvas: Option<(crate::runtime::WorthUiCanvasSpatialFrameReceipt, u64)>,
    realtime: Option<crate::runtime::WorthUiRealtimeFrameReceipt>,
    preview: Option<super::lowering::UiMountedPreviewProjectionInput>,
    visual_overlay: Option<super::super::UiMountedVisualOverlayProjectionInput>,
    portal_overlays: std::rc::Rc<[super::super::UiMountedPortalOverlayProjectionInput]>,
    projection_changes: super::super::UiMountedProjectionChangeSnapshot,
    presentation_changed_instances:
        std::rc::Rc<[worth_ui_host_contract::UiMountedInstanceIdentity]>,
    appearance_selection: std::rc::Rc<UiMountedAppearanceProjectionSelection>,
    appearance_invalidation: Option<crate::runtime::appearance::UiAppearanceInvalidationBatch>,
    theme_revision: Option<u64>,
    portal_overlays_changed: bool,
    counters: super::super::UiMountStageCounters,
    capability_generation: worth_ui_host_contract::WorthUiHostCapabilityObservationGeneration,
    capability_profile_digest: u64,
    font_collection: std::sync::Arc<worth_ui_text::UiGlobalFontCollection>,
}

pub(super) struct UiPreparedMountedProjectionInput {
    pub(super) plan_digest: u64,
    pub(super) semantic: UiMountedSemanticProjection,
    pub(super) preview: Option<super::lowering::UiMountedPreviewProjectionInput>,
    pub(super) visual_overlay: Option<super::super::UiMountedVisualOverlayProjectionInput>,
    pub(super) portal_overlays: std::rc::Rc<[super::super::UiMountedPortalOverlayProjectionInput]>,
    pub(super) projection_changes: super::super::UiMountedProjectionChangeSnapshot,
    pub(super) presentation_changed_instances:
        std::rc::Rc<[worth_ui_host_contract::UiMountedInstanceIdentity]>,
    pub(super) appearance_selection: std::rc::Rc<UiMountedAppearanceProjectionSelection>,
    pub(super) appearance_invalidation:
        Option<crate::runtime::appearance::UiAppearanceInvalidationBatch>,
    pub(super) theme_revision: Option<u64>,
    pub(super) portal_overlays_changed: bool,
    pub(super) counters: super::super::UiMountStageCounters,
    pub(super) capability_generation:
        worth_ui_host_contract::WorthUiHostCapabilityObservationGeneration,
    pub(super) capability_profile_digest: u64,
    pub(super) font_collection: std::sync::Arc<worth_ui_text::UiGlobalFontCollection>,
}

#[path = "prepared_projection/candidate.rs"]
mod candidate;
pub use candidate::UiProjectedMountedFrameCandidate;

impl UiPreparedMountedProjection {
    pub(super) fn new(input: UiPreparedMountedProjectionInput) -> Self {
        Self {
            plan_digest: input.plan_digest,
            semantic: input.semantic,
            ordinary: None,
            virtualized: None,
            canvas: None,
            realtime: None,
            preview: input.preview,
            visual_overlay: input.visual_overlay,
            portal_overlays: input.portal_overlays,
            projection_changes: input.projection_changes,
            presentation_changed_instances: input.presentation_changed_instances,
            appearance_selection: input.appearance_selection,
            appearance_invalidation: input.appearance_invalidation,
            theme_revision: input.theme_revision,
            portal_overlays_changed: input.portal_overlays_changed,
            counters: input.counters,
            capability_generation: input.capability_generation,
            capability_profile_digest: input.capability_profile_digest,
            font_collection: input.font_collection,
        }
    }

    pub(crate) fn record_ordinary(
        &mut self,
        receipt: &crate::runtime::WorthUiOrdinaryLaneFrameReceipt,
    ) -> Result<(), UiMountedProjectionDenial> {
        require_vacant(&self.ordinary)?;
        self.ordinary = Some(receipt.clone());
        Ok(())
    }

    pub(crate) fn record_virtualized(
        &mut self,
        receipt: &crate::runtime::WorthUiVirtualizedDataFrameReceipt,
    ) -> Result<(), UiMountedProjectionDenial> {
        require_vacant(&self.virtualized)?;
        self.virtualized = Some(receipt.clone());
        Ok(())
    }

    pub(crate) fn record_canvas(
        &mut self,
        receipt: &crate::runtime::WorthUiCanvasSpatialFrameReceipt,
        resource_content_identity: u64,
    ) -> Result<(), UiMountedProjectionDenial> {
        require_vacant(&self.canvas)?;
        self.canvas = Some((receipt.clone(), resource_content_identity));
        Ok(())
    }

    pub(crate) fn record_realtime(
        &mut self,
        receipt: &crate::runtime::WorthUiRealtimeFrameReceipt,
    ) -> Result<(), UiMountedProjectionDenial> {
        require_vacant(&self.realtime)?;
        self.realtime = Some(receipt.clone());
        Ok(())
    }

    pub(crate) fn finish(
        self,
        state: &super::super::UiMountedIdentityState,
        presentation_predecessor: Option<worth_ui_host_contract::UiMountedFrameIdentity>,
    ) -> Result<UiProjectedMountedFrameCandidate, UiMountedProjectionDenial> {
        self.validate_capacity()?;
        let identity_candidate = state.prepare_frame_candidate_for(self.semantic.membership())?;
        let content_generation = worth_ui_host_contract::UiMountedContentGeneration::mint_unbound()
            .map_err(|_| {
                UiMountedProjectionDenial::Identity(
                    super::super::UiMountedIdentityDenial::IdentityExhausted,
                )
            })?;
        let appearance_predecessor = state.current_projection_owner();
        let predecessor = appearance_predecessor
            .filter(|owner| owner.projection().plan_digest() == self.plan_digest)
            .map(UiMountedProjectionFrameOwner::projection);
        let mechanics = predecessor
            .map(UiMountedProjectionFrame::mechanic_source)
            .unwrap_or_default();
        let presentation_effects = predecessor
            .map(UiMountedProjectionFrame::presentation_effect_source)
            .unwrap_or_default();
        let diagnostics = predecessor
            .map(UiMountedProjectionFrame::diagnostic_source)
            .unwrap_or_default();
        let presentation_node_changed_instances = presentation_node_changes_with_overlay(
            &self.presentation_changed_instances,
            predecessor.and_then(UiMountedProjectionFrame::visual_overlay_target),
            self.visual_overlay.map(|overlay| overlay.target_instance()),
        );
        let mut frame = UiMountedProjectionFrame::new(UiMountedProjectionFrameInput {
            frame: identity_candidate.frame(),
            content_generation,
            receipt_basis: identity_candidate.receipt_basis().clone(),
            plan_digest: self.plan_digest,
            semantic: self.semantic,
            counters: self.counters,
            capability_generation: self.capability_generation,
            capability_profile_digest: self.capability_profile_digest,
            font_collection: self.font_collection,
            mechanics,
            presentation_effects,
            diagnostics,
            portal_overlays: self.portal_overlays,
            portal_overlays_changed: self.portal_overlays_changed,
            changed_instances: self.presentation_changed_instances.clone(),
        });
        frame.complete_mechanics()?;
        if let Some(receipt) = self.ordinary.as_ref() {
            frame.record_ordinary(receipt)?;
        }
        if let Some(receipt) = self.virtualized.as_ref() {
            frame.record_virtualized(receipt)?;
        }
        if let Some((receipt, resource)) = self.canvas.as_ref() {
            frame.record_canvas(receipt, *resource)?;
        }
        if let Some(receipt) = self.realtime.as_ref() {
            frame.record_realtime(receipt)?;
        }
        if let Some(preview) = self.preview {
            frame.record_preview(preview)?;
        }
        frame.record_visual_overlay(self.visual_overlay)?;
        frame.complete_presentation_effects(&presentation_node_changed_instances);
        frame.complete_diagnostics(&presentation_node_changed_instances);
        let appearance = UiMountedAppearanceFrameState::fork(
            state.appearance_predecessor(),
            self.appearance_selection,
        );
        let mut owner = UiMountedProjectionFrameOwner::new(
            std::rc::Rc::new(frame),
            appearance,
            self.theme_revision,
            state.pointer_predecessor().cloned().unwrap_or_default(),
        );
        if let Some(batch) = self.appearance_invalidation {
            owner.set_appearance_invalidation_batch(batch);
        }
        Ok(UiProjectedMountedFrameCandidate {
            owner,
            identity_candidate,
            projection_changes: self.projection_changes,
            presentation_predecessor,
            presentation_changed_instances: self.presentation_changed_instances,
            presentation_node_changed_instances,
        })
    }

    fn validate_capacity(&self) -> Result<(), UiMountedProjectionDenial> {
        let paint_rows = usize::from(self.ordinary.is_some())
            + usize::from(self.virtualized.is_some())
            + usize::from(self.canvas.is_some())
            + usize::from(self.realtime.is_some());
        if paint_rows > 2_048 {
            return Err(UiMountedProjectionDenial::TableCapacityExceeded);
        }
        Ok(())
    }
}

fn presentation_node_changes_with_overlay(
    changed: &[worth_ui_host_contract::UiMountedInstanceIdentity],
    predecessor_target: Option<worth_ui_host_contract::UiMountedInstanceIdentity>,
    successor_target: Option<worth_ui_host_contract::UiMountedInstanceIdentity>,
) -> std::rc::Rc<[worth_ui_host_contract::UiMountedInstanceIdentity]> {
    let mut changed = changed.to_vec();
    for target in [predecessor_target, successor_target].into_iter().flatten() {
        if !changed.contains(&target) {
            changed.push(target);
        }
    }
    changed.into()
}

fn require_vacant<T>(slot: &Option<T>) -> Result<(), UiMountedProjectionDenial> {
    slot.is_none()
        .then_some(())
        .ok_or(UiMountedProjectionDenial::DuplicateLaneContribution)
}
