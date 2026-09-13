impl super::UiMountedIdentityState {
    pub(crate) fn text_publication_coverage(
        &self,
        graph: crate::graph::UiGraphNodeIdentity,
        surfaces: &[worth_ui_host_contract::UiMountedSurfaceBindingRequirement],
    ) -> Result<
        (
            crate::runtime::presentation_state::UiApplicationTextMountedCoverage,
            usize,
        ),
        super::super::UiMountedFramePreparationDenial,
    > {
        let deny = || {
            super::super::UiMountedFramePreparationDenial::Projection(
                super::super::UiMountedProjectionDenial::CostCounterOverflow,
            )
        };
        let affected = self
            .try_projection_instances_for_graph_nodes(&[graph])
            .ok_or_else(deny)?;
        let mut occurrences = Vec::with_capacity(affected.instances().len());
        let work = affected
            .index_entries_touched()
            .checked_add(affected.instances().len())
            .ok_or_else(deny)?;
        let surfaces = surfaces
            .iter()
            .map(|surface| surface.semantic_surface())
            .collect::<std::collections::BTreeSet<_>>();
        for instance in affected.instances() {
            let view = self.projection_instance(*instance).ok_or(
                super::super::UiMountedFramePreparationDenial::Projection(
                    super::super::UiMountedProjectionDenial::UnknownGraphNode,
                ),
            )?;
            occurrences.push((
                *instance,
                view.mount_incarnation(),
                surfaces.contains(&view.basis().semantic_surface_identity()),
            ));
        }
        Ok((
            crate::runtime::presentation_state::UiApplicationTextMountedCoverage::from_occurrences(
                occurrences,
            ),
            work,
        ))
    }
}
