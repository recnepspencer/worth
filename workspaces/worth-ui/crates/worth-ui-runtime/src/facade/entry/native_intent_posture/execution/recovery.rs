use super::*;

impl WorthUiNativeIntentPosturePublicationRecovery<'_> {
    pub fn frame(&self) -> &crate::mounting::UiMountedIndeterminateFrame {
        &self
            .state
            .as_deref()
            .expect("live posture recovery owns its state")
            .frame
    }
}

impl<'session> WorthUiNativeIntentPosturePublicationRecovery<'session> {
    pub(in crate::facade::entry) fn detach_for_native(
        mut self,
    ) -> (
        crate::runtime::rebind::UiDetachedRebindRecovery,
        crate::mounting::UiMountedIndeterminateFrame,
    ) {
        let state = self
            .state
            .take()
            .expect("live posture recovery owns its state");
        let NativeIntentPostureIndeterminate { admitted, frame } = *state;
        let NativeIntentPostureAdmitted {
            session,
            plan,
            reservation,
            transfer,
        } = admitted;
        drop(transfer);
        let recovery = crate::runtime::rebind::UiDetachedRebindRecovery::from_indeterminate_posture(
            session.session_identity(),
            plan,
            reservation,
            &frame,
        );
        (recovery, frame)
    }

    pub fn into_session_for_shutdown(mut self) -> &'session mut WorthUiActiveApplicationSession {
        let state = self
            .state
            .take()
            .expect("live posture recovery owns its state");
        drop((
            state.admitted.plan,
            state.admitted.reservation,
            state.admitted.transfer,
            state.frame,
        ));
        state.admitted.session
    }
}
