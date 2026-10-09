use super::{WorthQueryGraphProviderMemoryArena, WorthQueryGraphProviderRetainedMemory};
use crate::domain_computation::provider_session::graph_provider::bounded_step::{
    WorthQueryCooperativeGraphProviderExecution, WorthQueryOwnedGraphProviderExecution,
    WorthQueryProviderExecutionReleaseEvidence,
};
use crate::domain_computation::{
    WorthQueryGraphProviderExecution, WorthQueryGraphProviderStepDenial,
    WorthQueryGraphProviderStepDenialKind,
};

/// Step-external port that can provisionally rebind retained provider memory
/// during the existing same-runtime readmission transaction.
pub struct WorthQueryGraphProviderRestoreMemory<'scope> {
    execution: worth_execution::ExecutionRequest<'scope, 'scope>,
    arena: WorthQueryGraphProviderMemoryArena,
    execution_admission_identity: u64,
    admitted_execution: Option<Box<dyn WorthQueryGraphProviderExecution>>,
    denial: Option<WorthQueryGraphProviderStepDenial>,
}

impl<'scope> WorthQueryGraphProviderRestoreMemory<'scope> {
    pub(crate) fn new(
        execution: worth_execution::ExecutionRequest<'scope, 'scope>,
        arena: WorthQueryGraphProviderMemoryArena,
    ) -> Self {
        Self {
            execution,
            arena,
            execution_admission_identity:
                crate::domain_computation::provider_session::graph_provider::bounded_step::cooperative_execution::next_cooperative_execution_admission_identity(),
            admitted_execution: None,
            denial: None,
        }
    }

    /// The active request owning this call; it is never stored in retained provider state.
    pub fn execution_request(&self) -> worth_execution::ExecutionRequest<'scope, 'scope> {
        self.execution
    }

    pub fn rebind(
        &mut self,
        retained: &WorthQueryGraphProviderRetainedMemory,
    ) -> Result<WorthQueryGraphProviderRetainedMemory, WorthQueryGraphProviderStepDenial> {
        let expected = self.arena.snapshot().arena_identity();
        if retained.arena_identity() != expected {
            let denial = WorthQueryGraphProviderStepDenial::new(
                WorthQueryGraphProviderStepDenialKind::ForeignRetainedMemory,
                "provider restore memory belongs to a different managed execution",
            );
            self.denial = Some(denial.clone());
            return Err(denial);
        }
        Ok(retained.provisional_restore_alias())
    }

    pub fn admit_cooperative_execution(
        &mut self,
        construct: impl FnOnce() -> Box<dyn WorthQueryGraphProviderExecution>,
    ) -> Result<
        WorthQueryCooperativeGraphProviderExecution<Box<dyn WorthQueryGraphProviderExecution>>,
        WorthQueryGraphProviderStepDenial,
    > {
        if let Some(denial) = &self.denial {
            return Err(denial.clone());
        }
        if self.admitted_execution.is_some() {
            let denial = WorthQueryGraphProviderStepDenial::new(
                WorthQueryGraphProviderStepDenialKind::MultipleExecutionAdmissions,
                "provider restore can admit exactly one cooperative execution",
            );
            self.denial = Some(denial.clone());
            return Err(denial);
        }
        self.admitted_execution = Some(construct());
        Ok(WorthQueryCooperativeGraphProviderExecution::new(
            self.arena.snapshot().arena_identity(),
            self.execution_admission_identity,
        ))
    }

    pub(crate) fn validate_returned_execution<E>(
        &mut self,
        admitted: WorthQueryCooperativeGraphProviderExecution<E>,
    ) -> Result<
        Box<dyn WorthQueryGraphProviderExecution>,
        crate::domain_computation::WorthQueryGraphProviderFailure,
    > {
        if let Some(denial) = &self.denial {
            return Err(
                crate::domain_computation::WorthQueryGraphProviderFailure::new(denial.detail()),
            );
        }
        if admitted.arena_identity() != self.arena.snapshot().arena_identity()
            || admitted.admission_identity() != self.execution_admission_identity
        {
            self.denial = Some(WorthQueryGraphProviderStepDenial::new(
                WorthQueryGraphProviderStepDenialKind::ForeignExecutionAdmission,
                "cooperative provider execution belongs to another managed restore",
            ));
            return Err(
                crate::domain_computation::WorthQueryGraphProviderFailure::new(
                    self.denial
                        .as_ref()
                        .expect("foreign restore admission latches a denial")
                        .detail(),
                ),
            );
        }
        self.admitted_execution.take().ok_or_else(|| {
            crate::domain_computation::WorthQueryGraphProviderFailure::new(
                "cooperative provider restore admission no longer owns an execution",
            )
        })
    }

    pub(crate) fn release_unreturned_execution(
        &mut self,
    ) -> Option<WorthQueryProviderExecutionReleaseEvidence> {
        self.admitted_execution
            .take()
            .map(|execution| WorthQueryOwnedGraphProviderExecution::new(execution).release())
    }
}
