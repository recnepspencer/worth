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

    /// The Scroll region owner that `instance` travels with, when it is
    /// scrolled content rather than a region owner in its own right.
    pub(crate) fn scrolled_content_owner(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> Option<worth_ui_host_contract::UiMountedInstanceIdentity> {
        let projected = self.identity.projection_instance(instance)?;
        self.occurrence_geometry
            .scrolled_content_owner(projected.basis().semantic_surface_identity(), instance)
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

    /// Retire one Scroll content group's Motion sample outright, because a
    /// pointer has taken direct control of the group's offset. From here until
    /// a later settle installs a new track, the displayed pose is whatever
    /// direct control applies, and no accepted sample stands behind it.
    pub(crate) fn retire_scroll_motion_sample(
        &mut self,
        target: crate::runtime::motion::UiMotionTargetIdentity,
    ) -> bool {
        self.motion_sampling.retire_scroll_group_track(target)
    }

    /// The accepted translation of every retained Scroll content group, keyed
    /// by the target that names it. This is the sole source of displayed
    /// scrolled geometry: it reports what the host has already presented, never
    /// the semantic target the content is still travelling toward.
    pub(crate) fn accepted_scroll_group_translations(
        &self,
    ) -> Vec<(crate::runtime::motion::UiMotionTargetIdentity, [f32; 2])> {
        self.motion_sampling
            .retained_targets()
            .into_iter()
            .filter_map(|target| {
                self.motion_sampling
                    .accepted_scroll_group_translation(target)
                    .map(|translation| (target, translation))
            })
            .collect()
    }
}
