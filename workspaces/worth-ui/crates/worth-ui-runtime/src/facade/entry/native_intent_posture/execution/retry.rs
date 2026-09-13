use super::*;

impl WorthUiNativeIntentPosturePublicationRetry<'_> {
    pub fn rejections(&self) -> &[crate::mounting::UiMountedSurfacePresentationRejection] {
        &self
            .state
            .as_deref()
            .expect("live posture retry owns its state")
            .rejections
    }
}

impl<'session> WorthUiNativeIntentPosturePublicationRetry<'session> {
    pub fn retry(
        mut self,
        now_tick: u64,
    ) -> WorthUiNativeIntentPosturePublicationOutcome<'session> {
        let state = self
            .state
            .take()
            .expect("live posture retry owns its state");
        let NativeIntentPostureRejected {
            mut admitted,
            frame,
            rejections: _,
        } = *state;
        if let Err(denial) = admitted
            .transfer
            .observation
            .validate(admitted.session, &frame)
        {
            return stopped(
                crate::runtime::intent_execution::UiIntentConsequenceStopReason::Preparation(
                    Box::new(denial),
                ),
            );
        }
        if let Err(denial) = admitted.reservation.begin_effecting() {
            return stopped(
                crate::runtime::intent_execution::UiIntentConsequenceStopReason::RebindAdmission(
                    denial,
                ),
            );
        }
        let deadline = presentation_deadline(&admitted.plan);
        let outcome = admitted
            .session
            .present_prepared_observed_frame(
                frame,
                &admitted.transfer.observation,
                None,
                deadline,
                now_tick,
            )
            .expect("exclusive retry preserves the validated observation/frame bond");
        finish(admitted, outcome)
    }
}
