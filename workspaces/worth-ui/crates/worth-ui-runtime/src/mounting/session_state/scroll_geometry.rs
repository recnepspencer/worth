impl super::WorthUiMountedSessionState {
    pub(crate) fn scroll_geometry_reservations(
        &self,
    ) -> Option<
        std::rc::Rc<
            std::collections::BTreeMap<worth_ui_host_contract::UiSemanticSurfaceIdentity, usize>,
        >,
    > {
        self.occurrence_geometry.scroll_geometry_reservations()
    }

    pub(crate) fn retained_scroll_chrome_geometry(
        &self,
        target: crate::runtime::motion::UiMotionTargetIdentity,
    ) -> Option<(
        worth_ui_host_contract::UiMountedCanonicalBox,
        worth_ui_host_contract::UiMountedCanonicalBox,
        worth_ui_host_contract::UiMountedCanonicalBox,
    )> {
        // This lookup admits the exact retained physical epoch and live binding,
        // not merely a frame number supplied by a candidate geometry caller.
        let presentation = self.current_presentation_for_surface(target.semantic_surface())?;
        self.presentation
            .retained_scroll_chrome_geometry(presentation, target)
    }

    pub(crate) fn scroll_presentation_members(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        owner: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> Option<
        std::sync::Arc<[crate::mounting::presentation::work_producer::UiMountedScrollMotionMember]>,
    > {
        self.occurrence_geometry
            .scroll_presentation_members(surface, owner)
    }

    pub(crate) fn rebase_presented_scroll_extent(
        &mut self,
        target: crate::runtime::motion::UiMotionTargetIdentity,
        tick: u64,
    ) {
        self.motion_sampling
            .rebase_presented_scroll_extent(target, tick)
            .expect("published extent and admitted Motion request carry finite geometry");
    }

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

    /// Move every named region's descendants to the offset it names, and
    /// report the presented hit transitions that crossing left behind.
    ///
    /// Displayed geometry and reachable geometry are two readings of one
    /// displacement, so they are taken from one prepared pose rather than
    /// separately: there is no ordering in which a caller can commit a pose
    /// whose pixels and whose hit rows disagree. A caller that has moved
    /// content under a pointer hands the transitions to interaction, which is
    /// how hover re-resolves without a synthetic pointer event.
    #[cfg(test)]
    pub(crate) fn apply_scroll_geometries(
        &mut self,
        poses: &[(
            worth_ui_host_contract::UiSemanticSurfaceIdentity,
            worth_ui_host_contract::UiMountedInstanceIdentity,
            crate::runtime::scroll::UiScrollOffset,
        )],
    ) -> Result<
        Box<[crate::mounting::UiCommittedPresentedHitTransition]>,
        super::super::UiMountedOccurrenceGeometryDenial,
    > {
        self.apply_scroll_geometry_changes(poses, true)
    }

    /// A host-accepted sample has already moved retained paint. Update its
    /// geometry and hit rows without scheduling that same paint a second time.
    pub(crate) fn apply_presented_scroll_geometries(
        &mut self,
        poses: &[(
            worth_ui_host_contract::UiSemanticSurfaceIdentity,
            worth_ui_host_contract::UiMountedInstanceIdentity,
            crate::runtime::scroll::UiScrollOffset,
        )],
    ) -> Result<
        Box<[crate::mounting::UiCommittedPresentedHitTransition]>,
        super::super::UiMountedOccurrenceGeometryDenial,
    > {
        self.apply_scroll_geometry_changes(poses, false)
    }

    fn apply_scroll_geometry_changes(
        &mut self,
        poses: &[(
            worth_ui_host_contract::UiSemanticSurfaceIdentity,
            worth_ui_host_contract::UiMountedInstanceIdentity,
            crate::runtime::scroll::UiScrollOffset,
        )],
        owes_paint: bool,
    ) -> Result<
        Box<[crate::mounting::UiCommittedPresentedHitTransition]>,
        super::super::UiMountedOccurrenceGeometryDenial,
    > {
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
        if owes_paint && !changed.is_empty() {
            self.identity
                .mark_occurrence_geometry_changed(&changed)
                .map_err(|_| Denial::StateRevisionExhausted)?;
        }
        let mut transitions = Vec::new();
        let mut hit_work = crate::mounting::UiHitTestSpatialWork::default();
        for pose in prepared {
            hit_work.merge(pose.work());
            let (transition, refreshed) = self
                .retention
                .refresh_presented_hit_scroll(pose.surface(), pose.translations());
            hit_work.merge(refreshed);
            if let Some(transition) = transition {
                transitions.push(transition);
            }
            self.occurrence_geometry.apply_scroll_pose(pose);
        }
        self.last_scroll_hit_index_work = hit_work;
        Ok(transitions.into_boxed_slice())
    }

    /// Presented hit-index maintenance done for the last call that applied
    /// poses, summed over every surface that call moved. A call that moved
    /// nothing reachable reports zero rather than nothing, because it did look.
    pub(crate) const fn last_scroll_hit_index_work(&self) -> crate::mounting::UiHitTestSpatialWork {
        self.last_scroll_hit_index_work
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

    pub(crate) fn accepted_scroll_group_translation(
        &self,
        target: crate::runtime::motion::UiMotionTargetIdentity,
    ) -> Option<[f32; 2]> {
        self.motion_sampling
            .accepted_scroll_group_translation(target)
    }
}
