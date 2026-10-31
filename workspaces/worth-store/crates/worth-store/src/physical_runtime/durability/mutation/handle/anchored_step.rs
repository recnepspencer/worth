use super::*;

impl PhysicalMutationAttempt {
    pub(in crate::physical_runtime) fn register_anchored_step(
        self: &Arc<Self>,
        step: Arc<
            crate::physical_runtime::anchored_records::publication::step_custody::AcceptedStep,
        >,
    ) {
        let mut retained = self.anchored_step.lock().unwrap_or_else(|p| p.into_inner());
        step.bind(self);
        if retained.is_none() {
            *retained = Some(step.managed_custody());
        }
    }

    pub(in crate::physical_runtime) fn finish_preserved_anchored_step(&self) {
        let fact = self.state().terminal.clone();
        if let Some(fact) = fact {
            self.finish_anchored_step(
                &fact,
                Some(
                    crate::physical_runtime::AnchoredRecordStepSettlementFailure::WorkerUnavailable,
                ),
            );
        }
    }

    pub(in crate::physical_runtime) fn finish_anchored_step(
        &self,
        fact: &PhysicalMutationTerminalFact,
        failure: Option<crate::physical_runtime::anchored_records::publication::step_custody::AnchoredRecordStepSettlementFailure>,
    ) {
        let step = self
            .anchored_step
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .take();
        if let Some(custody) = step {
            custody.step().finish(fact, failure);
        }
    }
}
