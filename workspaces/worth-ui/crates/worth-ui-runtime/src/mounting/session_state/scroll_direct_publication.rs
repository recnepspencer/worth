//! Carry exact direct Scroll preparation through mounted frame authority.
use crate::runtime::scroll::UiPreparedScrollDirectSuccession;
use worth_ui_host_contract::UiSemanticSurfaceIdentity;

impl super::WorthUiMountedSessionState {
    /// Forget `surface`'s occurrence geometry with the direct input staged
    /// in it. The two live and die together: a rebind keeps both, so the
    /// rebound surface publishes a staged page with its evidence.
    pub(super) fn retire_occurrence_geometry_surface(
        &mut self,
        surface: UiSemanticSurfaceIdentity,
    ) {
        self.occurrence_geometry.retire_surface(surface);
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
        // Even two inputs with equal geometry have distinct consequences. An
        // older prepared frame must never acknowledge the newer pending one.
        // A chrome-only or equal-pose succession still needs an ordinary
        // attempt to acknowledge its exact cause. Moving content otherwise
        // invalidates only its descendants, never the stationary owner.
        self.stage_scroll_poses(
            prepared.iter().filter_map(|record| record.pose()),
            prepared.iter().map(|record| record.occurrence()),
        )?;
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
