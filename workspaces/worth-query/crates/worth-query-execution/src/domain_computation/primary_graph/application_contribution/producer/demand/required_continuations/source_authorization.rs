//! Fresh disclosure admission for the actual required successor's source.

use super::*;

impl<Schema, Family> TypedRequiredSuccessor<Schema, Family>
where
    Schema: ApplicationSchema + 'static,
    Family: WorthQueryProducerOutputFamily<Schema>,
    FamilySourceValue<Schema, Family>:
        WorthQueryApplicationProjection<Schema, FamilySourceQuery<Schema, Family>> + 'static,
    FamilySourceQuery<Schema, Family>: 'static,
{
    pub(super) fn authorize_source_in_current_invocation(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        principal: &WorthQueryAuthenticatedExternalPrincipal<Schema>,
        request: &WorthQueryRequestScope,
        invocation: &ProducerInvocationAuthority<'_>,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        if invocation.history_control().is_none() {
            return Ok(());
        }
        let demand = self.demand.as_ref().ok_or_else(|| {
            WorthQueryOutputDemandDenial::new(
                super::super::WorthQueryOutputDemandDenialKind::RequiredSuccessorStateUnavailable,
                "required successor has no admitted source",
            )
        })?;
        super::super::progression::retained_read::authorize_history_source::<Schema, Family>(
            execution,
            runtime,
            demand.observed_source.source(),
            principal,
            request,
            invocation.history_control(),
        )
    }
}
