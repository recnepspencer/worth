//! Reconcile an existing Scroll track against an accepted mounted extent.
//! This retains its lifecycle identity and does not create a second proposal.

impl super::UiMotionRuntimeState {
    pub(crate) fn committed_scroll_targets(&self) -> Vec<super::UiMotionTargetIdentity> {
        self.tracks
            .keys()
            .copied()
            .filter(|target| target.scope() == super::UiMotionTargetScope::ScrollContents)
            .collect()
    }

    pub(crate) fn reconcile_presented_scroll_extent(
        &mut self,
        request: super::UiMotionTransitionRequest,
        publication: &crate::mounting::UiMountedFramePublicationReceipt,
    ) -> Option<super::UiMotionCommitReceipt> {
        let target = request.successor().target();
        if target.scope() != super::UiMotionTargetScope::ScrollContents
            || publication
                .presentation_for_surface(target.semantic_surface())
                .map(|displayed| displayed.basis())
                != Some(request.successor().presentation())
        {
            return None;
        }
        let previous = self.tracks.get(&target).copied()?;
        if previous.successor_presentation().binding()
            != request.successor().presentation().binding()
        {
            return None;
        }
        let track = previous.with_presented_extent(request);
        let fact = self.publish(
            track.identity(),
            request,
            super::UiMotionProducedFactKind::Retargeted(track.retarget().expect("extent retarget")),
        );
        self.tracks.insert(target, track);
        Some(super::UiMotionCommitReceipt::new(track, fact, None, None))
    }
}
