use worth_ui_host_contract::{UiMountedSemanticTextCompletionInput, UiMountedSemanticTextMechanic};

use super::UiMountedQualifiedSemanticText;
use crate::mounting::UiMountedProjectionDenial;

impl UiMountedQualifiedSemanticText {
    pub(in crate::mounting::projection) fn reattributed_mechanic(
        &self,
        content_generation: worth_ui_host_contract::UiMountedContentGeneration,
        frame: worth_ui_host_contract::UiMountedFrameIdentity,
        node_receipt: worth_ui_host_contract::UiMountedNodeReceiptIdentity,
    ) -> Result<UiMountedSemanticTextMechanic, UiMountedProjectionDenial> {
        let layout = self
            .qualified_layout()
            .ok_or(UiMountedProjectionDenial::MissingSemanticTextReconstructionSource)?;
        UiMountedSemanticTextMechanic::complete_from_runtime_mounting_with_reused_layout(
            UiMountedSemanticTextCompletionInput {
                content_generation,
                frame,
                surface: self.surface(),
                binding: self.binding(),
                mounted_instance: self.mounted_instance(),
                portal_group: self.portal_group(),
                node_receipt,
                allocation_basis: self.allocation_basis(),
                bounds: self.bounds(),
                clip_bounds: self.clip_bounds(),
                origin_x: self.origin_x(),
                origin_y: self.origin_y(),
                text: self.retained_text_for_runtime_mounting(),
                layout: layout.view(),
                slot: self.slot(),
                collection_row: self.collection_row().cloned(),
                foregrounds: self.retained_foregrounds_for_runtime_mounting(),
                profile: self.profile(),
                layer_semantic_order: self.layer_semantic_order(),
                capability_generation: self.capability_generation(),
                capability_profile_digest: self.capability_profile_digest(),
            },
        )
        .map_err(UiMountedProjectionDenial::SemanticTextCompletion)
    }
}
