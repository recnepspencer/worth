use super::{
    WorthUiActiveApplicationSession, WorthUiApplicationCutoverDenial,
    WorthUiPendingApplicationCutover, WorthUiPreparedApplicationActivation,
    WorthUiPreparedApplicationCutoverOutcome,
};

mod admission;
mod detached;
mod outcome;
#[path = "mounted/published.rs"]
mod published;
use published::WorthUiPresentedApplicationReplacement;

pub(crate) use outcome::{
    WorthUiDetachedMountedApplicationReplacementInFlight,
    WorthUiDetachedPreparedMountedApplicationReplacement,
};
pub use outcome::{
    WorthUiMountedApplicationReplacementInFlight,
    WorthUiMountedApplicationReplacementIndeterminate, WorthUiMountedApplicationReplacementOutcome,
    WorthUiMountedReplacementAdmissionDenial, WorthUiMountedReplacementCompletionDenial,
    WorthUiMountedReplacementHostRejection, WorthUiMountedReplacementPreparationOutcome,
    WorthUiMountedReplacementRetentionDenial, WorthUiPreparedMountedApplicationReplacement,
};

impl WorthUiActiveApplicationSession {
    pub fn prepare_mounted_replacement(
        &mut self,
        pending: WorthUiPendingApplicationCutover,
        admitted_delta: crate::graph::UiAdmittedAllocationCatalogDelta,
        boundary: crate::runtime::WorthUiFrameBoundary,
        lane_parity_report: Option<crate::runtime::WorthUiLaneParityReport>,
        request: crate::mounting::UiMountedFrameRequest,
    ) -> Result<WorthUiMountedReplacementPreparationOutcome<'_>, WorthUiApplicationCutoverDenial>
    {
        self.prepare_mounted_replacement_with_content(
            pending,
            admitted_delta,
            boundary,
            lane_parity_report,
            crate::mounting::UiMountedSemanticContentInput::empty(),
            request,
        )
    }

    pub(crate) fn prepare_mounted_replacement_with_content(
        &mut self,
        pending: WorthUiPendingApplicationCutover,
        admitted_delta: crate::graph::UiAdmittedAllocationCatalogDelta,
        boundary: crate::runtime::WorthUiFrameBoundary,
        lane_parity_report: Option<crate::runtime::WorthUiLaneParityReport>,
        mut semantic_content: crate::mounting::UiMountedSemanticContentInput,
        request: crate::mounting::UiMountedFrameRequest,
    ) -> Result<WorthUiMountedReplacementPreparationOutcome<'_>, WorthUiApplicationCutoverDenial>
    {
        let active_query_plan = self.application.prepared_authority().query_binding_plan();
        let candidate_query_plan = pending.next_app.prepared_authority().query_binding_plan();
        if active_query_plan != candidate_query_plan {
            semantic_content.require_projection_input_replacement(
                candidate_query_plan.projection_input_count(),
            );
        }
        let candidate_graph = pending.next_app.graph_snapshot().clone();
        let candidate_generation = pending.next_app.generation_identity().clone();
        let prepared = self.prepare_application_cutover(
            pending,
            admitted_delta,
            boundary,
            lane_parity_report,
        )?;
        let application = match prepared {
            WorthUiPreparedApplicationCutoverOutcome::SemanticNoOp(receipt) => {
                return Ok(WorthUiMountedReplacementPreparationOutcome::SemanticNoOp(
                    receipt,
                ));
            }
            WorthUiPreparedApplicationCutoverOutcome::Activation(application) => application,
        };
        let mounted_successor = self
            .mounted
            .prepare_graph_replacement_successor(crate::graph::UiGraphAuthority::new(
                &candidate_graph,
            ))
            .map_err(WorthUiApplicationCutoverDenial::MountedIdentity)?;
        let lifecycle = self.prepare_application_lifecycle(
            &mounted_successor,
            application.candidate_service_policy_plan().portal(),
        );
        let capability_report = self.host_session.capability_report();
        let frame = super::mounted_frame::prepare_candidate_mounted_frame(
            &application,
            &mounted_successor,
            crate::graph::UiGraphAuthority::new(&candidate_graph),
            super::mounted_frame::UiMountedReplacementReuseBasis {
                generation: candidate_generation,
                host_session: self.host_session.identity().as_u64(),
                protocol: self.host_session.protocol(),
                capability_generation: capability_report.observation_generation(),
                capability_profile_digest: capability_report.profile_identity_digest(),
            },
            semantic_content,
            request,
        )
        .map_err(WorthUiApplicationCutoverDenial::MountedFrame)?;
        Ok(WorthUiMountedReplacementPreparationOutcome::Prepared(
            Box::new(WorthUiPreparedMountedApplicationReplacement {
                session: self,
                application,
                mounted_successor,
                frame,
                lifecycle,
            }),
        ))
    }
}

