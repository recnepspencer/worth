use crate::domain_computation::managed_run::tests::admit_provider_execution;
use crate::domain_computation::managed_run::tests::graph_material;
use crate::domain_computation::managed_run::tests::ManagedGraph;
use worth_query_execution::facade::domain_computation::WorthQueryCooperativeGraphProviderExecution;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderCall;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderExecutionStart;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderFailure;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderRetainedMemory;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderStep;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderStepDisposition;

use crate::domain_computation::managed_run::tests::*;
pub struct TwoStepProvider;
pub struct TwoStepExecution {
    pub phase: u8,
    pub retained: Option<WorthQueryGraphProviderRetainedMemory>,
}
impl WorthQueryGraphProviderExecution for TwoStepExecution {
    fn advance(
        &mut self,
        step: &mut WorthQueryGraphProviderStep,
    ) -> Result<WorthQueryGraphProviderStepDisposition, WorthQueryGraphProviderFailure> {
        match self.phase {
            0 => {
                self.phase = 1;
                for _ in 0..3 {
                    step.perform_work_unit(|| Ok(()))?;
                }
                step.with_scratch_bytes(8, |_| Ok(()))?;
                self.retained = Some(step.retain_bytes(4).map_err(step_failure)?);
                Ok(WorthQueryGraphProviderStepDisposition::continue_work())
            }
            1 => {
                self.phase = 2;
                drop(self.retained.take());
                for _ in 0..2 {
                    step.perform_work_unit(|| Ok(()))?;
                }
                step.emit_projection_chunk(graph_material())
                    .map_err(step_failure)?;
                WorthQueryGraphProviderStepDisposition::complete("two-step-provider")
                    .map_err(WorthQueryGraphProviderFailure::new)
            }
            _ => Err(WorthQueryGraphProviderFailure::new(
                "completed provider advanced again",
            )),
        }
    }

    fn dispose(&mut self) -> Result<(), WorthQueryGraphProviderFailure> {
        Ok(())
    }
}
impl WorthQueryGraphParticipationProvider<ManagedGraph> for TwoStepProvider {
    type Execution = TwoStepExecution;

    fn execution_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport {
        crate::domain_computation::provider_session::tests::execution_resource_support(
            "managed-two-step",
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
            TwoStepExecution {
                phase: 0,
                retained: None,
            },
        )
    }
}
pub struct FailingProvider;
pub struct FailingExecution {
    pub retained: Option<WorthQueryGraphProviderRetainedMemory>,
}
impl WorthQueryGraphProviderExecution for FailingExecution {
    fn advance(
        &mut self,
        step: &mut WorthQueryGraphProviderStep,
    ) -> Result<WorthQueryGraphProviderStepDisposition, WorthQueryGraphProviderFailure> {
        for _ in 0..2 {
            step.perform_work_unit(|| Ok(()))?;
        }
        self.retained = Some(step.retain_bytes(4).map_err(step_failure)?);
        Err(WorthQueryGraphProviderFailure::new(
            "provider failed after governed work",
        ))
    }

    fn dispose(&mut self) -> Result<(), WorthQueryGraphProviderFailure> {
        Ok(())
    }
}
impl WorthQueryGraphParticipationProvider<ManagedGraph> for FailingProvider {
    type Execution = FailingExecution;

    fn execution_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport {
        crate::domain_computation::provider_session::tests::execution_resource_support(
            "managed-failing",
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
        admit_provider_execution(start, FailingExecution { retained: None })
    }
}
pub fn step_failure(
    denial: worth_query_execution::facade::domain_computation::WorthQueryGraphProviderStepDenial,
) -> WorthQueryGraphProviderFailure {
    WorthQueryGraphProviderFailure::new(denial.detail())
}
