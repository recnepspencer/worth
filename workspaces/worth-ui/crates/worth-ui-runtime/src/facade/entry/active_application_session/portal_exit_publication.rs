use super::portal_exit_retention::UiPortalExitTerminalPending;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::facade::entry) enum UiPortalExitTerminalProgress {
    Idle,
    Published,
    Retry,
    AwaitingPhysical,
}

enum UiNormalizedPortalExitTerminalOutcome {
    Published,
    InFlight(super::super::portal_dismissal::DetachedUiPortalDismissalInFlight),
    Indeterminate(super::super::portal_dismissal::DetachedUiPortalDismissalIndeterminate),
    Stopped,
}

impl super::WorthUiActiveApplicationSession {
    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn progress_portal_exit_terminal_for_certification(
        &mut self,
        now_tick: u64,
    ) -> crate::certification_support::UiPortalExitTerminalCertificationOutcome {
        use crate::certification_support::UiPortalExitTerminalCertificationOutcome as Outcome;
        match self.progress_portal_exit_terminal(now_tick) {
            UiPortalExitTerminalProgress::Idle => Outcome::Idle,
            UiPortalExitTerminalProgress::Published => Outcome::Published,
            UiPortalExitTerminalProgress::Retry => Outcome::Retry,
            UiPortalExitTerminalProgress::AwaitingPhysical => Outcome::AwaitingPhysical,
        }
    }

    pub(super) fn shutdown_portal_exit_retention(&mut self) {
        let pending = self.portal_exit_retention.take_pending();
        match pending {
            Some(UiPortalExitTerminalPending::Retry(_)) | None => {}
            Some(UiPortalExitTerminalPending::InFlight { track, completion }) => {
                match normalize(completion.cancel(self)) {
                    UiNormalizedPortalExitTerminalOutcome::Published => {
                        let retention = self
                            .portal_exit_retention
                            .remove(track)
                            .expect("published shutdown closure retains exact coordination");
                        assert!(self
                            .motion
                            .as_mut()
                            .expect("retained exit track retains Motion installation")
                            .release_exit_retention(retention.motion()));
                    }
                    UiNormalizedPortalExitTerminalOutcome::Indeterminate(recovery) => {
                        let (frame, proposal) = recovery.into_parts();
                        drop(frame);
                        self.application
                            .abandon_indeterminate_portal_service_proposal_for_shutdown(
                                proposal,
                                self.focus
                                    .as_mut()
                                    .expect("indeterminate exit retains Focus installation"),
                                self.motion
                                    .as_mut()
                                    .expect("indeterminate exit retains Motion installation"),
                            );
                    }
                    UiNormalizedPortalExitTerminalOutcome::Stopped => {}
                    UiNormalizedPortalExitTerminalOutcome::InFlight(_) => {
                        unreachable!("mounted cancellation cannot remain in flight")
                    }
                }
            }
            Some(UiPortalExitTerminalPending::Indeterminate { recovery, .. }) => {
                let (frame, proposal) = recovery.into_parts();
                drop(frame);
                self.application
                    .abandon_indeterminate_portal_service_proposal_for_shutdown(
                        proposal,
                        self.focus
                            .as_mut()
                            .expect("indeterminate exit retains Focus installation"),
                        self.motion
                            .as_mut()
                            .expect("indeterminate exit retains Motion installation"),
                    );
            }
            Some(UiPortalExitTerminalPending::Reconstruction {
                proposal,
                in_flight,
                ..
            }) => {
                drop(self.cancel_mounted_presentation(in_flight));
                self.application
                    .abandon_indeterminate_portal_service_proposal_for_shutdown(
                        proposal,
                        self.focus
                            .as_mut()
                            .expect("indeterminate exit retains Focus installation"),
                        self.motion
                            .as_mut()
                            .expect("indeterminate exit retains Motion installation"),
                    );
            }
        }
        self.portal_exit_retention.clear_for_shutdown();
        debug_assert_eq!(self.portal_exit_retention.len(), 0);
    }

    pub(in crate::facade::entry) fn progress_portal_exit_terminal(
        &mut self,
        now_tick: u64,
    ) -> UiPortalExitTerminalProgress {
        let track = match self.portal_exit_retention.take_pending() {
            Some(UiPortalExitTerminalPending::Retry(track)) => track,
            Some(pending) => {
                self.portal_exit_retention.retain_pending(pending);
                return UiPortalExitTerminalProgress::AwaitingPhysical;
            }
            None => match self.portal_exit_retention.next_terminal() {
                Some(retention) => retention.motion().track(),
                None => return UiPortalExitTerminalProgress::Idle,
            },
        };
        self.begin_portal_exit_terminal(track, now_tick)
    }

    pub(in crate::facade::entry) fn portal_exit_terminal_work_pending(&self) -> bool {
        self.portal_exit_retention.has_terminal_work()
    }

