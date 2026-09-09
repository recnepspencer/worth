impl super::WorthUiActiveApplicationSession {
    pub(crate) fn observe_activation_operability(
        &self,
        target: crate::runtime::interaction::UiPresentedInteractionTargetView,
    ) -> Result<
        crate::runtime::intent::UiIntentStandingOperabilityObservation,
        crate::runtime::intent::UiIntentStandingOperabilityUnavailable,
    > {
        let prepared = self.application.prepared_authority();
        crate::runtime::intent::observe_activation_operability(
            target,
            prepared.intent_catalog(),
            prepared.capabilities().intent_definitions(),
            prepared.intent_execution_bindings(),
            &self.active_generation_identity(),
            &self.mounted,
            &self.intent_application_facts,
            self.intent_execution.occupancy(),
            &self.intent_confirmation,
            self.observation_clock.as_ref().map(|clock| {
                worth_ui_host_contract::UiHostObservationTimeBasis::HostMonotonicMillis(
                    clock.sample_millis(),
                )
            }),
        )
    }
}
