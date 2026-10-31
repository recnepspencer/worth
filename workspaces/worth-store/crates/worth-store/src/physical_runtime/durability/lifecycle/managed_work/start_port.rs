//! Existing-owner start capability and read-only observation of an exact local attempt.
use super::*;
/// This owns neither a prepared payload nor a start/cancellation capability.
pub(in crate::physical_runtime) struct ExistingPhysicalMutationObservation {
    attempt: Arc<crate::physical_runtime::PhysicalMutationAttempt>,
}
impl ExistingPhysicalMutationObservation {
    pub(in crate::physical_runtime) fn terminal(&self) -> Option<PhysicalMutationTerminalFact> {
        self.attempt.visible_terminal_fact()
    }
}

impl PhysicalMutationStartPort {
    pub(in crate::physical_runtime) fn with_interruption(
        mut self,
        interruption: Option<Arc<worth_execution::ExecutionInterruptionView>>,
    ) -> Self {
        self.interruption = interruption;
        self
    }
    pub(in crate::physical_runtime) fn interruption(
        &self,
    ) -> Option<Arc<worth_execution::ExecutionInterruptionView>> {
        self.interruption.clone()
    }
    pub(in crate::physical_runtime) fn original_request_stop(
        &self,
    ) -> Option<worth_execution::MapKernelStop> {
        self.interruption
            .as_ref()
            .and_then(|view| view.check_live().err())
    }

    pub(in crate::physical_runtime) fn observe_existing(
        &self,
        prepared: &PreparedPhysicalMutation,
    ) -> Option<ExistingPhysicalMutationObservation> {
        let owner = self.owner.upgrade()?;
        let attempt = owner
            .state()
            .attempts
            .get(&prepared.mutation_identity())
            .cloned()?;
        if attempt.identity() != prepared.mutation_identity()
            || attempt.identity().runtime_identity()
                != prepared.mutation_identity().runtime_identity()
            || attempt.idempotency_identity() != prepared.idempotency_identity()
            || attempt.fingerprint() != prepared.request_fingerprint()
        {
            return None;
        }
        Some(ExistingPhysicalMutationObservation { attempt })
    }

    pub(in crate::physical_runtime) fn start(
        &self,
        prepared: PreparedPhysicalMutation,
    ) -> PhysicalMutationHandle {
        self.start_registered(prepared, None)
    }

    pub(in crate::physical_runtime) fn start_registered(
        &self,
        prepared: PreparedPhysicalMutation,
        step: Option<
            Arc<crate::physical_runtime::anchored_records::publication::step_custody::AcceptedStep>,
        >,
    ) -> PhysicalMutationHandle {
        let Some(owner) = self.owner.upgrade() else {
            let identity = prepared.mutation_identity();
            let attempt =
                crate::physical_runtime::durability::mutation::PhysicalMutationAttempt::new(
                    &prepared,
                    Weak::new(),
                );
            if let Some(step) = step {
                attempt.register_anchored_step(step);
            }
            attempt.mark_stale();
            attempt.install_terminal(PhysicalMutationTerminalFact::Indeterminate(
                IndeterminatePhysicalMutation::possible_effect(
                    identity,
                    prepared.idempotency_identity(),
                    prepared.request_fingerprint(),
                    PhysicalMutationIndeterminateStage::RuntimeUnavailable,
                    0,
                ),
            ));
            attempt.finish_preserved_anchored_step();
            attempt.publish_terminal();
            return PhysicalMutationHandle::new(&attempt);
        };
        owner.start(prepared, step)
    }
}
