use super::WorthQueryPrimaryLogicalGraph;
use crate::domain_computation::{
    WorthQueryCooperativeGraphProviderExecution, WorthQueryGraphParticipationProvider,
    WorthQueryGraphProviderCall, WorthQueryGraphProviderExecution,
    WorthQueryGraphProviderExecutionStart, WorthQueryGraphProviderFailure,
    WorthQueryGraphProviderStep, WorthQueryGraphProviderStepDisposition,
};

#[doc(hidden)]
pub struct WorthQueryPrimaryGraphUnusedExecution;

impl WorthQueryGraphProviderExecution for WorthQueryPrimaryGraphUnusedExecution {
    fn advance(
        &mut self,
        _step: &mut WorthQueryGraphProviderStep,
    ) -> Result<WorthQueryGraphProviderStepDisposition, WorthQueryGraphProviderFailure> {
        Err(WorthQueryGraphProviderFailure::new(
            "primary application graph uses the sealed provider-session progression",
        ))
    }

    fn dispose(&mut self) -> Result<(), WorthQueryGraphProviderFailure> {
        Ok(())
    }
}

impl WorthQueryGraphParticipationProvider<WorthQueryPrimaryLogicalGraph>
    for crate::domain_computation::runtime_ports::PrimaryGraphProviderArc
{
    type Execution = WorthQueryPrimaryGraphUnusedExecution;

    fn execution_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport {
        self.resource_support.graph()
    }

    fn begin(
        &self,
        _call: &WorthQueryGraphProviderCall,
        _start: &mut WorthQueryGraphProviderExecutionStart,
    ) -> Result<
        WorthQueryCooperativeGraphProviderExecution<Self::Execution>,
        WorthQueryGraphProviderFailure,
    > {
        Err(WorthQueryGraphProviderFailure::new(
            "primary application graph requires a sealed provider session",
        ))
    }
}
