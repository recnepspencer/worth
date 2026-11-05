use std::sync::Arc;

use super::{WorthQueryPrimaryGraphProvider, WorthQueryPrimaryLogicalGraph};
use {crate::domain_computation::WorthQueryCooperativeGraphProviderExecution, crate::domain_computation::WorthQueryGraphParticipationProvider, crate::domain_computation::WorthQueryGraphProviderCall, crate::domain_computation::WorthQueryGraphProviderExecution, crate::domain_computation::WorthQueryGraphProviderExecutionStart, crate::domain_computation::vocabulary::provider_session::failure::WorthQueryGraphProviderFailure, crate::domain_computation::WorthQueryGraphProviderStep, crate::domain_computation::WorthQueryGraphProviderStepDisposition};

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
    for Arc<WorthQueryPrimaryGraphProvider>
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
