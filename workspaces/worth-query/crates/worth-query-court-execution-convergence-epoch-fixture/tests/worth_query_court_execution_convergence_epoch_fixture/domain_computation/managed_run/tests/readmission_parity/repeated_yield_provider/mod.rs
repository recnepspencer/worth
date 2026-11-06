//! The repeated-yield provider and its native checkpoint custody.
use crate::domain_computation::managed_run::tests::readmission_parity::*;

#[derive(Clone, Copy)]
pub struct RepeatedYieldProvider {
    pub step_count: u8,
}

pub struct RepeatedYieldExecution {
    pub next_step: u8,
    pub step_count: u8,
    pub retained: Option<WorthQueryGraphProviderRetainedMemory>,
}

pub struct RepeatedYieldCheckpoint {
    pub next_step: u8,
    pub step_count: u8,
    pub retained: WorthQueryGraphProviderRetainedMemory,
}

impl WorthQueryGraphParticipationProvider<ManagedGraph> for RepeatedYieldProvider {
    type Execution = RepeatedYieldExecution;

    fn execution_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport {
        worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::provider_session::tests::execution_resource_support_with_yield(
            "repeated-yield-parity",
            8,
        )
    }

    fn begin(
        &self,
        _call: &WorthQueryGraphProviderCall,
        start: &mut WorthQueryGraphProviderExecutionStart,
    ) -> Result<
        WorthQueryCooperativeGraphProviderExecution<Self::Execution>,
        WorthQueryGraphProviderFailure,
    > {
        admit_provider_execution(
            start,
            RepeatedYieldExecution {
                next_step: 0,
                step_count: self.step_count,
                retained: None,
            },
        )
    }
}

impl WorthQueryGraphProviderExecution for RepeatedYieldExecution {
    fn advance(
        &mut self,
        step: &mut WorthQueryGraphProviderStep,
    ) -> Result<WorthQueryGraphProviderStepDisposition, WorthQueryGraphProviderFailure> {
        step.perform_work_unit(|| Ok(()))?;
        self.next_step = self.next_step.saturating_add(1);
        if self.next_step < self.step_count {
            if self.retained.is_none() {
                self.retained = Some(step.retain_bytes(1).map_err(step_failure)?);
            }
            step.record_checkpoint_available().map_err(step_failure)?;
            return Ok(WorthQueryGraphProviderStepDisposition::continue_work());
        }
        drop(self.retained.take());
        WorthQueryGraphProviderStepDisposition::complete("repeated-yield-parity-complete")
            .map_err(WorthQueryGraphProviderFailure::new)
    }

    fn suspend(
        &mut self,
    ) -> Result<
        Box<dyn worth_query_execution::facade::domain_computation::WorthQueryGraphProviderCheckpoint>,
        WorthQueryGraphProviderFailure,
    >{
        Ok(Box::new(RepeatedYieldCheckpoint {
            next_step: self.next_step,
            step_count: self.step_count,
            retained: self
                .retained
                .take()
                .expect("checkpointable execution retains governed memory"),
        }))
    }

    fn dispose(&mut self) -> Result<(), WorthQueryGraphProviderFailure> {
        Ok(())
    }
}

impl worth_query_execution::facade::domain_computation::WorthQueryGraphProviderCheckpoint
    for RepeatedYieldCheckpoint
{
    fn retained_bytes(&self) -> u64 {
        u64::try_from(self.retained.len()).unwrap()
    }

    fn restore(
        &self,
        _call: &WorthQueryGraphProviderCall,
        memory: &mut WorthQueryGraphProviderRestoreMemory,
    ) -> Result<
        WorthQueryCooperativeGraphProviderExecution<Box<dyn WorthQueryGraphProviderExecution>>,
        WorthQueryGraphProviderFailure,
    > {
        let execution = Box::new(RepeatedYieldExecution {
            next_step: self.next_step,
            step_count: self.step_count,
            retained: Some(memory.rebind(&self.retained).map_err(step_failure)?),
        }) as Box<dyn WorthQueryGraphProviderExecution>;
        admit_restored_provider_execution(memory, execution)
    }
}

pub fn step_failure(
    denial: worth_query_execution::facade::domain_computation::WorthQueryGraphProviderStepDenial,
) -> WorthQueryGraphProviderFailure {
    WorthQueryGraphProviderFailure::new(denial.detail())
}
