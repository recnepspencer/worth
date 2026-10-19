//! Shared occurrence selection and candidate preparation; authorization is a typed caller closure.
use super::super::WorthQueryApplicationMutationAttemptReport;
use super::{
    map_selected_program_owner_denial, AdvancementPhase, ApplicationMutationBinding,
    ApplicationMutationIntent, ApplicationMutationScopeResolution, ApplicationProgramDefinition,
    ApplicationSchema, CandidatePreparation, ExecutionAllocationPolicy,
    WorthQueryApplicationMutationRequestWithIdempotency,
    WorthQueryApplicationProgramMutationPreparation, WorthQueryApplicationRequestMutationDenial,
    WorthQueryPreparedProgramMutation, WorthQueryProgramApplicationRuntime,
};
use worth_query_execution::facade::application_installation::WorthQueryProgramOwner;

impl<'application, 'principal, 'scope, 'key, Schema, Intent, SourcePreparation>
    WorthQueryApplicationMutationRequestWithIdempotency<
        'application,
        'principal,
        'scope,
        'key,
        Schema,
        Intent,
        SourcePreparation,
    >
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::Input: Clone + Send + Sync,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<
            Schema,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
{
    /// Runs the installed decision and candidate phases, retaining the unpublished
    /// result outside the commit lane. Call `commit` once ready to publish.
    pub fn prepare_in_program<'request, Program>(
        &'request mut self,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        WorthQueryApplicationProgramMutationPreparation<
            'request,
            'application,
            Schema,
            Intent::Binding,
            Program,
        >,
        WorthQueryApplicationRequestMutationDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
    {
        self.prepare_in_program_report(application, allocation_policy)
            .into_outcome()
    }

    /// Reports decision work without releasing the candidate's unpublished result.
    pub fn prepare_in_program_report<'request, Program>(
        &'request mut self,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationMutationAttemptReport<
        Result<
            WorthQueryApplicationProgramMutationPreparation<
                'request,
                'application,
                Schema,
                Intent::Binding,
                Program,
            >,
            WorthQueryApplicationRequestMutationDenial,
        >,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
    {
        let scope = self.request_scope().clone();
        let runtime = self.request.application;
        runtime
            .with_application_advancement(&scope, |phase| {
                let mut decision_work =
            worth_query_execution::facade::primary_graph::WorthQueryMutationHandlerWork::NotStarted;
                let outcome = self.prepare_in_program_with_work(
                    &phase,
                    application,
                    &mut decision_work,
                    allocation_policy,
                );
                WorthQueryApplicationMutationAttemptReport::new(outcome, decision_work)
            })
            .unwrap_or_else(|cause| {
                WorthQueryApplicationMutationAttemptReport::new(
            Err(WorthQueryApplicationRequestMutationDenial::ExecutionRequest(cause)),
            worth_query_execution::facade::primary_graph::WorthQueryMutationHandlerWork::NotStarted,
        )
            })
    }

    pub(in crate::application_entry::mutation) fn prepare_in_program_with_work<'request, Program>(
        &'request mut self,
        phase: &AdvancementPhase<'_>,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        decision_work: &mut worth_query_execution::facade::primary_graph::WorthQueryMutationHandlerWork,
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        WorthQueryApplicationProgramMutationPreparation<
            'request,
            'application,
            Schema,
            Intent::Binding,
            Program,
        >,
        WorthQueryApplicationRequestMutationDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
    {
        self.prepare_in_program_using(
            phase,
            application,
            decision_work,
            allocation_policy,
            |request, identities, staged, selected| {
                super::super::authorization::prepare_selected(
                    phase, request, identities, staged, selected,
                )
            },
        )
    }

    pub(super) fn prepare_in_program_using<'request, Program>(
        &'request mut self,
        phase: &AdvancementPhase<'_>,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        decision_work: &mut worth_query_execution::facade::primary_graph::WorthQueryMutationHandlerWork,
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
        prepare: impl FnOnce(
            &Self,
            &worth_query_declaration::facade::application_operation::ApplicationMutationIdentities<
                '_,
                Schema,
                Intent::Binding,
            >,
            super::super::staged::WorthQueryStagedMutation<Schema, Intent>,
            &worth_query_execution::facade::primary_graph::WorthQuerySelectedProductOperation<
                '_,
                Schema,
            >,
        ) -> Result<
            super::super::authorization::PreparedMutation<Schema, Intent::Binding>,
            WorthQueryApplicationRequestMutationDenial,
        >,
    ) -> Result<
        WorthQueryApplicationProgramMutationPreparation<
            'request,
            'application,
            Schema,
            Intent::Binding,
            Program,
        >,
        WorthQueryApplicationRequestMutationDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
    {
        if !std::ptr::eq(application.runtime(), self.request.application) {
            return Err(WorthQueryApplicationRequestMutationDenial::ApplicationProgramMismatch);
        }
        let selected = self
            .request
            .application
            .on_branch(self.request.branch)
            .select(phase)
            .map_err(WorthQueryApplicationRequestMutationDenial::ProductSelection)?;
        let owner = application
            .selected_program_owner(&selected)
            .map_err(map_selected_program_owner_denial)?;
        let selected_owns_action = owner.contains_action::<Intent::Binding>();
        if !selected_owns_action && !application.contains_action::<Intent::Binding>() {
            return Err(WorthQueryApplicationRequestMutationDenial::ApplicationProgramRequired);
        }
        match self.prepare_candidate(
            phase,
            move |request, identities, staged| prepare(request, identities, staged, &selected),
            decision_work,
            allocation_policy,
        )? {
            CandidatePreparation::Prepared(candidate) => {
                Ok(WorthQueryApplicationProgramMutationPreparation::Prepared(
                    WorthQueryPreparedProgramMutation {
                        candidate,
                        application,
                        selected_owner: selected_owns_action.then_some(owner),
                    },
                ))
            }
            CandidatePreparation::Settled(outcome) => Ok(
                WorthQueryApplicationProgramMutationPreparation::Settled(outcome),
            ),
        }
    }
}
