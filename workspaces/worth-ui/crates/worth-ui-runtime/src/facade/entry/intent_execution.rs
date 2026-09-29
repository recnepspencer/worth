use super::WorthUiActiveApplicationSession;

impl WorthUiActiveApplicationSession {
    pub fn advance_intent_executions(
        &mut self,
        reading: crate::facade::intent::UiIntentExecutionClockReading,
    ) -> crate::facade::intent::UiIntentExecutionAdvanceOutcome {
        let outcome = self.intent_execution.advance(reading.tick());
        if let crate::runtime::intent_execution::UiIntentExecutionAdvanceOutcome::Advanced(report) =
            &outcome
        {
            self.intent_evidence
                .record_transitions(report.transitions());
        }
        outcome
    }

    pub fn retry_intent_recovery(
        &mut self,
        recovery: crate::facade::intent::UiIntentRecoveryHandle,
        reading: crate::facade::intent::UiIntentExecutionClockReading,
    ) -> crate::facade::intent::UiIntentRecoveryProgressOutcome {
        self.intent_execution
            .retry_recovery(recovery, reading.tick())
    }

    pub fn dispatch_admitted_intent<I: crate::facade::intent::UiIntent>(
        &mut self,
        admitted: crate::facade::intent::UiAdmittedIntent<I>,
        deadline: crate::facade::intent::UiIntentExecutionDeadlineBasis,
    ) -> crate::facade::intent::UiIntentExecutionDispatchOutcome {
        let admission = admitted.slot_identity();
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
        match self.intent_execution.dispatch(
            admitted,
            context,
            self.identity.as_u64(),
            deadline.tick(),
        ) {
            crate::runtime::intent_execution::UiIntentExecutionDispatchOutcome::AttemptPrepared(
                receipt,
            ) => {
                let reference = self.intent_evidence.record_dispatch(admission, receipt);
                crate::runtime::intent_execution::UiIntentExecutionDispatchOutcome::AttemptPrepared(
                    receipt.with_evidence_reference(reference),
                )
            }
            crate::runtime::intent_execution::UiIntentExecutionDispatchOutcome::Stopped(stop) => {
                crate::runtime::intent_execution::UiIntentExecutionDispatchOutcome::Stopped(stop)
            }
        }
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn install_intent_execution_capacity_for_certification(
        &mut self,
        capacity: crate::runtime::intent_execution::UiIntentExecutionCapacity,
    ) -> bool {
        self.intent_execution
            .install_capacity_for_certification(capacity)
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn intent_execution_census_for_certification(
        &self,
    ) -> crate::runtime::intent_execution::UiIntentExecutionAdmissionCensus {
        self.intent_execution.census()
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn exhaust_intent_execution_reservation_identities_for_certification(
        &mut self,
    ) -> usize {
        self.intent_execution
            .exhaust_reservation_identities_for_certification()
    }
}
