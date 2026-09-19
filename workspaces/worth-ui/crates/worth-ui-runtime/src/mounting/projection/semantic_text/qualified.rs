use std::{ops::Deref, sync::Arc};

use worth_ui_host_contract::{UiMountedSemanticTextCompletionInput, UiMountedSemanticTextMechanic};

use super::super::UiMountedProjectionDenial;

mod frame_affinity;
#[cfg(test)]
mod tests;

#[derive(Clone)]
pub(in crate::mounting::projection) struct UiMountedQualifiedSemanticText {
    mechanic: UiMountedSemanticTextMechanic,
    layout: UiMountedQualifiedLayoutState,
}

#[derive(Clone)]
enum UiMountedQualifiedLayoutState {
    Current(Arc<worth_ui_text::UiQualifiedTextLayout>),
    #[cfg(any(test, feature = "certification-support"))]
    ReconstructionRequired(Arc<worth_ui_text::UiQualifiedTextReconstructionSource>),
}

pub(in crate::mounting::projection) struct UiMountedSemanticTextRepaintInput {
    pub(in crate::mounting::projection) content_generation:
        worth_ui_host_contract::UiMountedContentGeneration,
    pub(in crate::mounting::projection) frame: worth_ui_host_contract::UiMountedFrameIdentity,
    pub(in crate::mounting::projection) node_receipt:
        worth_ui_host_contract::UiMountedNodeReceiptIdentity,
    pub(in crate::mounting::projection) capability_generation:
        worth_ui_host_contract::WorthUiHostCapabilityObservationGeneration,
    pub(in crate::mounting::projection) capability_profile_digest: u64,
    pub(in crate::mounting::projection) foregrounds:
        Arc<[worth_ui_host_contract::UiMountedTextForegroundSpan]>,
}

impl UiMountedQualifiedSemanticText {
    pub(super) fn new(
        mechanic: UiMountedSemanticTextMechanic,
        layout: Arc<worth_ui_text::UiQualifiedTextLayout>,
    ) -> Self {
        debug_assert_eq!(mechanic.qualified_layout_identity(), layout.identity());
        Self {
            mechanic,
            layout: UiMountedQualifiedLayoutState::Current(layout),
        }
    }

    pub(in crate::mounting::projection) const fn mechanic(&self) -> &UiMountedSemanticTextMechanic {
        &self.mechanic
    }

    pub(in crate::mounting::projection) fn mechanic_clone(&self) -> UiMountedSemanticTextMechanic {
        self.mechanic.clone()
    }

