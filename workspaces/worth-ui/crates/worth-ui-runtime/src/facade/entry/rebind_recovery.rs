use super::WorthUiActiveApplicationSession;

pub(crate) struct WorthUiRebindRecoveryAuthority<'session> {
    session: &'session mut WorthUiActiveApplicationSession,
}

impl<'session> WorthUiRebindRecoveryAuthority<'session> {
    pub(crate) fn from_indeterminate(
        indeterminate: Box<
            crate::facade::WorthUiMountedApplicationReplacementIndeterminate<'session>,
        >,
    ) -> Self {
        Self {
            session: indeterminate.into_session_for_shutdown(),
        }
    }

    pub(crate) fn from_content_indeterminate(
        indeterminate: Box<super::WorthUiMountedContentRebindIndeterminate<'session>>,
    ) -> Self {
        let (session, frame) = indeterminate.into_parts();
        drop(frame);
        Self { session }
    }

    pub(crate) fn rebind_host_surface(
        &mut self,
        binding: crate::mounting::UiSurfaceBindingGeneration,
        mode: worth_ui_host_contract::UiHostSurfacePresentationMode,
        profile: crate::mounting::UiSurfaceBindingProfile,
    ) -> Result<
        crate::mounting::UiSurfaceBindingIdentityView,
        crate::mounting::UiMountedIdentityDenial,
    > {
        self.session.rebind_host_surface(binding, mode, profile)
    }

    pub(crate) fn present_current(
        &mut self,
        replacements: &[crate::mounting::UiMountedSurfaceReconciliationBinding],
        deadline: worth_ui_host_contract::UiPresentationDeadline,
        now_tick: u64,
    ) -> Result<crate::mounting::UiMountedFrameOutcome, crate::mounting::UiMountedIdentityDenial>
    {
        self.session
            .present_current_mounted_frame_for_reconciliation(replacements, deadline, now_tick)
    }

    pub(crate) fn complete(
        &mut self,
        in_flight: crate::mounting::UiMountedPresentationInFlight,
        now_tick: u64,
    ) -> crate::mounting::UiMountedFrameOutcome {
        self.session
            .complete_mounted_presentation(in_flight, now_tick)
    }

    pub(crate) fn cancel(
        &mut self,
        in_flight: crate::mounting::UiMountedPresentationInFlight,
    ) -> crate::mounting::UiMountedFrameOutcome {
        self.session.cancel_mounted_presentation(in_flight)
    }

    pub(crate) fn into_session(self) -> &'session mut WorthUiActiveApplicationSession {
        self.session
    }
}

impl<'session> crate::runtime::rebind::UiRebindRecoveryHandle<'session> {
    pub fn into_session_for_shutdown(self) -> &'session mut WorthUiActiveApplicationSession {
        self.into_recovery_authority_for_shutdown().into_session()
    }
}

impl<'session> crate::runtime::rebind::UiRebindReconciliation<'session> {
    pub fn into_session_for_shutdown(self) -> &'session mut WorthUiActiveApplicationSession {
        self.into_recovery_authority_for_shutdown().into_session()
    }
}

impl<'session> crate::runtime::rebind::UiRebindRecoveryInternalDefect<'session> {
    pub fn into_session_for_shutdown(self) -> &'session mut WorthUiActiveApplicationSession {
        self.into_recovery_authority_for_shutdown().into_session()
    }
}
