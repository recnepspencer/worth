use super::index::{CompiledOwnerIndex, OccurrenceIndex, RegionIndex};

impl super::UiMountedOccurrenceGeometryValidationAuthority<'_> {
    pub(super) fn validate_complete_seam_geometry(
        &self,
        surface: Option<worth_ui_dsl::UiSemanticSurfaceDeclarationIdentity>,
        occurrences: &OccurrenceIndex,
        submitted_regions: &RegionIndex,
        compiled_owners: &mut CompiledOwnerIndex,
        work: &mut super::validation_authority::UiMountedGeometryValidationWork,
    ) -> Result<(), crate::mounting::UiMountedOccurrenceGeometryDenial> {
        let Some(surface) = surface else {
            return Ok(());
        };
        for owner in occurrences.keys().copied() {
            let basis = self.basis(owner).ok_or(
                crate::mounting::UiMountedOccurrenceGeometryDenial::UnknownMountedInstance,
            )?;
            let regions = self.owner_regions(
                compiled_owners,
                surface,
                basis.graph_node_identity(),
                &mut work.region_plan_rows_visited,
            );
            for binding in regions.bindings() {
                work.region_lookup_steps += 1;
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
