use super::{
    WorthUiActiveApplicationSession, WorthUiDetachedMountedApplicationReplacementInFlight,
    WorthUiDetachedPreparedMountedApplicationReplacement,
    WorthUiMountedApplicationReplacementInFlight, WorthUiMountedApplicationReplacementOutcome,
    WorthUiMountedReplacementCompletionDenial, WorthUiPreparedMountedApplicationReplacement,
};

impl WorthUiDetachedPreparedMountedApplicationReplacement {
    pub(crate) const fn session_identity(
        &self,
    ) -> crate::facade::WorthUiActiveApplicationSessionIdentity {
        self.session_identity
    }

    pub(crate) fn attach<'session>(
        self,
        session: &'session mut WorthUiActiveApplicationSession,
    ) -> Box<WorthUiPreparedMountedApplicationReplacement<'session>> {
        Box::new(WorthUiPreparedMountedApplicationReplacement {
            session,
            application: self.application,
            mounted_successor: self.mounted_successor,
            frame: self.frame,
            lifecycle: self.lifecycle,
        })
    }
}

impl WorthUiDetachedMountedApplicationReplacementInFlight {
    pub(crate) fn session_identity(
        &self,
    ) -> crate::facade::WorthUiActiveApplicationSessionIdentity {
        self.session_identity
    }

    pub(crate) fn attempt(&self) -> worth_ui_host_contract::UiMountedPresentationAttemptIdentity {
        self.mounted.handle().attempt()
    }

    pub(crate) fn awaits_progress_class(
        &self,
        class: worth_ui_host_contract::UiHostPresentationProgressClass,
    ) -> bool {
        self.mounted.handle().awaits_progress_class(class)
    }

    pub(crate) fn pending_bindings(
        &self,
    ) -> impl ExactSizeIterator<Item = worth_ui_host_contract::UiSurfaceBindingGeneration> + '_
    {
        self.mounted.handle().pending_bindings()
    }

    pub(crate) fn complete<'session>(
        self,
        session: &'session mut WorthUiActiveApplicationSession,
        now: u64,
    ) -> WorthUiMountedApplicationReplacementOutcome<'session> {
        let Self {
            application,
            mounted,
            lifecycle,
            ..
        } = self;
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

    pub(crate) fn cancel<'session>(
        self,
        session: &'session mut WorthUiActiveApplicationSession,
    ) -> WorthUiMountedApplicationReplacementOutcome<'session> {
        let Self {
            application,
            mounted,
            lifecycle,
            ..
        } = self;
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
