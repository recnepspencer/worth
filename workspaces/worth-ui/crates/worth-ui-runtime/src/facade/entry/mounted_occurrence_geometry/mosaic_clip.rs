use super::index::{compiled_owner_regions, CompiledOwnerIndex, OccurrenceIndex, RegionIndex};
use crate::facade::mounted::{UiMountedOccurrenceGeometryDenial, UiMountedSurfaceGeometryBatch};

impl super::WorthUiMountedLayout<'_> {
    pub(super) fn resolve_mosaic_clips(
        &self,
        batch: &UiMountedSurfaceGeometryBatch,
        occurrence_index: &OccurrenceIndex,
        region_index: &RegionIndex,
        surface_declaration: Option<worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity>,
        compiled_owners: &mut CompiledOwnerIndex,
        region_plan_rows_visited: &mut usize,
        occurrence_ancestry_steps: &mut usize,
        region_lookup_steps: &mut usize,
    ) -> Result<
        std::collections::BTreeMap<
            worth_ui_host_contract::UiMountedInstanceIdentity,
            Box<[crate::mounting::UiMountedMosaicClipBinding]>,
        >,
        UiMountedOccurrenceGeometryDenial,
    > {
        let Some(surface_declaration) = surface_declaration else {
            return Ok(std::collections::BTreeMap::new());
        };
        let mut resolved = std::collections::BTreeMap::new();
        for target in batch.occurrences().iter().map(|row| row.instance()) {
            let mut clips = Vec::new();
            let mut cursor = Some(target);
            for _ in 0..occurrence_index.len() {
                let Some(owner) = cursor else {
                    break;
                };
                *occurrence_ancestry_steps += 1;
                let basis = self
                    .session
                    .mounted
                    .current_mounted_identity_basis(owner)
                    .ok_or(UiMountedOccurrenceGeometryDenial::UnknownMountedInstance)?;
                let bindings = compiled_owner_regions(
                    &self.session.application,
                    compiled_owners,
                    surface_declaration,
                    basis.graph_node_identity(),
                    region_plan_rows_visited,
                )
                .bindings();
                let direct_depth = bindings.iter().map(|binding| binding.depth()).min();
                let direct = bindings
                    .iter()
                    .filter(|binding| Some(binding.depth()) == direct_depth)
                    .collect::<Vec<_>>();
                if direct
                    .iter()
                    .filter(|binding| {
                        matches!(
                            binding.clipping(),
                            crate::capability::MosaicClippingPosture::ClipToRegion
                        )
                    })
                    .count()
                    > 1
                {
                    return Err(UiMountedOccurrenceGeometryDenial::AmbiguousMosaicRegionGeometry);
                }
                for binding in direct {
                    use crate::capability::MosaicClippingPosture as Clipping;
                    match binding.clipping() {
                        Clipping::AllowOverlayEscape => {}
                        Clipping::ViewportClipped => {
                            clips.push(crate::mounting::UiMountedMosaicClipBinding::Viewport)
                        }
                        Clipping::ClipToRegion => {
                            *region_lookup_steps += 1;
                            if !region_index.contains(&(owner, binding.declaration())) {
                                return Err(
                                    UiMountedOccurrenceGeometryDenial::MissingMosaicRegionGeometry,
                                );
                            }
                            clips.push(crate::mounting::UiMountedMosaicClipBinding::Region {
                                owner,
                                declaration: binding.declaration(),
                            });
                        }
                        Clipping::MissingForDiagnostics => {
                            return Err(
                                UiMountedOccurrenceGeometryDenial::MosaicBindingUnavailable,
                            );
                        }
                    }
                }
                cursor = occurrence_index.get(&owner).and_then(|row| row.parent());
            }
            if cursor.is_some() {
                return Err(UiMountedOccurrenceGeometryDenial::ParentCycle);
            }
            if !clips.is_empty() {
                resolved.insert(target, clips.into_boxed_slice());
            }
        }
        Ok(resolved)
    }
}
