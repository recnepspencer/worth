use super::*;

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
            owners,
        } = *self;
        WorthUiDetachedMountedApplicationReplacementInFlight {
            session_identity: session.session_identity(),
            application,
            mounted,
            lifecycle,
            owners,
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
            owners,
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
                            owners,
                        },
                    },
                ));
            }
        };
        WorthUiPreparedMountedApplicationReplacement::finish(
            session,
            application,
            lifecycle,
            owners,
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
            owners,
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
                            owners,
                        },
                    },
                ));
            }
        };
        WorthUiPreparedMountedApplicationReplacement::finish(
            session,
            application,
            lifecycle,
            owners,
            outcome,
            |presented| presented.commit_once(),
        )
    }
}
