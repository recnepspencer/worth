use super::{WorthUiActiveApplicationSession, WorthUiNativeApplicationShell};
use crate::inspection::visual_snapshot::{
    UiClearedVisualOverlayReceipt, UiPendingVisualOverlay, UiPublishedVisualOverlay,
    UiVisualOverlayClearFailure, UiVisualOverlayGrant, UiVisualOverlayIdentity,
    UiVisualOverlayPublicationFailure, UiVisualOverlayTarget,
};

impl WorthUiActiveApplicationSession {
    pub fn show_identity_overlay(
        &mut self,
        grant: &UiVisualOverlayGrant,
        target: UiVisualOverlayTarget,
    ) -> Result<UiPendingVisualOverlay, worth_ui_inspection::UiVisualOverlayDenial> {
        if grant.session() != self.identity || target.session() != self.identity {
            return Err(worth_ui_inspection::UiVisualOverlayDenial::ForeignSession);
        }
        if grant.scope().audience() != self.visual_inspection.policy().audience() {
            return Err(worth_ui_inspection::UiVisualOverlayDenial::ForeignSession);
        }
        match target.relation()? {
            worth_ui_inspection::UiVisualSnapshotRelation::Current => {}
            worth_ui_inspection::UiVisualSnapshotRelation::RetainedPredecessor
            | worth_ui_inspection::UiVisualSnapshotRelation::Historical => {
                return Err(worth_ui_inspection::UiVisualOverlayDenial::Superseded);
            }
        }
        let identity = self.next_visual_overlay_identity;
        self.next_visual_overlay_identity = identity
            .checked_add(1)
            .ok_or(worth_ui_inspection::UiVisualOverlayDenial::CapacityExceeded)?;
        self.visual_overlays
            .register(UiVisualOverlayIdentity::issued_by_runtime(identity), target)
    }

    pub fn present_visual_overlay(
        &mut self,
        pending: UiPendingVisualOverlay,
        deadline_tick: u64,
        now_tick: u64,
    ) -> Result<UiPublishedVisualOverlay, UiVisualOverlayPublicationFailure> {
        let publishing = self.begin_visual_overlay_publication(pending)?;
        let base_frame = publishing.selection.presentation.frame;
        match present_overlay_successor(self, deadline_tick, now_tick) {
            Some(frame) if frame != base_frame => self
                .visual_overlays
                .commit_publication(publishing, frame)
                .map_err(|denial| panic!("presented overlay must remain registered: {denial:?}")),
            _ => Err(UiVisualOverlayPublicationFailure::new(
                worth_ui_inspection::UiVisualOverlayDenial::Presentation,
                self.visual_overlays.rollback_publication(publishing),
            )),
        }
    }

    fn begin_visual_overlay_publication(
        &mut self,
        pending: UiPendingVisualOverlay,
    ) -> Result<
        crate::inspection::visual_snapshot::UiPublishingVisualOverlay,
        UiVisualOverlayPublicationFailure,
    > {
        if pending.session() != self.identity {
            return Err(UiVisualOverlayPublicationFailure::new(
                worth_ui_inspection::UiVisualOverlayDenial::ForeignSession,
                pending,
            ));
        }
        Ok(self.visual_overlays.begin_publication(pending))
    }

    pub fn clear_visual_overlay(
        &mut self,
        published: UiPublishedVisualOverlay,
        deadline_tick: u64,
        now_tick: u64,
    ) -> Result<UiClearedVisualOverlayReceipt, UiVisualOverlayClearFailure> {
        if published.session() != self.identity {
            return Err(UiVisualOverlayClearFailure::new(
                worth_ui_inspection::UiVisualOverlayDenial::ForeignSession,
                published,
            ));
        }
        let clearing = self.visual_overlays.begin_clear(published);
        let published_frame = clearing.published_frame;
        match present_overlay_successor(self, deadline_tick, now_tick) {
            Some(frame) if frame != published_frame => self
                .visual_overlays
                .commit_clear(clearing, frame)
                .map_err(|denial| panic!("cleared overlay must remain registered: {denial:?}")),
            _ => Err(UiVisualOverlayClearFailure::new(
                worth_ui_inspection::UiVisualOverlayDenial::Presentation,
                self.visual_overlays.rollback_clear(clearing),
            )),
        }
    }

    fn begin_visual_overlay_clear(
        &mut self,
        published: UiPublishedVisualOverlay,
    ) -> Result<
        crate::inspection::visual_snapshot::UiClearingVisualOverlay,
        UiVisualOverlayClearFailure,
    > {
        if published.session() != self.identity {
            return Err(UiVisualOverlayClearFailure::new(
                worth_ui_inspection::UiVisualOverlayDenial::ForeignSession,
                published,
            ));
        }
        Ok(self.visual_overlays.begin_clear(published))
    }

