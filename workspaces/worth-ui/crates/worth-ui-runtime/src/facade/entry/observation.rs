impl super::WorthUiActiveApplicationSession {
    pub fn classify_observations(
        &mut self,
        mut observations: crate::facade::observation::UiAdmittedObservationSet,
    ) -> Result<
        crate::facade::observation::UiChangeClassificationOutcome,
        crate::facade::observation::UiChangeClassificationDenial,
    > {
        self.application
            .validate_observation_basis(self.identity, &observations)?;
        if observations
            .appearance_owner_snapshot()
            .is_some_and(|snapshot| snapshot.generation() != &self.active_generation_identity())
        {
            return Err(crate::facade::observation::UiChangeClassificationDenial::ForeignApplicationGeneration);
        }
        self.validate_pointer_observation_currentness(&observations)?;
        let owners = observations.take_appearance_owner_snapshot();
        let outcome = self
            .application
            .classify_observations(self.identity, observations)?;
        self.appearance_owner_snapshot = owners;
        Ok(outcome)
    }

    fn validate_pointer_observation_currentness(
        &self,
        observations: &crate::facade::observation::UiAdmittedObservationSet,
    ) -> Result<(), crate::facade::observation::UiChangeClassificationDenial> {
        let active_generation = self.active_generation_identity();
        for transition in observations.observations().iter().filter_map(
            crate::runtime::observation::UiAdmittedObservation::pointer_presence_transition,
        ) {
            if transition.generation() != &active_generation {
                return Err(
                    crate::facade::observation::UiChangeClassificationDenial::ForeignApplicationGeneration,
                );
            }
            self.mounted
                .validate_current_frame(transition.presentation().frame())
                .map_err(|_| stale_pointer_transition())?;
            self.mounted
                .validate_binding(transition.presentation().binding())
                .map_err(|_| stale_pointer_transition())?;
            if let (Some(instance), Some(receipt)) =
                (transition.current(), transition.current_node_receipt())
            {
                self.mounted
                    .validate_current_receipt(instance, receipt)
                    .map_err(|_| stale_pointer_transition())?;
            }
        }
        Ok(())
    }

    #[allow(
        dead_code,
        reason = "Gate 0 proves origin admission without enabling live theme switching"
    )]
    pub(crate) fn issue_theme_switch_origin(
        &self,
        admitted: &crate::runtime::observation::UiAdmittedObservationSet,
        family: crate::runtime::appearance::UiThemeSwitchOriginFamily,
    ) -> Result<
        crate::runtime::appearance::UiThemeSwitchOrigin,
        crate::runtime::appearance::UiThemeSwitchOriginAdmissionDenial,
    > {
        crate::runtime::appearance::UiThemeSwitchOrigin::admit_current(self, admitted, family)
    }

    pub fn begin_observation_turn(
        &mut self,
    ) -> Result<
        crate::facade::observation::UiObservationTurn<'_>,
        crate::facade::observation::UiObservationTurnDenial,
    > {
        let session = self.identity;
        let consumed_facts = self.application.prepared_authority().consumed_fact_index();
        let appearance_axis_demand = consumed_facts.appearance_axis_demand();
        let appearance_close = consumed_facts.has_appearance_consumers().then(|| {
            crate::runtime::observation::UiAppearanceObservationCloseInput::new(
                appearance_axis_demand,
                crate::runtime::WorthUiActiveApplicationGenerationIdentity::current(
                    session,
                    self.application.generation_identity(),
                ),
                self.focus.as_ref(),
                self.selection.as_ref(),
                &self.intent_admission,
                &self.intent_application_facts,
                &self.interaction,
            )
        });
        self.application
            .begin_observation_turn(session, appearance_close)
    }
}

const fn stale_pointer_transition() -> crate::facade::observation::UiChangeClassificationDenial {
    crate::facade::observation::UiChangeClassificationDenial::StalePointerPresenceTransition
}
