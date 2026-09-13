use super::completion::finish;
use super::*;

impl UiPortalDismissalPublicationCompletion<'_> {
    pub(in crate::facade::entry) fn detach_for_native(
        mut self,
    ) -> DetachedUiPortalDismissalInFlight {
        let state = self
            .state
            .take()
            .expect("live dismissal completion owns state");
        DetachedUiPortalDismissalInFlight {
            session: state.admitted.session.session_identity(),
            proposal: state
                .admitted
                .proposal
                .expect("live dismissal completion retains proposal"),
            mounted: state.mounted,
            retain_exit: state.admitted.retain_exit,
        }
    }
}

impl DetachedUiPortalDismissalInFlight {
    pub(in crate::facade::entry) const fn session_identity(
        &self,
    ) -> crate::facade::WorthUiActiveApplicationSessionIdentity {
        self.session
    }

    pub(in crate::facade::entry) fn matches_native_progress(
        &self,
        progress: &crate::native_platform::UiNativeApplicationPhysicalProgress,
    ) -> bool {
        self.matches_native_physical(progress.class(), progress.presentation())
    }

    pub(in crate::facade::entry) fn matches_native_physical(
        &self,
        class: worth_ui_host_native::UiNativePhysicalProgressClass,
        presentation: Option<worth_ui_host_native::UiNativePhysicalPresentationCorrelation>,
    ) -> bool {
        let class = match class {
            worth_ui_host_native::UiNativePhysicalProgressClass::Presentation => {
                worth_ui_host_contract::UiHostPresentationProgressClass::PhysicalSurface
            }
            worth_ui_host_native::UiNativePhysicalProgressClass::TextAtlas => {
                worth_ui_host_contract::UiHostPresentationProgressClass::TextAtlas
            }
            worth_ui_host_native::UiNativePhysicalProgressClass::PresentationRecovery => {
                return false;
            }
        };
        self.mounted.awaits_progress_class(class)
            && presentation.is_none_or(|presentation| {
                presentation.attempt() == self.mounted.attempt()
                    && self
                        .mounted
                        .pending_bindings()
                        .any(|binding| binding == presentation.binding())
            })
    }

    pub(in crate::facade::entry) fn complete<'session>(
        self,
        session: &'session mut WorthUiActiveApplicationSession,
        now_tick: u64,
    ) -> UiPortalDismissalPublicationOutcome<'session> {
        let outcome = session.complete_mounted_presentation(self.mounted, now_tick);
        super::completion::finish_presented(
            UiPortalDismissalAdmitted {
                session,
                proposal: Some(self.proposal),
                retain_exit: self.retain_exit,
            },
            outcome,
            now_tick,
        )
    }

    pub(in crate::facade::entry) fn cancel<'session>(
        self,
        session: &'session mut WorthUiActiveApplicationSession,
    ) -> UiPortalDismissalPublicationOutcome<'session> {
        let outcome = session.cancel_mounted_presentation(self.mounted);
        finish(
            UiPortalDismissalAdmitted {
                session,
                proposal: Some(self.proposal),
                retain_exit: self.retain_exit,
            },
            outcome,
        )
    }
}

impl DetachedUiPortalDismissalIndeterminate {
    pub(in crate::facade::entry) fn matches_native_physical(
        &self,
        class: worth_ui_host_native::UiNativePhysicalProgressClass,
        presentation: Option<worth_ui_host_native::UiNativePhysicalPresentationCorrelation>,
    ) -> bool {
        // An exit gets first refusal in the driver. Claim only its own recovery;
        // another pending presentation must receive its completion unchanged.
        matches!(
            class,
            worth_ui_host_native::UiNativePhysicalProgressClass::PresentationRecovery
                | worth_ui_host_native::UiNativePhysicalProgressClass::Presentation
        ) && presentation.is_some_and(|presentation| {
            presentation.attempt() == self.frame.report().attempt()
                && self
                    .frame
                    .report()
                    .physical_recovery_bindings()
                    .contains(&presentation.binding())
        })
    }

    pub(in crate::facade::entry) const fn session_identity(
        &self,
    ) -> crate::facade::WorthUiActiveApplicationSessionIdentity {
        self.session
    }

    pub(in crate::facade::entry) fn into_parts(
        self,
    ) -> (
        crate::mounting::UiMountedIndeterminateFrame,
        crate::runtime::session::UiIndeterminatePortalProposalTransaction,
    ) {
        (self.frame, self.proposal)
    }

    pub(in crate::facade::entry) fn from_parts(
        session: crate::facade::WorthUiActiveApplicationSessionIdentity,
        frame: crate::mounting::UiMountedIndeterminateFrame,
        proposal: crate::runtime::session::UiIndeterminatePortalProposalTransaction,
    ) -> Self {
        Self {
            session,
            frame,
            proposal,
        }
    }
}

impl<'session> UiPortalDismissalPublicationRecovery<'session> {
    pub(in crate::facade::entry) fn detach_for_native(
        mut self,
    ) -> DetachedUiPortalDismissalIndeterminate {
        let state = self
            .state
            .take()
            .expect("live dismissal recovery owns state");
        DetachedUiPortalDismissalIndeterminate {
            session: state.session.session_identity(),
            frame: state.frame,
            proposal: state.proposal,
        }
    }
}

impl Drop for UiPortalDismissalPublicationCompletion<'_> {
    fn drop(&mut self) {
        let Some(state) = self.state.take() else {
            return;
        };
        let outcome = state
            .admitted
            .session
            .cancel_mounted_presentation(state.mounted);
        drop(finish(state.admitted, outcome));
    }
}

impl Drop for UiPortalDismissalPublicationRecovery<'_> {
    fn drop(&mut self) {
        let Some(state) = self.state.take() else {
            return;
        };
        drop(state.frame);
        state
            .session
            .application
            .dispose_indeterminate_portal_service_proposal(
                state.proposal,
                state
                    .session
                    .focus
                    .as_mut()
                    .expect("retained proposal owns Focus"),
                state
                    .session
                    .motion
                    .as_mut()
                    .expect("retained proposal owns Motion"),
            );
    }
}