    /// The frame request that states this session's current overlay truth: every
    /// Portal it still presents, and the visual overlay it has selected. A bare
    /// `UiMountedFrameRequest` states the opposite - that nothing is overlaid -
    /// so a caller replaces these only to lower a transition away from what the
    /// session currently holds.
    pub fn mounted_frame_request(&self) -> crate::mounting::UiMountedFrameRequest {
        let overlay = self
            .visual_overlays
            .active_selection()
            .map(|(identity, selection)| mounted_overlay_input(identity, selection));
        let (portal_revision, portal_overlays) = self.portal.as_ref().map_or_else(
            || (0, Vec::new()),
            |owner| (owner.revision(), owner.current_mounted_projection_inputs()),
        );
        crate::mounting::UiMountedFrameRequest::all_bound_surfaces()
            .with_portal_overlays(portal_revision, portal_overlays)
            .with_visual_overlay(self.visual_overlays.revision(), overlay)
    }
}

impl WorthUiNativeApplicationShell {
    pub fn show_identity_overlay(
        &mut self,
        grant: &UiVisualOverlayGrant,
        target: UiVisualOverlayTarget,
    ) -> Result<UiPendingVisualOverlay, worth_ui_inspection::UiVisualOverlayDenial> {
        self.session.show_identity_overlay(grant, target)
    }

    pub fn present_visual_overlay(
        &mut self,
        pending: UiPendingVisualOverlay,
        deadline_tick: u64,
        now_tick: u64,
    ) -> Result<UiPublishedVisualOverlay, UiVisualOverlayPublicationFailure> {
        let publishing = self.session.begin_visual_overlay_publication(pending)?;
        let base_frame = publishing.selection.presentation.frame;
        let outcome = self.present_frame(deadline_tick, now_tick);
        match outcome {
            Ok(crate::mounting::UiMountedFrameOutcome::Published(receipt))
                if receipt.frame() != base_frame =>
            {
                self.session
                    .visual_overlays
                    .commit_publication(publishing, receipt.frame())
                    .map_err(|denial| {
                        panic!("presented overlay must remain registered: {denial:?}")
                    })
            }
            _ => Err(UiVisualOverlayPublicationFailure::new(
                worth_ui_inspection::UiVisualOverlayDenial::Presentation,
                self.session
                    .visual_overlays
                    .rollback_publication(publishing),
            )),
        }
    }

    pub fn clear_visual_overlay(
        &mut self,
        published: UiPublishedVisualOverlay,
        deadline_tick: u64,
        now_tick: u64,
    ) -> Result<UiClearedVisualOverlayReceipt, UiVisualOverlayClearFailure> {
        let clearing = self.session.begin_visual_overlay_clear(published)?;
        let published_frame = clearing.published_frame;
        match self.present_frame(deadline_tick, now_tick) {
            Ok(crate::mounting::UiMountedFrameOutcome::Published(receipt))
                if receipt.frame() != published_frame =>
            {
                self.session
                    .visual_overlays
                    .commit_clear(clearing, receipt.frame())
                    .map_err(|denial| panic!("cleared overlay must remain registered: {denial:?}"))
            }
            _ => Err(UiVisualOverlayClearFailure::new(
                worth_ui_inspection::UiVisualOverlayDenial::Presentation,
                self.session.visual_overlays.rollback_clear(clearing),
            )),
        }
    }
}

fn present_overlay_successor(
    session: &mut WorthUiActiveApplicationSession,
    deadline_tick: u64,
    now_tick: u64,
) -> Option<worth_ui_host_contract::UiMountedFrameIdentity> {
    let request = session.mounted_frame_request();
    match session.execute_mounted_frame(
        request,
        worth_ui_host_contract::UiPresentationDeadline::at_tick(deadline_tick),
        now_tick,
        |_| {},
    ) {
        Ok(crate::mounting::UiMountedFrameOutcome::Published(receipt)) => Some(receipt.frame()),
        Ok(_) | Err(_) => None,
    }
}

fn mounted_overlay_input(
    identity: UiVisualOverlayIdentity,
    selection: crate::inspection::visual_snapshot::UiVisualOverlaySelection,
) -> crate::mounting::UiMountedVisualOverlayProjectionInput {
    let region = selection.target_region;
    let target_region = worth_ui_host_contract::UiMountedClientPhysicalRect::from_runtime_mounting(
        region.left(),
        region.top(),
        region.right(),
        region.bottom(),
    )
    .expect("snapshot-bound overlay targets carry nonempty validated regions");
    let coordinate_basis =
        worth_ui_host_contract::UiMountedClientCoordinateBasis::from_runtime_mounting(
            selection.host_coordinate_transform,
        )
        .expect("validated host capture transforms form mounted client coordinate bases");
    crate::mounting::UiMountedVisualOverlayProjectionInput {
        overlay_identity: identity.diagnostic_value(),
        base_snapshot: selection.base_snapshot.diagnostic_value(),
        base_frame: selection.presentation.frame,
        target_receipt: selection.target_receipt,
        surface: selection.presentation.semantic_surface,
        binding: selection.presentation.binding,
        coordinate_basis,
        target_region,
    }
}