impl<'session> WorthUiPreparedMountedApplicationReplacement<'session> {
    pub fn frame(&self) -> &crate::mounting::UiPreparedMountedFrame {
        &self.frame
    }

    pub(crate) fn detach(self: Box<Self>) -> WorthUiDetachedPreparedMountedApplicationReplacement {
        let Self {
            session,
            application,
            mounted_successor,
            frame,
            lifecycle,
        } = *self;
        WorthUiDetachedPreparedMountedApplicationReplacement {
            session_identity: session.session_identity(),
            application,
            mounted_successor,
            frame,
            lifecycle,
        }
    }

    pub fn present(
        self: Box<Self>,
        deadline: worth_ui_host_contract::UiPresentationDeadline,
        now: u64,
    ) -> WorthUiMountedApplicationReplacementOutcome<'session> {
        self.present_with_publication_tail(deadline, now, |presented| presented.commit_once())
    }

    #[cfg(feature = "certification-support")]
    #[doc(hidden)]
    pub fn present_observing_publication_tail_for_certification(
        self: Box<Self>,
        deadline: worth_ui_host_contract::UiPresentationDeadline,
        now: u64,
        observe: impl FnOnce(&mut dyn FnMut()),
    ) -> WorthUiMountedApplicationReplacementOutcome<'session> {
        self.present_with_publication_tail(deadline, now, |presented| {
            let mut presented = Some(presented);
            let mut outcome = None;
            let mut commit = || {
                outcome = Some(
                    presented
                        .take()
                        .expect("certification observer invokes the tail once")
                        .commit_once(),
                );
            };
            observe(&mut commit);
            assert!(
                presented.is_none(),
                "certification observer must invoke the publication tail"
            );
            outcome.expect("publication-tail observer produces the real outcome")
        })
    }

    fn present_with_publication_tail(
        self: Box<Self>,
        deadline: worth_ui_host_contract::UiPresentationDeadline,
        now: u64,
        publish: impl FnOnce(
            WorthUiPresentedApplicationReplacement<'session>,
        ) -> WorthUiMountedApplicationReplacementOutcome<'session>,
    ) -> WorthUiMountedApplicationReplacementOutcome<'session> {
        let Self {
            session,
            application,
            mounted_successor,
            frame,
            lifecycle,
        } = *self;
        let admitted = match admission::prepare_replacement_presentation(
            admission::WorthUiMountedReplacementAdmissionInput {
                session,
                application,
                mounted_successor,
                frame,
                lifecycle,
            },
            deadline,
            now,
        ) {
            Ok(admitted) => admitted,
            Err(outcome) => return *outcome,
        };
        let admission::WorthUiAdmittedMountedReplacement {
            session,
            application,
            mounted,
            lifecycle,
        } = admitted;
        let outcome =
            session
                .mounted
                .present_graph_replacement(&session.host_session, mounted, now);
        Self::finish(session, application, lifecycle, outcome, publish)
    }

    fn finish(
        session: &'session mut WorthUiActiveApplicationSession,
        application: Box<WorthUiPreparedApplicationActivation>,
        lifecycle: super::portal_lifecycle::WorthUiPreparedApplicationLifecycle,
        outcome: crate::mounting::UiMountedGraphReplacementPresentation,
        publish: impl FnOnce(
            WorthUiPresentedApplicationReplacement<'session>,
        ) -> WorthUiMountedApplicationReplacementOutcome<'session>,
    ) -> WorthUiMountedApplicationReplacementOutcome<'session> {
        match outcome {
            crate::mounting::UiMountedGraphReplacementPresentation::Published {
                successor,
                receipt,
            } => publish(WorthUiPresentedApplicationReplacement::new(
                session,
                application,
                lifecycle,
                successor,
                receipt,
            )),
            crate::mounting::UiMountedGraphReplacementPresentation::RejectedBeforeEffects {
                successor,
                frame,
                rejections,
                observation,
            } => {
                crate::facade::entry::mounted_publication::record_mounted_observation(
                    &mut session.host_exchange,
                    observation,
                );
                WorthUiMountedApplicationReplacementOutcome::RejectedBeforeEffects(
                    WorthUiMountedReplacementHostRejection {
                        rejections,
                        replacement: Box::new(Self {
                            session,
                            application,
                            mounted_successor: successor,
                            frame,
                            lifecycle,
                        }),
                    },
                )
            }
            crate::mounting::UiMountedGraphReplacementPresentation::InFlight(mounted) => {
                WorthUiMountedApplicationReplacementOutcome::InFlight(Box::new(
                    WorthUiMountedApplicationReplacementInFlight {
                        session,
                        application,
                        mounted,
                        lifecycle,
                    },
                ))
            }
            crate::mounting::UiMountedGraphReplacementPresentation::PresentationIndeterminate {
                frame,
                observation,
            } => {
                crate::facade::entry::mounted_publication::record_mounted_observation(
                    &mut session.host_exchange,
                    observation,
                );
                WorthUiMountedApplicationReplacementOutcome::PresentationIndeterminate(Box::new(
                    WorthUiMountedApplicationReplacementIndeterminate {
                        session,
                        application,
                        frame,
                        lifecycle,
                    },
                ))
            }
        }
    }
}

