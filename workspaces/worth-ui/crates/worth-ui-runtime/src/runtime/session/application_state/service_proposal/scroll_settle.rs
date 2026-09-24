//! Publishing one prepared Scroll settle transition into Motion.
//!
//! The portal lane mints a successor frame and settles its proposal against it.
//! A settle mints nothing: a wheel notch changes no mounted content, so the
//! frame the transition was prepared against is the frame already published.
//! The compiler names that stage `SubmitToExistingPublication`, and this lane
//! runs the whole proposal against it in one call -- reservation, staging,
//! derivation, commit and settlement -- so the occupancy this settle holds is
//! released before the next notch of the same owner asks for it. A settle that
//! held its lease across frames would coalesce its own successor, and the
//! mid-flight retarget could never reach the live track.

use super::super::WorthUiApplicationSessionState;
use super::UiScrollSettlePublicationDenial;

impl WorthUiApplicationSessionState {
    /// Publish `transition` into Motion against the currently published frame,
    /// returning the committed track the mounted sampler installs.
    pub(crate) fn publish_scroll_settle_service_proposal(
        &mut self,
        transition: &crate::runtime::scroll::UiPreparedScrollSettleTransition,
        mounted: &crate::mounting::UiMountedFramePublicationReceipt,
        application: crate::runtime::intent::WorthUiActiveApplicationGenerationIdentity,
        motion_state: &mut crate::runtime::motion::UiMotionRuntimeState,
    ) -> Result<crate::runtime::motion::UiMotionCommitReceipt, UiScrollSettlePublicationDenial>
    {
        let presentation = mounted
            .presentation_for_surface(transition.semantic_surface())
            .map(|displayed| displayed.basis())
            .ok_or(UiScrollSettlePublicationDenial::UnpublishedSurface)?;
        let request = crate::runtime::session::service_proposal::UiServiceRequestBasis::<
            crate::runtime::session::service_proposal::UiScrollSettleServiceRequestAuthority,
        >::from_scroll_settle(transition, presentation, application)
        .map_err(UiScrollSettlePublicationDenial::RequestBasis)?;
        let coherence = request.coherence();
        let motion_request = transition.motion_request();
        let scroll_family = crate::runtime::scroll::UiStagedScrollSettleProposal::family_proposal(
            transition.scope(),
        );
        let motion_family =
            crate::runtime::motion::UiStagedMotionServiceProposal::family_proposal(&motion_request);
        let candidate =
            crate::runtime::session::service_proposal::UiServiceProposalCandidate::from_request(
                &request,
                vec![scroll_family, motion_family],
            )
            .map_err(UiScrollSettlePublicationDenial::Demand)?;
        let proposal = candidate.identity();
        let support = self.app.runtime_service_support();
        let preflighted = self
            .runtime
            .service_proposals
            .preflight(candidate, &coherence, support)
            .map_err(UiScrollSettlePublicationDenial::Preflight)?;
        let reservation = match self
            .runtime
            .service_proposals
            .reserve(preflighted)
            .map_err(UiScrollSettlePublicationDenial::Reservation)?
        {
            crate::runtime::session::service_proposal::UiServiceProposalReservationOutcome::Reserved(
                reservation,
            ) => reservation,
            crate::runtime::session::service_proposal::UiServiceProposalReservationOutcome::Coalesced { incumbent } => {
                return Err(UiScrollSettlePublicationDenial::Coalesced(incumbent))
            }
        };
        let scroll = crate::runtime::scroll::UiStagedScrollSettleProposal::prepare(
            proposal,
            transition.scope(),
        );
        let motion = match motion_state.stage(proposal, motion_request) {
            Ok(motion) => motion,
            Err(denial) => {
                self.runtime
                    .service_proposals
                    .cancel_before_effect(reservation)
                    .expect("a reserved settle stays cancellable before owner staging");
                return Err(UiScrollSettlePublicationDenial::MotionStaging(denial));
            }
        };
        let staging = match self.runtime.service_proposals.begin_staging(reservation) {
            Ok(staging) => staging,
            Err((reservation, denial)) => {
                motion_state.discard_staged(motion);
                // The reservation never entered staging, so its occupancy and
                // cancellation records are released here rather than leaked.
                let _ = self
                    .runtime
                    .service_proposals
                    .shutdown_reservation(reservation);
                return Err(UiScrollSettlePublicationDenial::Staging(denial));
            }
        };
        let (batch, scroll, derived) =
            self.stage_scroll_settle_proposal(staging, scroll, motion, mounted, motion_state)?;
        self.settle_published_scroll_settle_proposal(
            batch,
            &scroll,
            derived,
            mounted,
            presentation,
            motion_state,
        )
    }

    /// Carry the reserved settle through every owner stage the compiler asks
    /// for: the two family receipts, the existing-publication assembly, and the
    /// Motion derivation against the frame already on screen.
    fn stage_scroll_settle_proposal(
        &mut self,
        mut staging: crate::runtime::session::service_proposal::UiServiceProposalStaging,
        scroll: crate::runtime::scroll::UiStagedScrollSettleProposal,
        motion: crate::runtime::motion::UiStagedMotionServiceProposal,
        mounted: &crate::mounting::UiMountedFramePublicationReceipt,
        motion_state: &mut crate::runtime::motion::UiMotionRuntimeState,
    ) -> Result<
        (
            crate::runtime::session::service_proposal::UiServiceProposalStagedBatch,
            crate::runtime::scroll::UiStagedScrollSettleProposal,
            crate::runtime::motion::UiDerivedMotionServiceProposal,
        ),
        UiScrollSettlePublicationDenial,
    > {
        let proposal = scroll.proposal();
        let motion_scope = motion.scope();
        for receipt in [
            scroll.family_stage_receipt(),
            motion.family_stage_receipt(),
            crate::runtime::session::service_proposal::UiServiceProposalStageReceipt::existing_preparation(
                proposal,
            ),
        ] {
            if let Err(denial) = self
                .runtime
                .service_proposals
                .advance_staging(&mut staging, receipt)
            {
                motion_state.discard_staged(motion);
                self.cancel_scroll_settle_staging(staging, &scroll, motion_scope);
                return Err(UiScrollSettlePublicationDenial::Staging(denial));
            }
        }
        let derived = motion_state.derive(motion, mounted.frame());
        if let Err(denial) = self
            .runtime
            .service_proposals
            .advance_staging(&mut staging, derived.derivation_receipt())
        {
            motion_state.discard_derived(derived);
            self.cancel_scroll_settle_staging(staging, &scroll, motion_scope);
            return Err(UiScrollSettlePublicationDenial::Staging(denial));
        }
        match self.runtime.service_proposals.finish_staging(staging) {
            Ok(batch) => Ok((batch, scroll, derived)),
            Err((staging, denial)) => {
                motion_state.discard_derived(derived);
                self.cancel_scroll_settle_staging(staging, &scroll, motion_scope);
                Err(UiScrollSettlePublicationDenial::Staging(denial))
            }
        }
    }
}
