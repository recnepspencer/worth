impl super::WorthUiMountedSessionState {
    pub(crate) fn scroll_region_geometry(
        &self,
        target: worth_ui_host_contract::UiMountedInstanceIdentity,
        slot: usize,
    ) -> Option<(
        worth_ui_host_contract::UiMountedInstanceIdentity,
        worth_ui_host_contract::UiMountedCanonicalBox,
        worth_ui_host_contract::UiMountedCanonicalBox,
    )> {
        let instance = self.identity.projection_instance(target)?;
        self.occurrence_geometry.scroll_region_geometry(
            instance.basis().semantic_surface_identity(),
            target,
            slot,
        )
    }

    pub(crate) fn apply_scroll_geometries(
        &mut self,
        poses: &[(
            worth_ui_host_contract::UiSemanticSurfaceIdentity,
            worth_ui_host_contract::UiMountedInstanceIdentity,
            crate::runtime::scroll::UiScrollOffset,
        )],
    ) -> Result<(), super::super::UiMountedOccurrenceGeometryDenial> {
        use super::super::UiMountedOccurrenceGeometryDenial as Denial;
        if self.has_active_presentation_attempt() {
            return Err(Denial::PresentationInFlight);
        }
        let mut surfaces = std::collections::BTreeMap::<_, Vec<_>>::new();
        for (surface, owner, offset) in poses {
            surfaces
                .entry(*surface)
                .or_default()
                .push((*owner, *offset));
        }
        let prepared = surfaces
            .into_iter()
            .map(|(surface, poses)| {
                self.occurrence_geometry
                    .prepare_scroll_pose(surface, &poses)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let changed = prepared
            .iter()
            .flat_map(|pose| pose.changed_instances().into_vec())
            .collect::<Vec<_>>();
        if !changed.is_empty() {
            self.identity
                .mark_occurrence_geometry_changed(&changed)
                .map_err(|_| Denial::StateRevisionExhausted)?;
        }
        for pose in prepared {
            self.occurrence_geometry.apply_scroll_pose(pose);
        }
        Ok(())
    }
}
