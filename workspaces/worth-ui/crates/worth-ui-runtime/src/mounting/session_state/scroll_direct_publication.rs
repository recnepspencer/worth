//! Carry exact direct Scroll preparation through mounted frame authority.
use crate::runtime::scroll::UiPreparedScrollDirectSuccession;
use worth_ui_host_contract::UiSemanticSurfaceIdentity;

impl super::WorthUiMountedSessionState {
    pub(super) fn retire_direct_scroll_surface(&mut self, surface: UiSemanticSurfaceIdentity) {
        self.pending_direct_scroll
            .retain(|_, prepared| prepared.surface() != surface);
    }

    pub(super) fn retire_direct_scroll_occurrence(
        &mut self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) {
        self.pending_direct_scroll.retain(|_, prepared| {
            prepared.occurrence() != instance
                && prepared
                    .pose()
                    .is_none_or(|(_, owner, _)| owner != instance)
        });
    }

    pub(crate) fn reconcile_direct_scroll_evidence(
        &mut self,
        scroll: Option<&crate::runtime::scroll::UiScrollRuntimeState>,
    ) {
        self.pending_direct_scroll.retain(|_, prepared| {
            scroll.is_some_and(|scroll| scroll.has_direct_succession(prepared))
        });
    }

    pub(crate) fn stage_direct_scroll_geometry(
        &mut self,
        prepared: &[UiPreparedScrollDirectSuccession],
    ) -> Result<(), crate::mounting::UiMountedOccurrenceGeometryDenial> {
        use crate::mounting::UiMountedOccurrenceGeometryDenial as Denial;
        if self.has_active_presentation_attempt() {
            return Err(Denial::PresentationInFlight);
        }
        let mut surfaces = std::collections::BTreeMap::<_, Vec<_>>::new();
        for (surface, owner, offset) in prepared.iter().filter_map(|record| record.pose()) {
            surfaces.entry(surface).or_default().push((owner, offset));
        }
        let poses = surfaces
            .into_iter()
            .map(|(surface, poses)| {
                self.occurrence_geometry
                    .prepare_scroll_pose(surface, &poses)
            })
            .collect::<Result<Vec<_>, _>>()?;
        // Even two inputs with equal geometry have distinct consequences. An
        // older prepared frame must never acknowledge the newer pending one.
        let mut changed = poses
            .iter()
            .flat_map(|pose| pose.changed_instances().into_vec())
            .collect::<Vec<_>>();
        if changed.is_empty() {
            // A chrome-only or equal-pose succession still needs an ordinary
            // attempt to acknowledge its exact cause. Moving content otherwise
            // invalidates only its descendants, never the stationary owner.
            changed.extend(prepared.iter().map(|record| record.occurrence()));
        }
        changed.sort_unstable();
        changed.dedup();
        self.identity
            .mark_occurrence_geometry_changed(&changed)
            .map_err(|_| Denial::StateRevisionExhausted)?;
        for pose in poses {
            self.occurrence_geometry.apply_scroll_pose(pose);
        }
        let identity = &self.identity;
        self.pending_direct_scroll
            .retain(|_, record| identity.projection_instance(record.occurrence()).is_some());
        for record in prepared {
            self.pending_direct_scroll.insert(record.owner(), *record);
        }
        Ok(())
    }

    pub(crate) fn has_pending_direct_scroll(&self, surface: UiSemanticSurfaceIdentity) -> bool {
        self.pending_direct_scroll
            .values()
            .any(|record| record.surface() == surface)
    }

    /// The caller immediately runs the existing exact prepared-frame authority
    /// admission. A stale geometry revision rejects this captured candidate
    /// before it can reserve or perform host effects.
    pub(in crate::mounting) fn bind_pending_direct_scroll(
        &self,
        frame: &mut crate::mounting::UiPreparedMountedFrame,
    ) {
        let records = self
            .pending_direct_scroll
            .values()
            .filter(|record| {
                frame
                    .surfaces()
                    .iter()
                    .any(|surface| surface.requirement().semantic_surface() == record.surface())
            })
            .copied()
            .collect();
        frame.bind_direct_scroll(records);
    }

    pub(crate) fn settle_published_direct_scroll(
        &mut self,
        publication: &crate::mounting::UiMountedFramePublicationReceipt,
    ) {
        for record in publication.direct_scroll() {
            if publication
                .presentation_for_surface(record.surface())
                .is_some()
                && self.pending_direct_scroll.get(&record.owner()) == Some(record)
            {
                self.pending_direct_scroll.remove(&record.owner());
            }
        }
    }
}
