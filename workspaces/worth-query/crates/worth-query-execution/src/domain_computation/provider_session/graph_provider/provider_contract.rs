use super::{
    WorthQueryCooperativeGraphProviderExecution, WorthQueryGraphCommitCall,
    WorthQueryGraphProviderCall, WorthQueryGraphProviderExecution,
    WorthQueryGraphProviderExecutionStart, WorthQueryGraphProviderFailure,
    WorthQueryGraphProviderReceipt,
};

pub trait WorthQueryGraphParticipationProvider<G>: Send + Sync + 'static {
    type Execution: WorthQueryGraphProviderExecution;

    fn execution_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport;

    fn begin(
        &self,
        call: &WorthQueryGraphProviderCall,
        start: &mut WorthQueryGraphProviderExecutionStart,
    ) -> Result<
        WorthQueryCooperativeGraphProviderExecution<Self::Execution>,
        WorthQueryGraphProviderFailure,
    >;
}

pub trait WorthQueryGraphCommitProvider<C>: Send + Sync + 'static {
    fn execution_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport;

    fn admit_commit(
        &self,
        call: &WorthQueryGraphCommitCall,
    ) -> Result<WorthQueryGraphProviderReceipt, WorthQueryGraphProviderFailure>;
}

/// Physical provider mechanics for the sealed session protocol.
///
/// Query supplies the one-use token admission capability. A provider can
/// acknowledge and name its physical session, but cannot construct Query
/// session authority or advance the protocol on its own.
pub trait WorthQueryProviderSessionLifecycle: Send + Sync + 'static {
    fn readmit_provider_plan(
        &self,
        plan: &crate::domain_computation::WorthQueryProviderExecutionPlanView<'_>,
        admission: crate::domain_computation::WorthQueryProviderSessionTokenAdmission,
    ) -> Result<
        crate::domain_computation::WorthQueryProviderSessionToken,
        crate::domain_computation::WorthQueryProviderSessionFailure,
    >;

    fn prepare_provider_session(
        &self,
        session: &crate::domain_computation::WorthQueryProviderSessionView<'_>,
    ) -> Result<(), crate::domain_computation::WorthQueryProviderSessionFailure>;

    fn prepare_staged_session(
        &self,
        session: &crate::domain_computation::WorthQueryProviderSessionView<'_>,
    ) -> Result<(), crate::domain_computation::WorthQueryProviderSessionFailure>;

    fn commit_prepared_session(
        &self,
        session: &crate::domain_computation::WorthQueryProviderSessionView<'_>,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        crate::domain_computation::WorthQueryProviderTerminalDescription,
        crate::domain_computation::WorthQueryProviderSessionCommitStop,
    >;

    fn abort_provider_session(
        &self,
        session: &crate::domain_computation::WorthQueryProviderSessionView<'_>,
    ) -> Result<
        crate::domain_computation::WorthQueryProviderTerminalDescription,
        crate::domain_computation::WorthQueryProviderSessionFailure,
    >;
}
