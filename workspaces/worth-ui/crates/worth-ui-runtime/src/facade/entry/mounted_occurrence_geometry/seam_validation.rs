use super::index::{compiled_owner_regions, CompiledOwnerIndex, OccurrenceIndex, RegionIndex};

impl super::WorthUiMountedLayout<'_> {
    pub(super) fn validate_complete_seam_geometry(
        &self,
        surface: Option<worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity>,
        occurrences: &OccurrenceIndex,
        submitted_regions: &RegionIndex,
        compiled_owners: &mut CompiledOwnerIndex,
        plan_rows_visited: &mut usize,
        region_lookup_steps: &mut usize,
    ) -> Result<(), crate::mounting::UiMountedOccurrenceGeometryDenial> {
        let Some(surface) = surface else {
            return Ok(());
        };
        for owner in occurrences.keys().copied() {
            let basis = self
                .session
                .mounted
                .current_mounted_identity_basis(owner)
                .ok_or(
                    crate::mounting::UiMountedOccurrenceGeometryDenial::UnknownMountedInstance,
                )?;
            let regions = compiled_owner_regions(
                &self.session.application,
                compiled_owners,
                surface,
                basis.graph_node_identity(),
                plan_rows_visited,
            );
            for binding in regions.bindings() {
                *region_lookup_steps += 1;
                if !submitted_regions.contains(&(owner, binding.declaration())) {
                    return Err(
                        crate::mounting::UiMountedOccurrenceGeometryDenial::MissingMosaicRegionGeometry,
                    );
                }
            }
        }
        Ok(())
    }
}
