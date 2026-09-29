use super::WorthUiActiveApplicationSession;

impl WorthUiActiveApplicationSession {
    pub fn admit_intent<I, D>(
        &mut self,
        definition: crate::facade::intent::UiIntentDefinition<I, D>,
        outcome: crate::facade::intent::UiIntentOperabilityOutcome,
    ) -> crate::facade::intent::UiIntentAdmissionDecision<I>
    where
        I: crate::facade::intent::UiIntent,
        D: crate::facade::intent::UiIntentDefinitionDestination,
    {
        match outcome {
            crate::runtime::intent::UiIntentOperabilityOutcome::Operable(proof) => self
                .admit_prepared_candidate(
                    definition,
                    crate::runtime::intent::UiPreparedIntentAdmissionCandidate::direct(proof),
                ),
            crate::runtime::intent::UiIntentOperabilityOutcome::Inoperable(candidate) => {
                let definitions = self
                    .application
                    .prepared_authority()
                    .capabilities()
                    .intent_definitions();
                let admission_cost = match crate::runtime::intent::validate_typed_inoperable(
                    definition,
                    &candidate,
                    definitions,
                ) {
                    Ok(cost) => cost,
                    Err(failure) => {
                        let (reason, cost) = failure.into_parts();
                        return self.intent_admission.reject(reason, cost);
                    }
                };
                if !candidate.is_exclusively_confirmable() {
                    let (_, decision) = candidate.into_parts();
                    return self.intent_admission.reject(
                        crate::runtime::intent::UiIntentAdmissionStopReason::Inoperable(Box::new(
                            decision,
                        )),
                        admission_cost,
                    );
                }
                match self.issue_intent_confirmation(candidate) {
                    crate::runtime::intent::UiIntentConfirmationIssueOutcome::Pending(pending) => {
                        crate::runtime::intent::UiIntentAdmissionDecision::ConfirmationRequired(
                            pending,
                        )
                    }
                    crate::runtime::intent::UiIntentConfirmationIssueOutcome::Stopped(stop) => {
                        self.intent_admission.reject(
                            crate::runtime::intent::UiIntentAdmissionStopReason::Confirmation(
                                Box::new(stop),
                            ),
                            admission_cost,
                        )
                    }
                }
            }
        }
    }

    pub fn admit_confirmed_intent<I, D>(
        &mut self,
        definition: crate::facade::intent::UiIntentDefinition<I, D>,
        candidate: crate::facade::intent::UiConfirmedIntentCandidate,
    ) -> crate::facade::intent::UiIntentAdmissionDecision<I>
    where
        I: crate::facade::intent::UiIntent,
        D: crate::facade::intent::UiIntentDefinitionDestination,
    {
        self.admit_prepared_candidate(
            definition,
            crate::runtime::intent::UiPreparedIntentAdmissionCandidate::confirmed(candidate),
        )
    }

    pub fn cancel_admitted_intent<I: crate::facade::intent::UiIntent>(
        &mut self,
        admitted: crate::facade::intent::UiAdmittedIntent<I>,
    ) -> crate::facade::intent::UiIntentAdmissionSettlementReceipt {
        self.intent_admission
            .release(admitted, &mut self.intent_execution)
    }

    pub fn intent_admission_metrics(&self) -> crate::facade::intent::UiIntentAdmissionMetrics {
        self.intent_admission.metrics(&self.intent_execution)
    }

    fn admit_prepared_candidate<I, D>(
        &mut self,
        definition: crate::facade::intent::UiIntentDefinition<I, D>,
        candidate: crate::runtime::intent::UiPreparedIntentAdmissionCandidate,
    ) -> crate::facade::intent::UiIntentAdmissionDecision<I>
    where
        I: crate::facade::intent::UiIntent,
        D: crate::facade::intent::UiIntentDefinitionDestination,
    {
        let generation = self.active_generation_identity();
        let command_contexts = self.all_current_command_routing_contexts();
        let prepared = self.application.prepared_authority();
        let context = crate::runtime::intent::UiIntentAdmissionCurrentnessContext {
            catalog: prepared.intent_catalog(),
            definitions: prepared.capabilities().intent_definitions(),
            generation: &generation,
            mounted: &self.mounted,
            application_facts: &self.intent_application_facts,
            expressions: &self.expressions,
            command_contexts,
        };
        match crate::runtime::intent::prepare_typed_admission_candidate(
            definition, candidate, context,
        ) {
            Ok(candidate) => {
                let evidence =
                    crate::runtime::intent::UiIntentCausalTraceAdmissionPrefix::from_candidate(
                        &candidate,
                    );
                let decision = self
                    .intent_admission
                    .admit(candidate, &mut self.intent_execution);
                if let (
                    Some(evidence),
                    crate::runtime::intent::UiIntentAdmissionDecision::Admitted(admitted),
                ) = (evidence, &decision)
                {
                    let _ = self.intent_evidence.record_admission(
                        evidence,
                        admitted.slot_identity(),
                        admitted.lineage(),
                    );
                }
                decision
            }
            Err(failure) => {
                let (reason, cost) = failure.into_parts();
                self.intent_admission.reject(reason, cost)
            }
        }
    }
}
