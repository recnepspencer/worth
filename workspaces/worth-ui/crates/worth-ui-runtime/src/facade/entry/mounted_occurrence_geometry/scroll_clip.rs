use super::index::{OccurrenceIndex, RegionIndex};
use crate::facade::mounted::UiMountedSurfaceGeometryBatch;

impl super::UiMountedOccurrenceGeometryValidationAuthority<'_> {
    pub(super) fn resolve_scroll_clips(
        &self,
        batch: &UiMountedSurfaceGeometryBatch,
        occurrences: &OccurrenceIndex,
        regions: &RegionIndex,
        surface_declaration: Option<worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity>,
        scroll: Option<&mut crate::runtime::scroll::UiScrollRuntimeState>,
        work: &mut super::validation_authority::UiMountedGeometryValidationWork,
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
            let Some(basis) = self.basis(occurrence.instance()) else {
                continue;
            };
            self.install_scroll_ownership(scroll, occurrence.instance(), basis);
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
                                occurrences,
                                regions,
                                occurrence.instance(),
                                surface_declaration,
                                owner,
                                work,
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
        occurrences: &OccurrenceIndex,
        regions: &RegionIndex,
        target: worth_ui_host_contract::UiMountedInstanceIdentity,
        surface_declaration: Option<worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity>,
        owner: crate::runtime::scroll::UiScrollOwnerIdentity,
        work: &mut super::validation_authority::UiMountedGeometryValidationWork,
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
                    occurrences,
                    target,
                    region,
                    repeated_instance_digest,
                    &mut work.occurrence_ancestry_steps,
                )?;
                if plan_region_index == u32::MAX || surface_declaration.is_none() {
                    return Some(crate::mounting::UiMountedScrollClipBinding::Occurrence(
                        exact_owner,
                    ));
                }
                let Some(declaration) =
                    self.region_declaration_for_plan_index(surface_declaration?, plan_region_index)
                else {
                    return Some(crate::mounting::UiMountedScrollClipBinding::Occurrence(
                        exact_owner,
                    ));
                };
                work.region_lookup_steps += 1;
                regions.contains(&(exact_owner, declaration)).then_some(
                    crate::mounting::UiMountedScrollClipBinding::Region {
                        owner: exact_owner,
                        declaration,
                    },
                )
            }
            _ => None,
        }
    }

    fn exact_ancestor_occurrence(
        &self,
        occurrences: &OccurrenceIndex,
        target: worth_ui_host_contract::UiMountedInstanceIdentity,
        graph_node: crate::graph::UiGraphNodeIdentity,
        repeated_instance_digest: u64,
        ancestry_steps: &mut usize,
    ) -> Option<worth_ui_host_contract::UiMountedInstanceIdentity> {
        let mut cursor = Some(target);
        for _ in 0..occurrences.len() {
            let instance = cursor?;
            *ancestry_steps += 1;
            let occurrence = occurrences.get(&instance)?;
            let basis = self.basis(instance)?;
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