    pub(in crate::mounting::projection) fn qualified_layout(
        &self,
    ) -> Option<&Arc<worth_ui_text::UiQualifiedTextLayout>> {
        match &self.layout {
            UiMountedQualifiedLayoutState::Current(layout) => Some(layout),
            #[cfg(any(test, feature = "certification-support"))]
            UiMountedQualifiedLayoutState::ReconstructionRequired(_) => None,
        }
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(in crate::mounting::projection) const fn layout_reconstruction_required(&self) -> bool {
        matches!(
            self.layout,
            UiMountedQualifiedLayoutState::ReconstructionRequired(_)
        )
    }

    #[cfg(not(any(test, feature = "certification-support")))]
    pub(in crate::mounting::projection) const fn layout_reconstruction_required(&self) -> bool {
        false
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(in crate::mounting::projection) fn require_layout_reconstruction(
        &mut self,
    ) -> Result<bool, UiMountedProjectionDenial> {
        let UiMountedQualifiedLayoutState::Current(layout) = &self.layout else {
            return Ok(false);
        };
        let source = layout
            .reconstruction_source()
            .cloned()
            .ok_or(UiMountedProjectionDenial::MissingSemanticTextReconstructionSource)?;
        self.layout = UiMountedQualifiedLayoutState::ReconstructionRequired(source);
        Ok(true)
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(in crate::mounting::projection) fn reconstruct_layout(
        &mut self,
    ) -> Result<bool, UiMountedProjectionDenial> {
        let UiMountedQualifiedLayoutState::ReconstructionRequired(source) = &self.layout else {
            return Ok(false);
        };
        let layout = source
            .reconstruct()
            .map_err(UiMountedProjectionDenial::SemanticTextReconstruction)?;
        if layout.identity() != self.mechanic.qualified_layout_identity()
            || layout.source() != self.mechanic.text()
        {
            return Err(UiMountedProjectionDenial::MissingSemanticTextReconstructionSource);
        }
        self.layout = UiMountedQualifiedLayoutState::Current(Arc::new(layout));
        Ok(true)
    }

    #[cfg(not(any(test, feature = "certification-support")))]
    pub(in crate::mounting::projection) const fn reconstruct_layout(
        &mut self,
    ) -> Result<bool, UiMountedProjectionDenial> {
        Ok(false)
    }

    pub(in crate::mounting::projection) fn font_collection_matches(
        &self,
        candidate: &Arc<worth_ui_text::UiGlobalFontCollection>,
    ) -> bool {
        match &self.layout {
            UiMountedQualifiedLayoutState::Current(layout) => {
                Arc::ptr_eq(layout.pinned_font_collection(), candidate)
            }
            #[cfg(any(test, feature = "certification-support"))]
            UiMountedQualifiedLayoutState::ReconstructionRequired(source) => {
                source.matches_font_collection(candidate)
            }
        }
    }

    pub(super) fn rebind(
        &self,
        replacement: super::super::super::UiSurfaceBindingIdentityView,
        allocation_basis: worth_ui_host_contract::UiMountedAllocationBasis,
    ) -> Result<Self, UiMountedProjectionDenial> {
        if replacement.semantic_surface_identity() != self.surface() {
            return Err(UiMountedProjectionDenial::MissingSurfaceBinding);
        }
        let layout = self
            .qualified_layout()
            .ok_or(UiMountedProjectionDenial::MissingSemanticTextReconstructionSource)?;
        let mechanic =
            UiMountedSemanticTextMechanic::complete_from_runtime_mounting_with_reused_layout(
                UiMountedSemanticTextCompletionInput {
                    content_generation: self.content_generation(),
                    frame: self.frame(),
                    surface: self.surface(),
                    binding: replacement.binding_generation(),
                    mounted_instance: self.mounted_instance(),
                    portal_group: self.portal_group(),
                    node_receipt: self.node_receipt(),
                    allocation_basis,
                    bounds: self.bounds(),
                    clip_bounds: self.clip_bounds(),
                    origin_x: self.origin_x(),
                    origin_y: self.origin_y(),
                    text: Arc::from(self.text()),
                    layout: layout.view(),
                    slot: self.slot(),
                    collection_row: self.collection_row().cloned(),
                    foregrounds: Arc::from(self.foregrounds()),
                    profile: self.profile(),
                    layer_semantic_order: self.layer_semantic_order(),
                    capability_generation: self.capability_generation(),
                    capability_profile_digest: self.capability_profile_digest(),
                },
            )
            .map_err(UiMountedProjectionDenial::SemanticTextCompletion)?;
        Ok(Self::new(mechanic, Arc::clone(layout)))
    }

    pub(in crate::mounting::projection) fn repaint(
        &self,
        input: UiMountedSemanticTextRepaintInput,
    ) -> Result<Self, UiMountedProjectionDenial> {
        let layout = self
            .qualified_layout()
            .ok_or(UiMountedProjectionDenial::MissingSemanticTextReconstructionSource)?;
        let mechanic =
            UiMountedSemanticTextMechanic::complete_from_runtime_mounting_with_reused_layout(
                UiMountedSemanticTextCompletionInput {
                    content_generation: input.content_generation,
                    frame: input.frame,
                    surface: self.surface(),
                    binding: self.binding(),
                    mounted_instance: self.mounted_instance(),
                    portal_group: self.portal_group(),
                    node_receipt: input.node_receipt,
                    allocation_basis: self.allocation_basis(),
                    bounds: self.bounds(),
                    clip_bounds: self.clip_bounds(),
                    origin_x: self.origin_x(),
                    origin_y: self.origin_y(),
                    text: Arc::from(self.text()),
                    layout: layout.view(),
                    slot: self.slot(),
                    collection_row: self.collection_row().cloned(),
                    foregrounds: input.foregrounds,
                    profile: self.profile(),
                    layer_semantic_order: self.layer_semantic_order(),
                    capability_generation: input.capability_generation,
                    capability_profile_digest: input.capability_profile_digest,
                },
            )
            .map_err(UiMountedProjectionDenial::SemanticTextCompletion)?;
        Ok(Self::new(mechanic, Arc::clone(layout)))
    }
}

impl Deref for UiMountedQualifiedSemanticText {
    type Target = UiMountedSemanticTextMechanic;

    fn deref(&self) -> &Self::Target {
        &self.mechanic
    }
}

pub(in crate::mounting::projection) fn rebind_semantic_text(
    rows: &mut [UiMountedQualifiedSemanticText],
    replacements: &[(
        worth_ui_host_contract::UiSurfaceBindingGeneration,
        super::super::super::UiSurfaceBindingIdentityView,
    )],
    semantic: &super::super::frame_storage::UiMountedSemanticProjection,
) -> Result<(), UiMountedProjectionDenial> {
    for row in rows {
        let Some((_, replacement)) = replacements
            .iter()
            .find(|(affected, _)| *affected == row.binding())
        else {
            continue;
        };
        let node = semantic
            .node(row.mounted_instance())
            .ok_or(UiMountedProjectionDenial::UnknownGraphNode)?;
        let allocation_basis = match node.presentation_allocation() {
            worth_ui_host_contract::UiMountedAllocationProjection::Known { basis, .. }
            | worth_ui_host_contract::UiMountedAllocationProjection::PortalAnchorObservation {
                basis,
                ..
            } => basis,
            worth_ui_host_contract::UiMountedAllocationProjection::Omitted(_) => {
                return Err(UiMountedProjectionDenial::MissingSemanticTextAllocation(
                    node.receipt().graph_node(),
                ));
            }
        };
        // Text retains its concrete layout allocation lineage. The mounted
        // owner supplies only the replacement surface's coordinate authority;
        // its regional allocation is not the text layout allocation.
        let text_basis = row.allocation_basis();
        let rebound_basis = worth_ui_host_contract::UiMountedAllocationBasis::new(
            text_basis.receipt_identity(),
            text_basis.receipt_generation(),
            allocation_basis.coordinate_ownership(),
            text_basis.transform(),
        );
        *row = row.rebind(*replacement, rebound_basis)?;
    }
    Ok(())
}
