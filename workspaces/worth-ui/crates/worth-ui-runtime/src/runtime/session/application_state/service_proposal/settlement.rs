impl super::WorthUiApplicationSessionState {
    fn begin_portal_proposal_settlement(
        &mut self,
        transaction: super::UiStagedPortalProposalTransaction,
        publication: crate::runtime::session::service_proposal::UiServiceProposalPublicationReceipt,
        focus_state: &mut crate::runtime::focus::UiFocusRuntimeState,
        motion_state: &mut crate::runtime::motion::UiMotionRuntimeState,
    ) -> Result<super::UiPortalProposalSettlement, super::UiPortalProposalPreparationDenial> {
        let scope = transaction.portal.scope();
        let settlement = match self
            .runtime
            .service_proposals
            .begin_settlement(transaction.batch, publication)
        {
            Ok(settlement) => settlement,
            Err((batch, denial)) => {
                focus_state
                    .discard_portal_proposal(transaction.focus.proposal())
                    .expect("malformed publication still names its staged Focus owner");
                let motion_scope = transaction.motion.as_ref().map(|motion| motion.scope());
                if let Some(motion) = transaction.motion {
                    motion_state.discard_derived(motion);
                }
                let teardown = self.runtime.service_proposals.cancel_staged(batch);
                self.finish_portal_teardown(
                    teardown,
                    &transaction.portal,
                    &transaction.focus,
                    &transaction.scroll,
                    transaction.selection.as_ref(),
                    motion_scope,
                    crate::runtime::session::service_proposal::UiServiceProposalTerminalReason::CancelledBeforePublication,
                );
                return Err(super::UiPortalProposalPreparationDenial::Publication(
                    denial,
                ));
            }
        };
        Ok(super::UiPortalProposalSettlement {
            settlement,
            transition: transaction.portal.into_transition(),
            focus: transaction.focus,
            scroll: transaction.scroll,
            staged_reveal: transaction.staged_reveal,
            selection: transaction.selection,
            motion: transaction.motion,
            prepared_frame: transaction.prepared_frame,
            publication,
            scope,
        })
    }

    fn finish_portal_proposal_settlement(
        &mut self,
        mut settlement: crate::runtime::session::service_proposal::UiServiceProposalSettlement,
        publication: crate::runtime::session::service_proposal::UiServiceProposalPublicationReceipt,
        scope: crate::runtime::session::service_proposal::UiServiceProposalOccupancyScopeIdentity,
        focus_scope: crate::runtime::session::service_proposal::UiServiceProposalOccupancyScopeIdentity,
        scroll_scope: crate::runtime::session::service_proposal::UiServiceProposalOccupancyScopeIdentity,
        selection: Option<&crate::runtime::selection::UiStagedDeclaredSelectionTransition>,
        motion_scope: Option<
            crate::runtime::session::service_proposal::UiServiceProposalOccupancyScopeIdentity,
        >,
    ) {
        let acknowledgement = crate::runtime::session::service_proposal::UiServiceProposalOwnerAcknowledgement::from_family_owner(
            publication,
            crate::capability::UiRuntimeServiceFamily::Portal,
            scope,
        );
        self.runtime
            .service_proposals
            .acknowledge_owner(&mut settlement, acknowledgement)
            .expect("exact portal owner acknowledgement matches its publication");
        let focus_acknowledgement = crate::runtime::session::service_proposal::UiServiceProposalOwnerAcknowledgement::from_family_owner(
            publication,
            crate::capability::UiRuntimeServiceFamily::Focus,
            focus_scope,
        );
        self.runtime
            .service_proposals
            .acknowledge_owner(&mut settlement, focus_acknowledgement)
            .expect("exact Focus owner acknowledgement matches its publication");
        let scroll_acknowledgement = crate::runtime::session::service_proposal::UiServiceProposalOwnerAcknowledgement::from_family_owner(
            publication,
            crate::capability::UiRuntimeServiceFamily::Scroll,
            scroll_scope,
        );
        self.runtime
            .service_proposals
            .acknowledge_owner(&mut settlement, scroll_acknowledgement)
            .expect("exact Scroll owner acknowledgement matches its publication");
        if let Some(selection) = selection {
            self.runtime
                .service_proposals
                .acknowledge_owner(
                    &mut settlement,
                    selection.settlement_acknowledgement(publication),
                )
                .expect("exact Selection owner acknowledgement matches its publication");
        }
        if let Some(motion_scope) = motion_scope {
            let motion_acknowledgement = crate::runtime::session::service_proposal::UiServiceProposalOwnerAcknowledgement::from_family_owner(
                publication,
                crate::capability::UiRuntimeServiceFamily::Motion,
                motion_scope,
            );
            self.runtime
                .service_proposals
                .acknowledge_owner(&mut settlement, motion_acknowledgement)
                .expect("exact Motion owner acknowledgement matches its publication");
        }
        self.runtime
            .service_proposals
            .finish_settlement(settlement)
            .expect("exact portal owner completes proposal settlement");
    }

    pub(crate) fn settle_published_portal_service_proposal(
        &mut self,
        transaction: super::UiStagedPortalProposalTransaction,
        mounted: &crate::mounting::UiMountedFramePublicationReceipt,
        portal: &mut crate::runtime::portal::UiPortalRuntimeState,
        focus: &mut crate::runtime::focus::UiFocusRuntimeState,
        selection_state: Option<&mut crate::runtime::selection::UiSelectionRuntimeState>,
        motion_state: &mut crate::runtime::motion::UiMotionRuntimeState,
    ) -> Result<super::UiPublishedPortalSettlement, super::UiPortalProposalPreparationDenial> {
        if !transaction.reveal_refinement_agrees() {
            return Err(super::UiPortalProposalPreparationDenial::RevealRefinementMismatch);
        }
        let publication = transaction.accepted_publication(mounted)?;
        let settlement =
            self.begin_portal_proposal_settlement(transaction, publication, focus, motion_state)?;
        let (
            settlement,
            transition,
            focus_owner,
            scroll_owner,
            staged_reveal,
            selection_owner,
            motion_owner,
            frame,
            publication,
            scope,
        ) = settlement.into_parts();
        focus
            .validate_portal_proposal(focus_owner.proposal(), frame)
            .expect("exclusive Focus proposal retains exact prepared successor");
        portal
            .validate_prepared(&transition)
            .expect("exclusive portal proposal retains exact portal revision");
        let motion_scope = motion_owner.as_ref().map(|motion| motion.scope());
        let motion_presentation = motion_owner.as_ref().map(|_| {
            mounted
                .presentation_for_surface(transition.request().semantic_surface())
                .map(|displayed| displayed.basis())
                .expect("published portal surface retains its exact host presentation basis")
        });
        let motion_commit = motion_owner.map(|motion| {
            motion_state
                .commit_published(
                    motion,
                    publication,
                    frame,
                    motion_presentation.expect("Motion owner has one presentation basis"),
                )
                .unwrap_or_else(|(_, denial)| {
                    panic!("validated Motion proposal must commit: {denial:?}")
                })
        });
        let focus_transition = focus
            .commit_portal_proposal(focus_owner.proposal(), frame)
            .expect("exclusive Focus proposal retains exact prepared successor");
        let motion_exit_retention = motion_commit.and_then(|commit| commit.exit_retention());
        let (_, portal_exit_retention) = portal
            .commit_published_with_exit_retention(transition, motion_exit_retention.is_some())
            .expect("exclusive portal proposal retains exact portal revision");
        // Portal mints a retention exactly when it commits `Closing`, and Motion
        // exactly for an `ExitRetention` fill. `prepare_portal_motion_request`
        // emits no request for an idempotent transition, and `portal_exit` is the
        // only declaration with that fill (both pinned by focused tests), so the
        // two decisions cannot disagree.
        let exit_retention = match (portal_exit_retention, motion_exit_retention) {
            (Some(portal), Some(motion)) => Some((portal, motion)),
            (None, None) => None,
            _ => unreachable!(
                "Portal `Closing` and Motion `ExitRetention` are one decision per transition"
            ),
        };
        self.finish_portal_proposal_settlement(
            settlement,
            publication,
            scope,
            focus_owner.scope(),
            scroll_owner.scope(),
            selection_owner.as_ref(),
            motion_scope,
        );
        if let Some(selection_owner) = selection_owner {
            selection_owner.commit(
                selection_state.expect("a staged Selection proposal retains its installed owner"),
            );
        }
        // The staged reveal leaves with the settlement: it lands as a direct
        // placement with the frame that carries it, not on this one.
        Ok(super::UiPublishedPortalSettlement {
            focus: focus_transition,
            motion: motion_commit,
            exit_retention,
            reveal: staged_reveal,
        })
    }

    pub(crate) fn settle_indeterminate_portal_service_proposal(
        &mut self,
        transaction: super::UiStagedPortalProposalTransaction,
        portal: &mut crate::runtime::portal::UiPortalRuntimeState,
        focus: &mut crate::runtime::focus::UiFocusRuntimeState,
        _motion_state: &mut crate::runtime::motion::UiMotionRuntimeState,
    ) -> Result<
        super::UiIndeterminatePortalProposalTransaction,
        super::UiPortalProposalPreparationDenial,
    > {
        focus
            .validate_portal_proposal(transaction.focus.proposal(), transaction.prepared_frame)
            .expect("indeterminate physical work retains exact semantic Focus successor");
        portal
            .validate_prepared(transaction.portal.transition())
            .expect("exclusive portal proposal retains exact portal revision");
        Ok(super::UiIndeterminatePortalProposalTransaction { transaction })
    }

    pub(crate) fn settle_indeterminate_portal_service_proposal_to_predecessor(
        &mut self,
        transaction: super::UiIndeterminatePortalProposalTransaction,
        focus: &mut crate::runtime::focus::UiFocusRuntimeState,
        motion_state: &mut crate::runtime::motion::UiMotionRuntimeState,
    ) {
        self.settle_rejected_portal_service_proposal(transaction.transaction, focus, motion_state)
            .expect("predecessor reconstruction rejects the exact retained proposal");
    }

    pub(crate) fn abandon_indeterminate_portal_service_proposal_for_shutdown(
        &mut self,
        transaction: super::UiIndeterminatePortalProposalTransaction,
        focus: &mut crate::runtime::focus::UiFocusRuntimeState,
        motion_state: &mut crate::runtime::motion::UiMotionRuntimeState,
    ) {
        let transaction = transaction.transaction;
        focus
            .discard_portal_proposal(transaction.focus.proposal())
            .expect("shutdown discards its exact staged Focus successor");
        let motion_scope = transaction.motion.as_ref().map(|motion| motion.scope());
        if let Some(motion) = transaction.motion {
            motion_state.discard_derived(motion);
        }
        let teardown = self
            .runtime
            .service_proposals
            .shutdown_staged(transaction.batch);
        self.finish_portal_teardown(
            teardown,
            &transaction.portal,
            &transaction.focus,
            &transaction.scroll,
            transaction.selection.as_ref(),
            motion_scope,
            crate::runtime::session::service_proposal::UiServiceProposalTerminalReason::AbandonedAtShutdown,
        );
    }

    pub(crate) fn dispose_indeterminate_portal_service_proposal(
        &mut self,
        transaction: super::UiIndeterminatePortalProposalTransaction,
        focus: &mut crate::runtime::focus::UiFocusRuntimeState,
        motion_state: &mut crate::runtime::motion::UiMotionRuntimeState,
    ) {
        let transaction = transaction.transaction;
        focus
            .discard_portal_proposal(transaction.focus.proposal())
            .expect("recovery disposal discards its exact staged Focus successor");
        let motion_scope = transaction.motion.as_ref().map(|motion| motion.scope());
        if let Some(motion) = transaction.motion {
            motion_state.discard_derived(motion);
        }
        let teardown = self
            .runtime
            .service_proposals
            .dispose_recovery_staged(transaction.batch);
        self.finish_portal_teardown(
            teardown,
            &transaction.portal,
            &transaction.focus,
            &transaction.scroll,
            transaction.selection.as_ref(),
            motion_scope,
            crate::runtime::session::service_proposal::UiServiceProposalTerminalReason::RecoveryDisposed,
        );
    }

    pub(crate) fn settle_rejected_portal_service_proposal(
        &mut self,
        transaction: super::UiStagedPortalProposalTransaction,
        focus: &mut crate::runtime::focus::UiFocusRuntimeState,
        motion_state: &mut crate::runtime::motion::UiMotionRuntimeState,
    ) -> Result<(), super::UiPortalProposalPreparationDenial> {
        let publication = transaction.rejected_publication();
        let settlement =
            self.begin_portal_proposal_settlement(transaction, publication, focus, motion_state)?;
        let (
            settlement,
            transition,
            focus_owner,
            scroll_owner,
            staged_reveal,
            selection_owner,
            motion_owner,
            _frame,
            publication,
            scope,
        ) = settlement.into_parts();
        drop(transition);
        drop(staged_reveal);
        focus
            .discard_portal_proposal(focus_owner.proposal())
            .expect("rejected publication discards its exact Focus successor");
        let motion_scope = motion_owner.as_ref().map(|motion| motion.scope());
        if let Some(motion) = motion_owner {
            motion_state.discard_derived(motion);
        }
        self.finish_portal_proposal_settlement(
            settlement,
            publication,
            scope,
            focus_owner.scope(),
            scroll_owner.scope(),
            selection_owner.as_ref(),
            motion_scope,
        );
        Ok(())
    }
}
