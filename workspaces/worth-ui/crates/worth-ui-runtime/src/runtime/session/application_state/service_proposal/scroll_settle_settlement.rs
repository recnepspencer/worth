//! Settling a staged Scroll settle batch against the published frame, and
//! releasing one that never got that far.
//!
//! The Motion commit is the whole point of the lane, so it runs inside the
//! settlement the way the portal lane runs it: the publication receipt is
//! minted `Accepted` from the compiled batch, the settlement opens against it,
//! and the Motion owner installs its track. Every reason the commit could
//! refuse -- a frame that is not the prepared frame, a publication naming
//! another proposal, a rejected disposition -- is decided by values this
//! function itself just produced, so a refusal here is a broken invariant of
//! the lane and not a condition a caller could answer.

impl super::super::WorthUiApplicationSessionState {
    pub(super) fn settle_published_scroll_settle_proposal(
        &mut self,
        batch: crate::runtime::session::service_proposal::UiServiceProposalStagedBatch,
        scroll: &crate::runtime::scroll::UiStagedScrollSettleProposal,
        derived: crate::runtime::motion::UiDerivedMotionServiceProposal,
        mounted: &crate::mounting::UiMountedFramePublicationReceipt,
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis,
        motion_state: &mut crate::runtime::motion::UiMotionRuntimeState,
    ) -> Result<crate::runtime::motion::UiMotionCommitReceipt, super::UiScrollSettlePublicationDenial>
    {
        let publication = crate::runtime::session::service_proposal::UiServiceProposalPublicationReceipt::from_staged_batch(
            &batch,
            crate::runtime::session::service_proposal::UiServiceProposalPublicationDisposition::Accepted,
        );
        let motion_scope = derived.scope();
        let mut settlement = match self
            .runtime
            .service_proposals
            .begin_settlement(batch, publication)
        {
            Ok(settlement) => settlement,
            Err((batch, denial)) => {
                motion_state.discard_derived(derived);
                let teardown = self.runtime.service_proposals.cancel_staged(batch);
                self.finish_scroll_settle_teardown(
                    teardown,
                    scroll.scope(),
                    motion_scope,
                    crate::runtime::session::service_proposal::UiServiceProposalTerminalReason::CancelledBeforePublication,
                );
                return Err(super::UiScrollSettlePublicationDenial::Publication(denial));
            }
        };
        let commit = motion_state
            .commit_published(derived, publication, mounted.frame(), presentation)
            .unwrap_or_else(|(_, denial)| {
                panic!("a settle derived against the published frame must commit: {denial:?}")
            });
        for (family, scope) in [
            (
                crate::capability::UiRuntimeServiceFamily::Scroll,
                scroll.scope(),
            ),
            (
                crate::capability::UiRuntimeServiceFamily::Motion,
                motion_scope,
            ),
        ] {
            let acknowledgement = crate::runtime::session::service_proposal::UiServiceProposalOwnerAcknowledgement::from_family_owner(
                publication,
                family,
                scope,
            );
            self.runtime
                .service_proposals
                .acknowledge_owner(&mut settlement, acknowledgement)
                .expect("the exact settle owner acknowledgement matches its publication");
        }
        self.runtime
            .service_proposals
            .finish_settlement(settlement)
            .expect("both settle owners complete the proposal settlement");
        Ok(commit)
    }

    /// Release a settle that never reached a compiled batch. Both owners are
    /// acknowledged terminally so no occupancy outlives the refusal.
    pub(super) fn cancel_scroll_settle_staging(
        &mut self,
        staging: crate::runtime::session::service_proposal::UiServiceProposalStaging,
        scroll: &crate::runtime::scroll::UiStagedScrollSettleProposal,
        motion_scope: crate::runtime::session::service_proposal::UiServiceProposalOccupancyScopeIdentity,
    ) {
        let teardown = self.runtime.service_proposals.cancel_staging(staging);
        self.finish_scroll_settle_teardown(
            teardown,
            scroll.scope(),
            motion_scope,
            crate::runtime::session::service_proposal::UiServiceProposalTerminalReason::CancelledBeforePublication,
        );
    }

    fn finish_scroll_settle_teardown(
        &mut self,
        mut teardown: crate::runtime::session::service_proposal::UiServiceProposalTeardown,
        scroll_scope: crate::runtime::session::service_proposal::UiServiceProposalOccupancyScopeIdentity,
        motion_scope: crate::runtime::session::service_proposal::UiServiceProposalOccupancyScopeIdentity,
        reason: crate::runtime::session::service_proposal::UiServiceProposalTerminalReason,
    ) {
        for (family, scope) in [
            (
                crate::capability::UiRuntimeServiceFamily::Scroll,
                scroll_scope,
            ),
            (
                crate::capability::UiRuntimeServiceFamily::Motion,
                motion_scope,
            ),
        ] {
            let outcome = crate::runtime::session::service_proposal::UiServiceProposalTerminalOwnerOutcome::from_family_owner(
                teardown.proposal(),
                family,
                scope,
                reason,
            );
            self.runtime
                .service_proposals
                .acknowledge_terminal_owner(&mut teardown, outcome)
                .expect("the exact settle teardown matches its owner");
        }
        self.runtime
            .service_proposals
            .finish_teardown(teardown)
            .expect("the exact settle teardown releases every compiler resource");
    }
}
