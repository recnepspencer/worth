use worth_ui_host_contract::{
    UiMountedInstanceIdentity, UiMountedPaintCommand, UiMountedPaintCommandChange,
    UiSemanticTextSlot,
};

use super::{
    UiMountedSemanticMechanicSource, UiMountedSemanticMechanicUpdate, UiMountedSemanticTextSeed,
    UiMountedSemanticTextSeedTransition,
};
use crate::mounting::projection::{
    frame_storage::UiMountedProjectionNodeRecord,
    semantic_text::{UiMountedSemanticTextCompletionContext, UiMountedSemanticTextRepaintInput},
};
use crate::mounting::UiMountedProjectionDenial;

impl UiMountedSemanticMechanicSource {
    pub(in crate::mounting::projection::frame_storage) fn apply_paint_only(
        &mut self,
        context: &UiMountedSemanticTextCompletionContext<'_>,
        node: &UiMountedProjectionNodeRecord,
        seed: &UiMountedSemanticTextSeed,
    ) -> Option<Result<UiMountedSemanticMechanicUpdate, UiMountedProjectionDenial>> {
        if !matches!(
            seed.transition(),
            UiMountedSemanticTextSeedTransition::PaintOnly
        ) {
            return None;
        }
        if !self.can_repaint_instance(context, node) {
            return None;
        }
        Some(self.repaint_instance(context, node.receipt.mounted_instance(), seed))
    }

    fn can_repaint_instance(
        &self,
        context: &UiMountedSemanticTextCompletionContext<'_>,
        node: &UiMountedProjectionNodeRecord,
    ) -> bool {
        let Some(rows) = self.by_instance.get(&node.receipt.mounted_instance()) else {
            return false;
        };
        let Some(surface) = context
            .semantic
            .surface_for(node.receipt.semantic_surface())
        else {
            return false;
        };
        let worth_ui_host_contract::UiMountedAllocationProjection::Known { bounds, basis } =
            node.presentation_allocation()
        else {
            return false;
        };
        rows.iter().all(|row| {
            row.surface() == surface.surface
                && row.binding() == surface.binding
                && row.bounds() == bounds
                && row.clip_bounds() == bounds
                && row.allocation_basis() == basis
                && row.font_collection_matches(context.font_collection)
        })
    }

    fn repaint_instance(
        &mut self,
        context: &UiMountedSemanticTextCompletionContext<'_>,
        instance: UiMountedInstanceIdentity,
        seed: &UiMountedSemanticTextSeed,
    ) -> Result<UiMountedSemanticMechanicUpdate, UiMountedProjectionDenial> {
        let node_receipt = context
            .receipt_basis
            .receipt_for(instance)
            .ok_or(UiMountedProjectionDenial::SemanticTextNodeReceiptMismatch)?;
        let mut rows = self
            .by_instance
            .get(&instance)
            .cloned()
            .ok_or(UiMountedProjectionDenial::DrawableSourceCoverageMismatch)?;
        let keys = rows.order.iter().copied().collect::<Vec<_>>();
        let mut command_changes = Vec::with_capacity(keys.len());
        for key in keys {
            let predecessor = rows
                .rows
                .get(&key)
                .cloned()
                .ok_or(UiMountedProjectionDenial::DrawableSourceCoverageMismatch)?;
            let formatting = if predecessor.slot() == UiSemanticTextSlot::Value {
                seed.formatting().scalar_value_row()
            } else {
                seed.formatting().default_row()
            };
            let foregrounds = formatting.materialize_foregrounds(predecessor.text())?;
            let replacement = predecessor.repaint(UiMountedSemanticTextRepaintInput {
                content_generation: context.content_generation,
                frame: context.frame,
                node_receipt,
                capability_generation: context.capability_generation,
                capability_profile_digest: context.capability_profile_digest,
                foregrounds,
            })?;
            rows.replace(key, replacement.clone())?;
            command_changes.push(UiMountedPaintCommandChange::replacement(
                worth_ui_host_contract::UiMountedPaintCommandIdentity::semantic_text(
                    predecessor.mechanic(),
                ),
                UiMountedPaintCommand::SemanticText {
                    identity: worth_ui_host_contract::UiMountedPaintCommandIdentity::semantic_text(
                        &replacement,
                    ),
                    mechanic: replacement.mechanic_clone(),
                },
            ));
        }
        let predecessor_digest = self
            .by_instance
            .get(&instance)
            .expect("paint-only update has predecessor rows")
            .digest;
        self.digest ^= predecessor_digest ^ rows.digest;
        self.by_instance.insert(instance, rows);
        Ok(UiMountedSemanticMechanicUpdate {
            rows_materialized: command_changes.len(),
            command_changes,
        })
    }
}
