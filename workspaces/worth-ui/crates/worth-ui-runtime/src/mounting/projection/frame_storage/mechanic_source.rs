use std::sync::Arc;

use worth_ui_host_contract::UiMountedInstanceIdentity;

use super::semantic_mechanics::UiMountedSemanticMechanicSource;
use super::UiMountedSemanticProjection;
use crate::mounting::UiMountedProjectionDenial;

#[path = "mechanic_source/view.rs"]
mod view;

#[derive(Clone, Default)]
pub(in crate::mounting::projection) struct UiMountedMechanicSource {
    semantic_text: UiMountedSemanticMechanicSource,
    hit_tests: super::UiMountedHitMechanicSource,
    pub(super) presented_hits: crate::mounting::presented_hit_index::UiPresentedHitIndex,
}

pub(super) struct UiMountedMechanicCompletion<'a> {
    pub(super) frame: worth_ui_host_contract::UiMountedFrameIdentity,
    pub(super) content: worth_ui_host_contract::UiMountedContentGeneration,
    pub(super) receipts: &'a super::super::super::UiMountedNodeReceiptBasis,
    pub(super) semantic: &'a UiMountedSemanticProjection,
    pub(super) changed: &'a [UiMountedInstanceIdentity],
    pub(super) capability_generation:
        worth_ui_host_contract::WorthUiHostCapabilityObservationGeneration,
    pub(super) capability_profile_digest: u64,
    pub(super) font_collection: &'a Arc<worth_ui_text::UiGlobalFontCollection>,
}

#[derive(Default)]
pub(super) struct UiMountedMechanicMutation {
    pub(super) semantic_text: usize,
    pub(super) hit_tests: usize,
    pub(super) hit_index_work: crate::mounting::hit_test_work::UiHitTestSpatialWork,
    pub(super) command_changes: Vec<worth_ui_host_contract::UiMountedPaintCommandChange>,
    pub(super) precise_instances: Vec<UiMountedInstanceIdentity>,
}

impl UiMountedMechanicSource {
    pub(super) fn qualified_layout_count(&self) -> usize {
        self.semantic_text.qualified_layout_count()
    }

    pub(super) fn apply(
        &mut self,
        completion: UiMountedMechanicCompletion<'_>,
    ) -> Result<UiMountedMechanicMutation, UiMountedProjectionDenial> {
        self.semantic_text
            .preflight(completion.changed, completion.semantic)?;
        let mut mutation = UiMountedMechanicMutation::default();
        let qualification_cache =
            super::super::semantic_text::UiMountedTextQualificationCache::default();
        for instance in completion.changed {
            let Some(node) = completion.semantic.node(*instance) else {
                mutation
                    .hit_index_work
                    .merge(self.hit_tests.replace(*instance, None)?);
                mutation
                    .command_changes
                    .extend(self.semantic_text.remove_instance(*instance));
                continue;
            };
            let text_context =
                super::super::semantic_text::UiMountedSemanticTextCompletionContext {
                    frame: completion.frame,
                    content_generation: completion.content,
                    receipt_basis: completion.receipts,
                    semantic: completion.semantic,
                    capability_generation: completion.capability_generation,
                    capability_profile_digest: completion.capability_profile_digest,
                    font_collection: completion.font_collection,
                    qualification_cache: &qualification_cache,
                };
            let sparse = node.semantic_text.as_ref().and_then(|seed| {
                self.semantic_text
                    .apply_paint_only(&text_context, node, seed)
                    .or_else(|| {
                        self.semantic_text
                            .apply_collection_patch(&text_context, node, seed)
                    })
            });
            let text_update = match sparse {
                Some(update) => {
                    let update = update?;
                    mutation.precise_instances.push(*instance);
                    update
                }
                None => {
                    let text = super::super::semantic_text::complete_node_semantic_text(
                        &text_context,
                        node,
                    )?;
                    self.semantic_text.replace_instance(*instance, text)?
                }
            };
            mutation.semantic_text = mutation
                .semantic_text
                .checked_add(text_update.rows_materialized)
                .ok_or(UiMountedProjectionDenial::SemanticTextCapacityExceeded)?;
            mutation.command_changes.extend(text_update.command_changes);
            let hit = super::super::hit_test::complete_hit_test(
                completion.frame,
                completion.receipts,
                completion.semantic,
                node,
            )?;
            mutation
                .hit_index_work
                .merge(self.hit_tests.replace(*instance, hit)?);
            mutation.hit_tests += usize::from(hit.is_some());
        }
        self.validate_capacity()?;
        Ok(mutation)
    }