    pub(in crate::facade::entry) fn portal_exit_terminal_awaits_physical(&self) -> bool {
        self.portal_exit_retention.awaits_physical_progress()
    }

    pub(in crate::facade::entry) fn pending_portal_exit_terminal_matches_native_physical(
        &self,
        class: worth_ui_host_native::UiNativePhysicalProgressClass,
        presentation: Option<worth_ui_host_native::UiNativePhysicalPresentationCorrelation>,
    ) -> bool {
        self.portal_exit_retention
            .pending()
            .is_some_and(|pending| pending.matches_native_physical(class, presentation))
    }

    pub(in crate::facade::entry) fn complete_portal_exit_terminal_physical(
        &mut self,
        now_tick: u64,
    ) -> UiPortalExitTerminalProgress {
        let Some(pending) = self.portal_exit_retention.take_pending() else {
            return UiPortalExitTerminalProgress::Idle;
        };
        let track = pending.track();
        match pending {
            UiPortalExitTerminalPending::InFlight { completion, .. } => {
                let outcome = normalize(completion.complete(self, now_tick));
                self.settle_portal_exit_terminal_outcome(track, outcome)
            }
            pending => {
                self.portal_exit_retention.retain_pending(pending);
                UiPortalExitTerminalProgress::AwaitingPhysical
            }
        }
    }

    pub(in crate::facade::entry) fn take_portal_exit_terminal_pending(
        &mut self,
    ) -> Option<UiPortalExitTerminalPending> {
        self.portal_exit_retention.take_pending()
    }

    pub(in crate::facade::entry) fn retain_portal_exit_terminal_pending(
        &mut self,
        pending: UiPortalExitTerminalPending,
    ) {
        self.portal_exit_retention.retain_pending(pending);
    }

    pub(in crate::facade::entry) fn settle_portal_exit_reconstruction(
        &mut self,
        track: crate::runtime::motion::UiMotionTrackIdentity,
        proposal: crate::runtime::session::UiIndeterminatePortalProposalTransaction,
        outcome: crate::mounting::UiMountedFrameOutcome,
    ) -> UiPortalExitTerminalProgress {
        match outcome {
            crate::mounting::UiMountedFrameOutcome::InFlight(in_flight) => {
                self.portal_exit_retention.retain_pending(
                    UiPortalExitTerminalPending::Reconstruction {
                        track,
                        proposal,
                        in_flight,
                    },
                );
                UiPortalExitTerminalProgress::AwaitingPhysical
            }
            crate::mounting::UiMountedFrameOutcome::PresentationIndeterminate(frame) => {
                self.portal_exit_retention.retain_pending(
                    UiPortalExitTerminalPending::Indeterminate {
                        track,
                        recovery: super::super::portal_dismissal::DetachedUiPortalDismissalIndeterminate::from_parts(
                            self.session_identity(),
                            frame,
                            proposal,
                        ),
                    },
                );
                UiPortalExitTerminalProgress::AwaitingPhysical
            }
            crate::mounting::UiMountedFrameOutcome::Published(_)
            | crate::mounting::UiMountedFrameOutcome::Unchanged(_)
            | crate::mounting::UiMountedFrameOutcome::Reconciled(_)
            | crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(_)
            | crate::mounting::UiMountedFrameOutcome::RetentionDenied(_)
            | crate::mounting::UiMountedFrameOutcome::AdmissionDenied(_)
            | crate::mounting::UiMountedFrameOutcome::Superseded(_) => {
                self.application
                    .settle_indeterminate_portal_service_proposal_to_predecessor(
                        proposal,
                        self.focus
                            .as_mut()
                            .expect("indeterminate exit retains Focus installation"),
                        self.motion
                            .as_mut()
                            .expect("indeterminate exit retains Motion installation"),
                    );
                self.retain_portal_exit_retry(track)
            }
            crate::mounting::UiMountedFrameOutcome::CompletionDenied(_) => {
                panic!("exact portal exit reconstruction completion became unknown")
            }
        }
    }

