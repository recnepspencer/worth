use super::frame_storage::{UiMountedProjectionFrameInput, UiMountedSemanticProjection};
use super::{UiMountedProjectionDenial, UiMountedProjectionFrame};

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
    pub(super) portal_overlays_changed: bool,
    pub(super) counters: super::super::UiMountStageCounters,
    pub(super) capability_generation:
        worth_ui_host_contract::WorthUiHostCapabilityObservationGeneration,
    pub(super) capability_profile_digest: u64,
    pub(super) font_collection: std::sync::Arc<worth_ui_text::UiGlobalFontCollection>,
}

#[derive(Clone)]
pub struct UiProjectedMountedFrameCandidate {
    pub(in crate::mounting) frame: std::rc::Rc<UiMountedProjectionFrame>,
    pub(in crate::mounting) identity_candidate:
        super::super::identity_state::UiMountedIdentityFrameCandidate,
    pub(in crate::mounting) projection_changes: super::super::UiMountedProjectionChangeSnapshot,
    pub(in crate::mounting) presentation_predecessor:
        Option<worth_ui_host_contract::UiMountedFrameIdentity>,
    pub(in crate::mounting) presentation_changed_instances:
        std::rc::Rc<[worth_ui_host_contract::UiMountedInstanceIdentity]>,
    pub(in crate::mounting) presentation_node_changed_instances:
        std::rc::Rc<[worth_ui_host_contract::UiMountedInstanceIdentity]>,
}

#[derive(Clone, Copy)]
pub(crate) struct UiMountedPresentationDeltaSource<'a> {
    frame: &'a UiMountedProjectionFrame,
    predecessor: Option<worth_ui_host_contract::UiMountedFrameIdentity>,
    changed_instances: &'a [worth_ui_host_contract::UiMountedInstanceIdentity],
    node_changed_instances: &'a [worth_ui_host_contract::UiMountedInstanceIdentity],
    changes: &'a super::super::UiMountedProjectionChangeSnapshot,
}

impl UiMountedPresentationDeltaSource<'_> {
    pub(in crate::mounting) const fn frame(&self) -> &UiMountedProjectionFrame {
        self.frame
    }
    pub(in crate::mounting) const fn predecessor(
        &self,
    ) -> Option<worth_ui_host_contract::UiMountedFrameIdentity> {
        self.predecessor
    }

    pub(in crate::mounting) const fn changed_instances(
        &self,
    ) -> &[worth_ui_host_contract::UiMountedInstanceIdentity] {
        self.changed_instances
    }

    pub(in crate::mounting) const fn node_changed_instances(
        &self,
    ) -> &[worth_ui_host_contract::UiMountedInstanceIdentity] {
        self.node_changed_instances
    }

    pub(in crate::mounting) fn surface_changed(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> bool {
        self.changes.affects_surface(surface)
    }
}

impl UiProjectedMountedFrameCandidate {
    pub(in crate::mounting) fn stage_appearance_projection(
        &mut self,
        attempt: crate::runtime::appearance::UiAppearanceProjectionAttempt,
    ) -> Result<(), super::frame_storage::UiAppearanceStateCapacityExceeded> {
        std::rc::Rc::make_mut(&mut self.frame).stage_appearance_projection(attempt)
    }

    pub(in crate::mounting) fn set_appearance_invalidation_batch(
        &mut self,
        batch: crate::runtime::appearance::UiAppearanceInvalidationBatch,
    ) {
        std::rc::Rc::make_mut(&mut self.frame).set_appearance_invalidation_batch(batch);
    }

    pub(in crate::mounting) fn appearance_invalidation_batch(
        &self,
    ) -> Option<crate::runtime::appearance::UiAppearanceInvalidationBatch> {
        self.frame.appearance_invalidation_batch()
    }

    pub(in crate::mounting) fn prune_appearance_state(
        &mut self,
        session: crate::facade::WorthUiActiveApplicationSessionIdentity,
        generation: &crate::runtime::WorthUiActiveApplicationGenerationIdentity,
    ) {
        std::rc::Rc::make_mut(&mut self.frame).prune_appearance_state(session, generation);
    }

    pub(in crate::mounting) fn reserve_appearance_state(
        &mut self,
        context: &crate::runtime::appearance::UiAppearanceAttemptContext,
    ) -> Result<(), super::frame_storage::UiAppearanceStateCapacityExceeded> {
        std::rc::Rc::make_mut(&mut self.frame).reserve_appearance_state(context)
    }

    pub(in crate::mounting) fn lower_appearance(
        &mut self,
        presentation: worth_ui_host_contract::UiMountedPresentationAttemptIdentity,
    ) -> Vec<crate::runtime::appearance::UiAppearanceInspectionRecord> {
        std::rc::Rc::make_mut(&mut self.frame).lower_appearance(presentation)
    }

    pub(in crate::mounting) fn prepare_surface_reconstruction(
        &mut self,
        replacements: &[(
            worth_ui_host_contract::UiSurfaceBindingGeneration,
            crate::mounting::UiSurfaceBindingIdentityView,
        )],
    ) -> Result<(), UiMountedProjectionDenial> {
        let frame = std::rc::Rc::make_mut(&mut self.frame);
        frame.rebind_retained_mechanics(replacements)?;
        frame.prepare_appearance_reconstruction();
        self.presentation_changed_instances = frame.mounted_instances().collect::<Vec<_>>().into();
        self.presentation_node_changed_instances = self.presentation_changed_instances.clone();
        Ok(())
    }

    pub(in crate::mounting) fn appearance_state_capacity_error(
        &self,
    ) -> Option<super::frame_storage::UiAppearanceStateCapacityExceeded> {
        self.frame.appearance_state_capacity_error()
    }

    pub fn frame(&self) -> &UiMountedProjectionFrame {
        &self.frame
    }

    pub(crate) fn presented_receipt_basis(&self) -> &super::super::UiMountedNodeReceiptBasis {
        self.identity_candidate.receipt_basis()
    }

    pub(in crate::mounting) fn presentation_delta_source(
        &self,
    ) -> UiMountedPresentationDeltaSource<'_> {
        UiMountedPresentationDeltaSource {
            frame: &self.frame,
            predecessor: self.presentation_predecessor,
            changed_instances: &self.presentation_changed_instances,
            node_changed_instances: &self.presentation_node_changed_instances,
            changes: &self.projection_changes,
        }
    }

    pub(crate) fn into_parts(
        self,
    ) -> (
        std::rc::Rc<UiMountedProjectionFrame>,
        super::super::identity_state::UiMountedIdentityFrameCandidate,
        super::super::UiMountedProjectionChangeSnapshot,
    ) {
        (self.frame, self.identity_candidate, self.projection_changes)
    }
}

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
        let appearance_predecessor = state.current_projection();
        let predecessor = appearance_predecessor
            .filter(|projection| projection.plan_digest() == self.plan_digest);
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
        frame.inherit_appearance_state(appearance_predecessor);
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
        Ok(UiProjectedMountedFrameCandidate {
            frame: std::rc::Rc::new(frame),
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
