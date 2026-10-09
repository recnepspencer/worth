use super::{
    cooperative_execution::next_cooperative_execution_admission_identity,
    WorthQueryCooperativeGraphProviderExecution, WorthQueryGraphProviderExecution,
    WorthQueryGraphProviderMemoryArena, WorthQueryGraphProviderRetainedMemory,
    WorthQueryGraphProviderStepDenial, WorthQueryGraphProviderStepDenialKind,
    WorthQueryOwnedGraphProviderExecution, WorthQueryProviderExecutionReleaseEvidence,
};

/// Provider-facing construction port bound to one admitted managed execution.
///
/// A provider can retain start-time state only by moving memory returned by
/// this port into its execution object.
pub struct WorthQueryGraphProviderExecutionStart<'scope> {
    execution: worth_execution::ExecutionRequest<'scope, 'scope>,
    memory: WorthQueryGraphProviderMemoryArena,
    denial: Option<WorthQueryGraphProviderStepDenial>,
    execution_admission_identity: u64,
    admitted_execution: Option<Box<dyn WorthQueryGraphProviderExecution>>,
}

impl<'scope> WorthQueryGraphProviderExecutionStart<'scope> {
    pub(crate) fn new(
        execution: worth_execution::ExecutionRequest<'scope, 'scope>,
        memory: WorthQueryGraphProviderMemoryArena,
    ) -> Self {
        Self {
            execution,
            memory,
            denial: None,
            execution_admission_identity: next_cooperative_execution_admission_identity(),
            admitted_execution: None,
        }
    }

    /// The active request owning this call; it is never stored in retained provider state.
    pub fn execution_request(&self) -> worth_execution::ExecutionRequest<'scope, 'scope> {
        self.execution
    }

    pub fn retain_bytes(
        &mut self,
        byte_count: usize,
    ) -> Result<WorthQueryGraphProviderRetainedMemory, WorthQueryGraphProviderStepDenial> {
        if let Some(denial) = &self.denial {
            return Err(denial.clone());
        }
        match self.memory.retain_bytes(byte_count) {
            Ok(memory) => Ok(memory),
            Err(denial) => {
                self.denial = Some(denial.clone());
                Err(denial)
            }
        }
    }

    pub fn admit_cooperative_execution<E: WorthQueryGraphProviderExecution>(
        &mut self,
        construct: impl FnOnce() -> E,
    ) -> Result<WorthQueryCooperativeGraphProviderExecution<E>, WorthQueryGraphProviderStepDenial>
    {
        if let Some(denial) = &self.denial {
            return Err(denial.clone());
        }
        if self.admitted_execution.is_some() {
            let denial = WorthQueryGraphProviderStepDenial::new(
                WorthQueryGraphProviderStepDenialKind::MultipleExecutionAdmissions,
                "provider start can admit exactly one cooperative execution",
            );
            self.denial = Some(denial.clone());
            return Err(denial);
        }
        self.admitted_execution = Some(Box::new(construct()));
        Ok(WorthQueryCooperativeGraphProviderExecution::new(
            self.memory.snapshot().arena_identity(),
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
        if admitted.arena_identity() != self.memory.snapshot().arena_identity()
            || admitted.admission_identity() != self.execution_admission_identity
        {
            let denial = WorthQueryGraphProviderStepDenial::new(
                WorthQueryGraphProviderStepDenialKind::ForeignExecutionAdmission,
                "cooperative provider execution belongs to another managed start",
            );
            self.denial = Some(denial.clone());
            return Err(
                crate::domain_computation::WorthQueryGraphProviderFailure::new(denial.detail()),
            );
        }
        self.admitted_execution.take().ok_or_else(|| {
            crate::domain_computation::WorthQueryGraphProviderFailure::new(
                "cooperative provider execution admission no longer owns an execution",
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

    pub(crate) fn finish(self) -> Result<(), WorthQueryGraphProviderStepDenial> {
        self.denial.map_or(Ok(()), Err)
    }
}