    fn begin_portal_exit_terminal(
        &mut self,
        track: crate::runtime::motion::UiMotionTrackIdentity,
        now_tick: u64,
    ) -> UiPortalExitTerminalProgress {
        let Some(retention) = self.portal_exit_retention.get(track) else {
            return UiPortalExitTerminalProgress::Idle;
        };
        let portal_retention = retention.portal();
        let Some(presentation) = self
            .portal
            .as_ref()
            .and_then(|portal| portal.exit_retention_presentation(portal_retention))
        else {
            return self.retain_portal_exit_retry(track);
        };
        let lineage = self.next_portal_service_event_identity;
        self.next_portal_service_event_identity = match lineage.checked_add(1) {
            Some(next) => next,
            None => return self.retain_portal_exit_retry(track),
        };
        let idempotency =
            crate::runtime::intent_execution::UiIntentExecutionIdempotencyIdentity::issued(
                self.session_identity().as_u64(),
                lineage,
            );
        let transition = match self
            .portal
            .as_ref()
            .expect("retained portal exit retains Portal installation")
            .prepare_exit_terminal(portal_retention, idempotency)
        {
            Ok(transition) => transition,
            Err(_) => return self.retain_portal_exit_retry(track),
        };
        let revision = transition.successor_revision();
        let overlays = self
            .portal
            .as_ref()
            .expect("retained portal exit retains Portal installation")
            .mounted_projection_inputs(&transition, false);
        let preparation = match self
            .application
            .begin_portal_exit_terminal_service_proposal(
                transition,
                portal_retention,
                presentation,
                self.active_generation_identity(),
                self.motion
                    .as_mut()
                    .expect("retained portal exit retains Motion installation"),
            ) {
            Ok(preparation) => preparation,
            Err(_) => return self.retain_portal_exit_retry(track),
        };
        let frame = match self.prepare_intent_consequence_frame(
            crate::mounting::UiMountedSemanticContentInput::empty(),
            revision,
            overlays,
        ) {
            Ok(frame) => frame,
            Err(_) => {
                self.application.cancel_portal_service_proposal_preparation(
                    preparation,
                    self.motion
                        .as_mut()
                        .expect("retained portal exit retains Motion installation"),
                );
                return self.retain_portal_exit_retry(track);
            }
        };
        let scroll_incarnation = self.scroll_owner_incarnation();
        let proposal = match self.application.bind_portal_service_proposal_frame(
            preparation,
            &frame,
            &self.mounted,
            self.focus
                .as_mut()
                .expect("retained portal exit retains Focus installation"),
            self.scroll.as_ref(),
            scroll_incarnation,
            self.motion
                .as_mut()
                .expect("retained portal exit retains Motion installation"),
        ) {
            Ok(proposal) => proposal,
            Err(_) => return self.retain_portal_exit_retry(track),
        };
        let outcome = self.present_prepared_portal_frame_internal(
            frame,
            &proposal,
            false,
            worth_ui_host_contract::UiPresentationDeadline::at_tick(u64::MAX),
            now_tick,
        );
        let outcome = normalize(
            super::super::portal_dismissal::finish_detached_portal_proposal(
                self, proposal, outcome,
            ),
        );
        self.settle_portal_exit_terminal_outcome(track, outcome)
    }

    fn settle_portal_exit_terminal_outcome(
        &mut self,
        track: crate::runtime::motion::UiMotionTrackIdentity,
        outcome: UiNormalizedPortalExitTerminalOutcome,
    ) -> UiPortalExitTerminalProgress {
        match outcome {
            UiNormalizedPortalExitTerminalOutcome::Published => {
                let retention = self
                    .portal_exit_retention
                    .remove(track)
                    .expect("published terminal closure retains exact coordination");
                assert!(self
                    .motion
                    .as_mut()
                    .expect("retained exit track retains Motion installation")
                    .release_exit_retention(retention.motion()));
                assert!(self.mounted.retire_terminal_motion_sample(track));
                UiPortalExitTerminalProgress::Published
            }
            UiNormalizedPortalExitTerminalOutcome::InFlight(completion) => {
                self.portal_exit_retention
                    .retain_pending(UiPortalExitTerminalPending::InFlight { track, completion });
                UiPortalExitTerminalProgress::AwaitingPhysical
            }
            UiNormalizedPortalExitTerminalOutcome::Indeterminate(recovery) => {
                self.portal_exit_retention
                    .retain_pending(UiPortalExitTerminalPending::Indeterminate { track, recovery });
                UiPortalExitTerminalProgress::AwaitingPhysical
            }
            UiNormalizedPortalExitTerminalOutcome::Stopped => self.retain_portal_exit_retry(track),
        }
    }

    fn retain_portal_exit_retry(
        &mut self,
        track: crate::runtime::motion::UiMotionTrackIdentity,
    ) -> UiPortalExitTerminalProgress {
        self.portal_exit_retention
            .retain_pending(UiPortalExitTerminalPending::Retry(track));
        UiPortalExitTerminalProgress::Retry
    }
}

fn normalize(
    outcome: super::super::portal_dismissal::UiPortalDismissalPublicationOutcome<'_>,
) -> UiNormalizedPortalExitTerminalOutcome {
    use super::super::portal_dismissal::UiPortalDismissalPublicationOutcome as Outcome;
    match outcome {
        Outcome::Published(_) => UiNormalizedPortalExitTerminalOutcome::Published,
        Outcome::InFlight(completion) => {
            UiNormalizedPortalExitTerminalOutcome::InFlight(completion.detach_for_native())
        }
        Outcome::Indeterminate(recovery) => {
            UiNormalizedPortalExitTerminalOutcome::Indeterminate(recovery.detach_for_native())
        }
        Outcome::Stopped(_)
        | Outcome::IgnoredNoMatchingPortal
        | Outcome::IgnoredInsideTopmostPortal => UiNormalizedPortalExitTerminalOutcome::Stopped,
    }
}