    fn validate_capacity(&self) -> Result<(), UiMountedProjectionDenial> {
        if self.semantic_text.len() > worth_ui_host_contract::UiMountedSemanticTextTable::MAX_ROWS {
            return Err(UiMountedProjectionDenial::SemanticTextCapacityExceeded);
        }
        if self.semantic_text.byte_len()
            > worth_ui_host_contract::UiMountedSemanticTextTable::MAX_BYTES
        {
            return Err(UiMountedProjectionDenial::SemanticTextCapacityExceeded);
        }
        if self.hit_tests.len() > worth_ui_host_contract::UiMountedHitTestTable::MAX_ROWS {
            return Err(UiMountedProjectionDenial::HitTestCapacityExceeded);
        }
        Ok(())
    }

    pub(super) fn qualified_layout(
        &self,
        identity: worth_ui_host_contract::UiQualifiedTextLayoutIdentity,
    ) -> Option<&std::sync::Arc<worth_ui_text::UiQualifiedTextLayout>> {
        self.semantic_text.qualified_layout(identity)
    }

    pub(super) fn require_qualified_layout_reconstruction(
        &mut self,
    ) -> Result<usize, super::super::UiMountedProjectionDenial> {
        self.semantic_text.require_layout_reconstruction()
    }

    pub(super) fn reconstruct_qualified_layouts(
        &mut self,
    ) -> Result<usize, super::super::UiMountedProjectionDenial> {
        self.semantic_text.reconstruct_layouts()
    }

    pub(super) fn qualified_layout_reconstruction_required(&self) -> bool {
        self.semantic_text.layout_reconstruction_required()
    }

    #[cfg(test)]
    pub(super) fn qualified_layout_for(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        slot: worth_ui_host_contract::UiSemanticTextSlot,
    ) -> Option<&std::sync::Arc<worth_ui_text::UiQualifiedTextLayout>> {
        self.semantic_text.qualified_layout_for(instance, slot)
    }

    #[cfg(test)]
    pub(super) fn begin_semantic_instance_index_observation(&self) {
        crate::runtime::persistent_index::begin_all_test_observation();
    }

    #[cfg(test)]
    pub(super) fn retained_semantic_row_count_for_test(&self) -> usize {
        self.semantic_text.retained_iter().count()
    }

    #[cfg(test)]
    pub(super) fn collection_layouts_for_test(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> std::collections::BTreeMap<[u8; 32], std::sync::Arc<worth_ui_text::UiQualifiedTextLayout>>
    {
        self.semantic_text.collection_layouts_for(instance)
    }

    pub(super) fn commands_for_instance(
        &self,
        instance: UiMountedInstanceIdentity,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        binding: worth_ui_host_contract::UiSurfaceBindingGeneration,
    ) -> Arc<[worth_ui_host_contract::UiMountedPaintCommand]> {
        let mut commands = Vec::new();
        commands.extend(
            self.semantic_text
                .rows_for_instance(instance)
                .filter(|row| row.surface() == surface && row.binding() == binding)
                .cloned()
                .map(
                    |mechanic| worth_ui_host_contract::UiMountedPaintCommand::SemanticText {
                        identity:
                            worth_ui_host_contract::UiMountedPaintCommandIdentity::semantic_text(
                                &mechanic,
                            ),
                        mechanic,
                    },
                ),
        );
        commands.sort_by_key(worth_ui_host_contract::UiMountedPaintCommand::layer_semantic_order);
        commands.into()
    }

    pub(super) fn rebind(
        &mut self,
        replacements: &[(
            worth_ui_host_contract::UiSurfaceBindingGeneration,
            crate::mounting::UiSurfaceBindingIdentityView,
        )],
        semantic: &super::UiMountedSemanticProjection,
    ) -> Result<crate::mounting::hit_test_work::UiHitTestSpatialWork, UiMountedProjectionDenial>
    {
        let mut text = self
            .semantic_text
            .retained_iter()
            .cloned()
            .collect::<Vec<_>>();
        super::super::semantic_text::rebind_semantic_text(&mut text, replacements, semantic)?;
        let (hit_tests, hit_work) = self.hit_tests.rebind(replacements)?;
        self.semantic_text = UiMountedSemanticMechanicSource::default();
        self.semantic_text.replace_all(text)?;
        self.hit_tests = hit_tests;
        Ok(hit_work)
    }

    pub(super) fn table_digest(&self) -> u64 {
        [
            self.semantic_text.len() as u64,
            self.hit_tests.len() as u64,
            self.semantic_text.digest(),
            self.hit_tests.digest(),
        ]
        .into_iter()
        .fold(0x6d65_6368_736f_7572_u64, |digest, value| {
            digest.rotate_left(11) ^ value
        })
    }

    pub(in crate::mounting) fn visual_region_basis(
        &self,
    ) -> crate::mounting::UiMountedVisualRegionBasis {
        crate::mounting::UiMountedVisualRegionBasis::from_persistent(
            self.hit_tests.clone(),
            self.semantic_text.clone(),
            self.presented_hits.clone(),
        )
    }
}
