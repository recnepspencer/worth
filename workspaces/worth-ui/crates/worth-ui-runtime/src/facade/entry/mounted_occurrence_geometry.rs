use super::WorthUiActiveApplicationSession;
use crate::facade::mounted::{
    UiMountedLayoutCompletionReceipt, UiMountedOccurrenceGeometryDenial,
    UiMountedSurfaceGeometryBatch,
};

#[path = "mounted_occurrence_geometry/index.rs"]
mod index;
#[path = "mounted_occurrence_geometry/mosaic_clip.rs"]
mod mosaic_clip;
#[path = "mounted_occurrence_geometry/seam_validation.rs"]
mod seam_validation;

use index::{compiled_owner_regions, CompiledOwnerIndex, OccurrenceIndex, RegionIndex};

/// Borrowed authority for committing completed mounted layout.
pub struct WorthUiMountedLayout<'session> {
    session: &'session mut WorthUiActiveApplicationSession,
}

impl WorthUiMountedLayout<'_> {
    pub fn basis(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> Result<crate::mounting::UiMountedLayoutBasis, UiMountedOccurrenceGeometryDenial> {
        self.session
            .mounted
            .layout_basis(surface, self.session.generation_identity().clone())
    }

    pub fn complete_surface_geometry(
        &mut self,
        mut batch: UiMountedSurfaceGeometryBatch,
    ) -> Result<UiMountedLayoutCompletionReceipt, UiMountedOccurrenceGeometryDenial> {
        if batch.basis().generation() != self.session.generation_identity() {
            return Err(UiMountedOccurrenceGeometryDenial::StaleLayoutBasis);
        }
        let declaration = self
            .session
            .authored_overlay_bindings
            .bound_owners()
            .find_map(|(declaration, runtime, _)| {
                (runtime == batch.surface()).then_some(declaration)
            });
        let mut compiled_owners = CompiledOwnerIndex::new();
        let mut region_plan_rows_visited = 0;
        let mut region_lookup_steps = 0;
        let mut region_kinds = std::collections::BTreeMap::new();
        for region in batch.regions() {
            let declaration =
                declaration.ok_or(UiMountedOccurrenceGeometryDenial::ForeignRegionDeclaration)?;
            let basis = self
                .session
                .mounted
                .current_mounted_identity_basis(region.owner())
                .ok_or(UiMountedOccurrenceGeometryDenial::UnknownMountedInstance)?;
            if basis.semantic_surface_identity() != batch.surface() {
                return Err(UiMountedOccurrenceGeometryDenial::ForeignSurface);
            }
            let regions = compiled_owner_regions(
                &self.session.application,
                &mut compiled_owners,
                declaration,
                basis.graph_node_identity(),
                &mut region_plan_rows_visited,
            );
            region_lookup_steps += 1;
            if !regions.contains(region.executed_region(), region.declaration()) {
                return Err(UiMountedOccurrenceGeometryDenial::UnknownExecutedRegion);
            }
            let region_kind = regions
                .region_kind(region.executed_region(), region.declaration())
                .and_then(|kind| crate::capability::MosaicRegionKindId::new(kind).ok())
                .ok_or(UiMountedOccurrenceGeometryDenial::UnknownExecutedRegion)?;
            region_kinds.insert((region.owner(), region.declaration()), region_kind);
        }
        let occurrence_index = batch
            .occurrences()
            .iter()
            .copied()
            .map(|occurrence| (occurrence.instance(), occurrence))
            .collect::<OccurrenceIndex>();
        let region_index = batch
            .regions()
            .iter()
            .map(|region| (region.owner(), region.declaration()))
            .collect::<RegionIndex>();
        let region_index_rows = batch.regions().len();
        if let Some(contract) = self
            .session
            .application
            .capabilities()
            .mosaic_regions()
            .seam_paint()
            .cloned()
        {
            self.validate_complete_seam_geometry(
                declaration,
                &occurrence_index,
                &region_index,
                &mut compiled_owners,
                &mut region_plan_rows_visited,
                &mut region_lookup_steps,
            )?;
            batch = batch.with_mosaic_seam_paint(contract, region_kinds);
        }
        let mut occurrence_ancestry_steps = 0;
        let mosaic_clips = self.resolve_mosaic_clips(
            &batch,
            &occurrence_index,
            &region_index,
            declaration,
            &mut compiled_owners,
            &mut region_plan_rows_visited,
            &mut occurrence_ancestry_steps,
            &mut region_lookup_steps,
        )?;
        batch = batch.with_mosaic_clips(mosaic_clips);
        let mut staged_scroll = self.session.scroll.as_ref().cloned();
        let scroll_clips = self.resolve_scroll_clips(
            &batch,
            &occurrence_index,
            &region_index,
            declaration,
            staged_scroll.as_mut(),
            &mut occurrence_ancestry_steps,
            &mut region_lookup_steps,
        );
        batch = batch.with_scroll_clips(scroll_clips);
        let receipt = self
            .session
            .mounted
            .replace_occurrence_geometry(batch)
            .map(|receipt| {
                receipt
                    .with_region_plan_rows_visited(region_plan_rows_visited)
                    .with_occurrence_resolution_work(
                        occurrence_index.len(),
                        occurrence_ancestry_steps,
                    )
                    .add_region_resolution_work(region_index_rows, region_lookup_steps)
            })?;
        if let Some(staged) = staged_scroll {
            *self
                .session
                .scroll
                .as_mut()
                .expect("the staged Scroll owner came from an installed service") = staged;
        }
        Ok(receipt)
    }

    fn resolve_scroll_clips(
        &self,
        batch: &UiMountedSurfaceGeometryBatch,
        occurrence_index: &OccurrenceIndex,
        region_index: &RegionIndex,
        surface_declaration: Option<worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity>,
        scroll: Option<&mut crate::runtime::scroll::UiScrollRuntimeState>,
        occurrence_ancestry_steps: &mut usize,
        region_lookup_steps: &mut usize,
    ) -> std::collections::BTreeMap<
        worth_ui_host_contract::UiMountedInstanceIdentity,
        Result<
            Box<[crate::mounting::UiMountedScrollClipBinding]>,
            crate::graph::UiGraphNodeIdentity,
        >,
    > {
        let Some(scroll) = scroll else {
            return std::collections::BTreeMap::new();
        };
        let mut resolved = std::collections::BTreeMap::new();
        for occurrence in batch.occurrences() {
            let Some(basis) = self
                .session
                .mounted
                .current_mounted_identity_basis(occurrence.instance())
            else {
                continue;
            };
            let incarnation =
                crate::runtime::scroll::UiScrollOwnerIncarnation::from_mount_incarnation(
                    basis.mount_incarnation(),
                );
            self.session.application.install_scroll_ownership(
                scroll,
                occurrence.instance(),
                incarnation,
                &basis,
            );
            let result = scroll
                .ownership_chain(occurrence.instance())
                .map_err(|_| basis.graph_node_identity())
                .and_then(|chain| {
                    chain
                        .owners()
                        .iter()
                        .copied()
                        .map(|owner| {
                            self.scroll_clip_binding(
                                batch,
                                occurrence_index,
                                region_index,
                                occurrence.instance(),
                                surface_declaration,
                                owner,
                                occurrence_ancestry_steps,
                                region_lookup_steps,
                            )
                            .ok_or(basis.graph_node_identity())
                        })
                        .collect::<Result<Vec<_>, _>>()
                        .map(Vec::into_boxed_slice)
                });
            resolved.insert(occurrence.instance(), result);
        }
        resolved
    }

    fn scroll_clip_binding(
        &self,
        batch: &UiMountedSurfaceGeometryBatch,
        occurrence_index: &OccurrenceIndex,
        region_index: &RegionIndex,
        target: worth_ui_host_contract::UiMountedInstanceIdentity,
        surface_declaration: Option<worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity>,
        owner: crate::runtime::scroll::UiScrollOwnerIdentity,
        occurrence_ancestry_steps: &mut usize,
        region_lookup_steps: &mut usize,
    ) -> Option<crate::mounting::UiMountedScrollClipBinding> {
        use crate::runtime::scroll::UiScrollOwnerIdentity as Owner;
        match owner {
            Owner::Surface(surface) | Owner::Viewport(surface) if surface == batch.surface() => {
                Some(crate::mounting::UiMountedScrollClipBinding::Viewport)
            }
            Owner::Region {
                surface,
                region,
                repeated_instance_digest,
                plan_region_index,
            } if surface == batch.surface() => {
                let exact_owner = self.exact_ancestor_occurrence(
                    occurrence_index,
                    target,
                    region,
                    repeated_instance_digest,
                    occurrence_ancestry_steps,
                )?;
                if plan_region_index == u32::MAX || surface_declaration.is_none() {
                    return Some(crate::mounting::UiMountedScrollClipBinding::Occurrence(
                        exact_owner,
                    ));
                }
                let Some(declaration) = self
                    .session
                    .application
                    .mounted_region_declaration_for_plan_index(
                        surface_declaration?,
                        plan_region_index,
                    )
                else {
                    return Some(crate::mounting::UiMountedScrollClipBinding::Occurrence(
                        exact_owner,
                    ));
                };
                *region_lookup_steps += 1;
                region_index
                    .contains(&(exact_owner, declaration))
                    .then_some(crate::mounting::UiMountedScrollClipBinding::Region {
                        owner: exact_owner,
                        declaration,
                    })
            }
            _ => None,
        }
    }

    fn exact_ancestor_occurrence(
        &self,
        occurrence_index: &OccurrenceIndex,
        target: worth_ui_host_contract::UiMountedInstanceIdentity,
        graph_node: crate::graph::UiGraphNodeIdentity,
        repeated_instance_digest: u64,
        occurrence_ancestry_steps: &mut usize,
    ) -> Option<worth_ui_host_contract::UiMountedInstanceIdentity> {
        let mut cursor = Some(target);
        for _ in 0..occurrence_index.len() {
            let instance = cursor?;
            *occurrence_ancestry_steps += 1;
            let occurrence = occurrence_index.get(&instance)?;
            let basis = self
                .session
                .mounted
                .current_mounted_identity_basis(instance)?;
            if basis.graph_node_identity() == graph_node
                && basis.repeated_instance_basis().identity_digest() == repeated_instance_digest
            {
                return Some(instance);
            }
            cursor = occurrence.parent();
        }
        None
    }
}

impl WorthUiActiveApplicationSession {
    pub fn begin_mounted_layout(&mut self) -> WorthUiMountedLayout<'_> {
        WorthUiMountedLayout { session: self }
    }
}
