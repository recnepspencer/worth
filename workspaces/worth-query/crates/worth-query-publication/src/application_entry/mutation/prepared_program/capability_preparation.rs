//! Capability admission enters the same prepared-candidate funnel as ordinary actions.
use super::{
    ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationScopeResolution,
    ApplicationProgramDefinition, ApplicationSchema, ExecutionAllocationPolicy,
    WorthQueryApplicationMutationRequestWithIdempotency,
    WorthQueryApplicationProgramMutationPreparation, WorthQueryApplicationRequestMutationDenial,
    WorthQueryProgramApplicationRuntime,
};
use crate::application_entry::mutation::recorded_replay::RecordedReplay;
use worth_query_declaration::facade::{
    application_capability::ApplicationCapabilityRequest,
    application_operation::{
        ApplicationCapabilityMutationBinding, ApplicationMutationScopeBinding,
    },
};

impl<'application, 'principal, 'scope, 'key, Schema, Intent, SourcePreparation> WorthQueryApplicationMutationRequestWithIdempotency<'application, 'principal, 'scope, 'key, Schema, Intent, SourcePreparation>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Intent::Binding: ApplicationCapabilityMutationBinding<Schema>,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::Input: Clone + Send + Sync + ApplicationCapabilityRequest<Schema, <Intent::Binding as ApplicationCapabilityMutationBinding<Schema>>::Capability, Scope = <<Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope>,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding: ApplicationMutationScopeResolution<Schema, <Intent::Binding as ApplicationMutationBinding<Schema>>::PrincipalIdentity>,
{
    /// Prepares once under the installed capability policy. Authentication, when
    /// declared by the native body, signs this retained candidate before commit.
    pub fn prepare_capability_in_program<'request, Program>(
        &'request mut self,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<WorthQueryApplicationProgramMutationPreparation<'request, 'application, Schema, Intent::Binding, Program>, WorthQueryApplicationRequestMutationDenial>
    where Program: ApplicationProgramDefinition<Schema>,
    {
        match self.recorded_replay(allocation_policy, |phase, request, identities, staged| {
            let selected = request.select_program_owner(phase, application)?.selected;
            super::super::authorization::prepare_capability_selected(phase, request, identities, staged, &selected)
        }) {
            RecordedReplay::Replayed(outcome) => {
                return Ok(WorthQueryApplicationProgramMutationPreparation::Settled(outcome));
            }
            RecordedReplay::Refused(refusal) => return Err(refusal),
            RecordedReplay::Miss => {}
        }
        let scope = self.request_scope().clone();
        let runtime = self.request.application;
        runtime.with_funded_application_advancement(&scope, allocation_policy, |phase| {
            let mut work = worth_query_execution::facade::primary_graph::WorthQueryMutationHandlerWork::NotStarted;
            self.prepare_in_program_using(&phase, application, &mut work, allocation_policy,
                |request, identities, staged, selected| super::super::authorization::prepare_capability_selected(&phase, request, identities, staged, selected))
        }).map_err(WorthQueryApplicationRequestMutationDenial::ExecutionRequest)?
    }
}
