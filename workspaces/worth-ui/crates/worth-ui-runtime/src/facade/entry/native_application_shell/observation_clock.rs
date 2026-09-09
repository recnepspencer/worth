impl super::WorthUiNativeApplicationShell {
    /// Whether current pointer meaning differs from committed mounted output.
    /// Preparing or retaining a candidate cannot acknowledge this work.
    pub fn native_pointer_presentation_pending(&self) -> bool {
        self.session.mounted.pointer_presentation_pending(
            self.surface,
            self.binding,
            self.session.pointer_affordance_snapshot.as_ref(),
        )
    }

    pub(crate) fn install_native_observation_clock(
        &mut self,
        clock: worth_ui_host_native::UiNativeObservationClock,
    ) -> Result<(), ()> {
        if self.session.observation_clock.is_some() {
            return Err(());
        }
        self.session.observation_clock = Some(clock);
        Ok(())
    }

    pub(crate) fn close_native_observation_time(&mut self) -> Result<Option<u64>, ()> {
        if !self.session.interaction.pointer_presence_is_enabled() {
            return Ok(None);
        }
        let turn = self.session.begin_observation_turn().map_err(|_| ())?;
        let observations = turn.seal().map_err(|_| ())?;
        self.session
            .classify_observations(observations)
            .map_err(|_| ())?;
        Ok(self
            .session
            .pointer_affordance_snapshot
            .as_ref()
            .and_then(|snapshot| snapshot.confirmation_deadline()))
    }
}
