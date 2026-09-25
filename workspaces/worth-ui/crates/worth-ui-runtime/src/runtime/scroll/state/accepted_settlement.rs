//! Bringing a Scroll owner's semantic offset to the accepted sample that a
//! settling Motion track has already put on screen.
//!
//! Displayed truth belongs to the sample a witness displayed, and published
//! truth belongs here. They are separate, and this is the one place they are
//! reconciled: the offset the owner holds moves by exactly the distance to the
//! offset the witness proved on screen. It is a route, not an assignment, so the same bounds
//! that clamp any reader gesture clamp this one, and the owner's pending settle
//! target is reclamped against those same bounds in the same call -- a settle
//! whose target has left the bounds cannot survive the frame that says so.

impl super::UiScrollRuntimeState {
    /// Route `owner` to the displayed offset under `bounds`, reclamping the
    /// owner's pending settle target against the same bounds. This consumes
    /// the displayed offset: it is where published truth follows it.
    pub(crate) fn settle_accepted_sample(
        &mut self,
        entry: super::super::UiScrollChainEntry,
        displayed: crate::mounting::presentation::UiDisplayedScrollOffset,
        bounds: super::super::UiScrollBounds,
    ) -> Result<super::super::UiScrollRouteReceipt, super::super::UiScrollRouteDenial> {
        let accepted = displayed.settled();
        let current = self.offset(entry.owner(), entry.incarnation())?;
        let inline = accepted
            .inline_subpixels()
            .checked_sub(current.inline_subpixels())
            .ok_or(super::super::UiScrollRouteDenial::OffsetDeltaOutOfRange)?;
        let block = accepted
            .block_subpixels()
            .checked_sub(current.block_subpixels())
            .ok_or(super::super::UiScrollRouteDenial::OffsetDeltaOutOfRange)?;
        let request = super::super::UiScrollDeltaRequest::new(
            vec![entry],
            super::super::UiScrollDelta::new(inline, block),
            super::super::UiScrollDeltaCause::AcceptedSampleSettlement,
        )?;
        self.route_with_reconciled_bounds(request, &[bounds])
    }
}