impl<'session> WorthUiMountedApplicationReplacementInFlight<'session> {
    pub fn attempt(&self) -> worth_ui_host_contract::UiMountedPresentationAttemptIdentity {
        self.mounted.handle().attempt()
    }

    pub fn deadline(&self) -> worth_ui_host_contract::UiPresentationDeadline {
        self.mounted.handle().deadline()
    }

    pub(crate) fn detach(self: Box<Self>) -> WorthUiDetachedMountedApplicationReplacementInFlight {
        let Self {
            session,
            application,
            mounted,
            lifecycle,
        } = *self;
        WorthUiDetachedMountedApplicationReplacementInFlight {
            session_identity: session.session_identity(),
            application,
            mounted,
            lifecycle,
        }
    }

    pub fn pending_bindings(
        &self,
    ) -> impl ExactSizeIterator<Item = worth_ui_host_contract::UiSurfaceBindingGeneration> + '_
    {
        self.mounted.handle().pending_bindings()
    }

    pub fn cost_report(&self) -> crate::mounting::UiMountCostReport {
        self.mounted.handle().cost_report()
    }

    pub fn complete(
        self: Box<Self>,
        now: u64,
    ) -> WorthUiMountedApplicationReplacementOutcome<'session> {
        let Self {
            session,
            application,
            mounted,
            lifecycle,
        } = *self;
        let outcome =
            session
                .mounted
                .complete_graph_replacement(&session.host_session, mounted, now);
        let outcome = match outcome {
            Ok(outcome) => outcome,
            Err(rejection) => {
                return WorthUiMountedApplicationReplacementOutcome::CompletionDenied(Box::new(
                    WorthUiMountedReplacementCompletionDenial {
                        denial: rejection.denial,
                        in_flight: WorthUiMountedApplicationReplacementInFlight {
                            session,
                            application,
                            mounted: *rejection.in_flight,
                            lifecycle,
                        },
                    },
                ));
            }
        };
        WorthUiPreparedMountedApplicationReplacement::finish(
            session,
            application,
            lifecycle,
            outcome,
            |presented| presented.commit_once(),
        )
    }

    pub fn cancel(self: Box<Self>) -> WorthUiMountedApplicationReplacementOutcome<'session> {
        let Self {
            session,
            application,
            mounted,
            lifecycle,
        } = *self;
        let outcome = session
            .mounted
            .cancel_graph_replacement(&session.host_session, mounted);
        let outcome = match outcome {
            Ok(outcome) => outcome,
            Err(rejection) => {
                return WorthUiMountedApplicationReplacementOutcome::CompletionDenied(Box::new(
                    WorthUiMountedReplacementCompletionDenial {
                        denial: rejection.denial,
                        in_flight: WorthUiMountedApplicationReplacementInFlight {
                            session,
                            application,
                            mounted: *rejection.in_flight,
                            lifecycle,
                        },
                    },
                ));
            }
        };
        WorthUiPreparedMountedApplicationReplacement::finish(
            session,
            application,
            lifecycle,
            outcome,
            |presented| presented.commit_once(),
        )
    }
}
