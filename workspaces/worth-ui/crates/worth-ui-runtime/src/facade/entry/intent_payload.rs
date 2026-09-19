use super::WorthUiActiveApplicationSession;

impl WorthUiActiveApplicationSession {
    pub fn prepare_intent_payload(
        &mut self,
        route: crate::facade::intent::UiResolvedProductIntentRoute,
    ) -> Result<
        crate::facade::intent::UiPreparedIntentPayload,
        crate::facade::intent::UiIntentPayloadStop,
    > {
        let generation = self.active_generation_identity();
        crate::runtime::intent::prepare_intent_payload(
            route,
            self.application
                .prepared_authority()
                .capabilities()
                .intent_definitions(),
            self.application
                .prepared_authority()
                .intent_execution_bindings(),
            &generation,
            &self.mounted,
            &self.intent_application_facts,
            self.intent_execution.occupancy(),
        )
    }

    pub fn evaluate_intent_operability(
        &mut self,
        candidate: crate::facade::intent::UiPreparedIntentPayload,
    ) -> crate::facade::intent::UiIntentOperabilityOutcome {
        let generation = self.active_generation_identity();
        let outcome = crate::runtime::intent::evaluate_intent_operability(
            candidate,
            &generation,
            &self.mounted,
        );
        let appearance_operability_demanded = self
            .application
            .prepared_authority()
            .consumed_fact_index()
            .appearance_axis_demand()
            .contains(worth_ui_dsl::UiAppearanceStateAxis::Operability);
        if appearance_operability_demanded {
            match &outcome {
                crate::runtime::intent::UiIntentOperabilityOutcome::Operable(proof) => {
                    self.intent_admission.record_operability_standing_fact(
                        proof.candidate_for_standing_fact(),
                        proof.decision(),
                    );
                }
                crate::runtime::intent::UiIntentOperabilityOutcome::Inoperable(candidate) => {
                    self.intent_admission.record_operability_standing_fact(
                        candidate.candidate(),
                        candidate.decision(),
                    );
                }
            }
        }
        outcome
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn reserve_intent_occupancy_for_certification(
        &mut self,
        proof: crate::facade::intent::UiIntentOperabilityProof,
    ) -> Result<
        crate::runtime::intent::UiIntentOccupancyReservation,
        crate::runtime::intent::UiIntentOccupancyReservationDenial,
    > {
        self.intent_admission
            .reserve_occupancy_for_certification(&mut self.intent_execution, proof)
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn release_intent_occupancy_for_certification(
        &mut self,
        reservation: crate::runtime::intent::UiIntentOccupancyReservation,
    ) -> crate::runtime::intent::UiIntentOccupancyReleasePosture {
        self.intent_admission
            .release_occupancy_for_certification(&mut self.intent_execution, reservation)
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn active_intent_occupancy_count_for_certification(&self) -> usize {
        self.intent_execution.occupancy().active_count()
    }
}
