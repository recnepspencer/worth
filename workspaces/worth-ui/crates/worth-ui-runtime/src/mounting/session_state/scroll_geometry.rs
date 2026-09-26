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
        let presentation = self
            .current_presentation_for_surface(target.semantic_surface())
            .map(|displayed| displayed.basis())?;
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

    /// Where the frame on screen presents the Scroll region occurrence
    /// `owner`: where it is laid out while no frame is on screen.
    pub(crate) fn presented_region_placement(
        &self,
        owner: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> crate::mounting::UiMountedRegionPlacement {
        self.identity.current_projection().map_or(
            crate::mounting::UiMountedRegionPlacement::InPlace,
            |frame| frame.semantic_projection().region_placement(owner),
        )
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

    /// The content box of the region at `slot` of `target`'s chain with that
    /// region and every region enclosing it at offset zero: what a Scroll
    /// sample of the region is measured from.
    pub(crate) fn scroll_region_rest(
        &self,
        target: worth_ui_host_contract::UiMountedInstanceIdentity,
        slot: usize,
    ) -> Option<worth_ui_host_contract::UiMountedCanonicalBox> {
        let instance = self.identity.projection_instance(target)?;
        self.occurrence_geometry.scroll_region_rest(
            instance.basis().semantic_surface_identity(),
            target,
            slot,
        )
    }

    /// The incarnation of the region owner at `slot` of `target`'s chain:
    /// the mount incarnation of the occurrence that owns the region.
    pub(crate) fn scroll_region_incarnation(
        &self,
        target: worth_ui_host_contract::UiMountedInstanceIdentity,
        slot: usize,
    ) -> Option<crate::runtime::scroll::UiScrollOwnerIncarnation> {
        let (owner, _, _) = self.scroll_region_geometry(target, slot)?;
        let basis = self.current_mounted_identity_basis(owner)?;
        Some(
            crate::runtime::scroll::UiScrollOwnerIncarnation::from_mount_incarnation(
                basis.mount_incarnation(),
            ),
        )
    }

    /// The Scroll region owner a gesture over `instance` addresses: the
    /// instance when it owns a region, otherwise the owner it travels with.
    pub(crate) fn addressed_scroll_owner(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> Option<worth_ui_host_contract::UiMountedInstanceIdentity> {
        let projected = self.identity.projection_instance(instance)?;
        self.occurrence_geometry
            .addressed_scroll_owner(projected.basis().semantic_surface_identity(), instance)
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

    /// A witness has already displayed these samples, so retained paint is
    /// already where they put it. Settle mounted geometry and hit rows to
    /// them without scheduling that same paint a second time, unless the pose
    /// changes which content its ancestor clips suppress.
    pub(crate) fn apply_presented_scroll_geometries(
        &mut self,
        poses: &[(
            worth_ui_host_contract::UiSemanticSurfaceIdentity,
            worth_ui_host_contract::UiMountedInstanceIdentity,
            crate::mounting::presentation::UiDisplayedScrollOffset,
        )],
    ) -> Result<
        Box<[crate::mounting::UiCommittedPresentedHitTransition]>,
        super::super::UiMountedOccurrenceGeometryDenial,
    > {
        let settled = self.displayed_scroll_poses(poses)?;
        self.apply_scroll_geometry_changes(&settled, false)
    }

    /// The offsets `poses` settle at, each admitted only for the surface
    /// whose binding displayed it.
    pub(super) fn displayed_scroll_poses(
        &self,
        poses: &[(
            worth_ui_host_contract::UiSemanticSurfaceIdentity,
            worth_ui_host_contract::UiMountedInstanceIdentity,
            crate::mounting::presentation::UiDisplayedScrollOffset,
        )],
    ) -> Result<
        Vec<(
            worth_ui_host_contract::UiSemanticSurfaceIdentity,
            worth_ui_host_contract::UiMountedInstanceIdentity,
            crate::runtime::scroll::UiScrollOffset,
        )>,
        super::super::UiMountedOccurrenceGeometryDenial,
    > {
        if poses.iter().any(|(surface, _, displayed)| {
            self.current_surface_for_binding(displayed.displayed_basis().binding())
                != Some(*surface)
        }) {
            return Err(super::super::UiMountedOccurrenceGeometryDenial::ForeignSurface);
        }
        Ok(poses
            .iter()
            .map(|(surface, owner, displayed)| (*surface, *owner, displayed.settled()))
            .collect())
    }

    /// Prepare `poses` against mounted geometry, one pose per surface,
    /// without committing any of them. Hit rows move from `shown`, where the
    /// frame the host shows committed each listed owner.
    pub(super) fn prepare_scroll_poses(
        &self,
        poses: &[(
            worth_ui_host_contract::UiSemanticSurfaceIdentity,
            worth_ui_host_contract::UiMountedInstanceIdentity,
            crate::runtime::scroll::UiScrollOffset,
        )],
        shown: &[(
            worth_ui_host_contract::UiSemanticSurfaceIdentity,
            worth_ui_host_contract::UiMountedInstanceIdentity,
            crate::runtime::scroll::UiScrollOffset,
        )],
    ) -> Result<
        Vec<crate::mounting::occurrence_geometry::UiPreparedMountedScrollPose>,
        super::super::UiMountedOccurrenceGeometryDenial,
    > {
        let mut surfaces = std::collections::BTreeMap::<_, Vec<_>>::new();
        for (surface, owner, offset) in poses {
            surfaces
                .entry(*surface)
                .or_default()
                .push((*owner, *offset));
        }
        surfaces
            .into_iter()
            .map(|(surface, poses)| {
                let shown = shown
                    .iter()
                    .filter(|(shown, _, _)| *shown == surface)
                    .map(|(_, owner, offset)| (*owner, *offset))
                    .collect::<Vec<_>>();
                self.occurrence_geometry
                    .prepare_scroll_pose(surface, &poses, &shown)
            })
            .collect()
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
        // Nothing is staged past what the host shows when a pose commits, so
        // hit rows move as far as mounted geometry does.
        let prepared = self.prepare_scroll_poses(poses, &[])?;
        // A sample only moves paint the host already holds. A pose that
        // brings content out from under disjoint clips, or hides it there,
        // owes that paint, and it owes the whole pose as a direct edit does,
        // so committed geometry never mixes two poses.
        // A pose that owes paint lowers its rows anew, with every row an
        // earlier settle carried there; the rest are carried by the sample.
        let owes = |pose: &crate::mounting::occurrence_geometry::UiPreparedMountedScrollPose| {
            owes_paint || pose.changes_coverage()
        };
        let mut changed = Vec::new();
        for pose in prepared.iter().filter(|pose| owes(pose)) {
            changed.extend(pose.changed_instances().into_vec());
            changed.extend(self.occurrence_geometry.sample_carried(pose.surface()));
        }
        if !changed.is_empty() {
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
            if owes(&pose) {
                self.occurrence_geometry
                    .release_sample_carried(pose.surface());
            } else {
                self.occurrence_geometry.carry_by_sample(&pose);
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

    /// The accepted content sample of every retained Scroll content group,
    /// keyed by the target that names it. This is the sole source of scrolled
    /// geometry: once a witness displays a sample it reports what the host has
    /// already presented, never the semantic target the content is still
    /// traveling toward.
    pub(crate) fn accepted_scroll_group_samples(
        &self,
    ) -> Vec<(
        crate::runtime::motion::UiMotionTargetIdentity,
        crate::mounting::presentation::UiAcceptedRect,
    )> {
        self.motion_sampling
            .retained_targets()
            .into_iter()
            .filter_map(|target| {
                self.motion_sampling
                    .accepted_scroll_group_sample(target)
                    .map(|sample| (target, sample))
            })
            .collect()
    }

    /// Whether the frame on screen placed the Scroll group `target` where it
    /// publishes it, displacing any sample a witness displayed.
    pub(crate) fn scroll_group_placed_on_screen(
        &self,
        target: crate::runtime::motion::UiMotionTargetIdentity,
    ) -> bool {
        self.current_presentation_for_surface(target.semantic_surface())
            .is_some_and(|displayed| {
                self.presentation
                    .placed_scroll_group(displayed.basis(), target)
            })
    }

    pub(crate) fn accepted_scroll_group_sample(
        &self,
        target: crate::runtime::motion::UiMotionTargetIdentity,
    ) -> Option<crate::mounting::presentation::UiAcceptedRect> {
        self.motion_sampling.accepted_scroll_group_sample(target)
    }
}
