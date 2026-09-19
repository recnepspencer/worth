use super::WorthUiActiveApplicationSession;

impl WorthUiActiveApplicationSession {
    /// Observe the current confirmation control at a host-supplied time.
    ///
    /// `time_basis` must describe the same host clock domain as challenge issuance.
    /// The result is an as-of observation, not a clock reading or admission proof.
    /// Callers must obtain fresh host time before treating expiry as current.
    pub fn observe_intent_confirmation(
        &self,
        target: crate::facade::interaction::UiPresentedInteractionTargetView,
        time_basis: worth_ui_host_contract::UiHostObservationTimeBasis,
    ) -> crate::facade::intent::UiIntentConfirmationObservation {
        let generation = self.active_generation_identity();
        let prepared = self.application.prepared_authority();
        crate::runtime::intent::observe_confirmation(
            &self.intent_confirmation,
            target,
            time_basis,
            crate::runtime::intent::UiIntentConfirmationReadContext {
                catalog: prepared.intent_catalog(),
                definitions: prepared.capabilities().intent_definitions(),
                generation: &generation,
                mounted: &self.mounted,
                application_facts: &self.intent_application_facts,
                occupancy: self.intent_execution.occupancy(),
            },
        )
    }

    pub fn issue_intent_confirmation(
        &mut self,
        candidate: crate::facade::intent::UiInoperableIntentCandidate,
    ) -> crate::facade::intent::UiIntentConfirmationIssueOutcome {
        let lineage = self.intent_admission.issue_lineage();
        self.intent_confirmation.issue(candidate, lineage)
    }

    pub fn continue_intent_confirmation(
        &mut self,
        route: crate::facade::intent::UiResolvedConfirmationIntentRoute,
    ) -> crate::facade::intent::UiIntentConfirmationContinuation {
        let generation = self.active_generation_identity();
        let prepared = self.application.prepared_authority();
        let context = crate::runtime::intent::UiIntentConfirmationReadContext {
            catalog: prepared.intent_catalog(),
            definitions: prepared.capabilities().intent_definitions(),
            generation: &generation,
            mounted: &self.mounted,
            application_facts: &self.intent_application_facts,
            occupancy: self.intent_execution.occupancy(),
        };
        crate::runtime::intent::continue_confirmation(&mut self.intent_confirmation, route, context)
    }

    pub fn intent_confirmation_metrics(
        &self,
    ) -> crate::facade::intent::UiIntentConfirmationMetrics {
        self.intent_confirmation.metrics()
    }
}
