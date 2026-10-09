use worth_query_declaration::facade::application_operation::{
    ApplicationCapabilityMutationBinding, ApplicationMutationBinding, ApplicationMutationIntent,
    ApplicationMutationScopeBinding, ApplicationMutationScopeResolution,
};
use worth_query_declaration::facade::application_program::ApplicationProgramDefinition;
use worth_query_execution::facade::application_installation::{
    WorthQueryProgramApplicationRuntime, WorthQueryProgramOwner,
    WorthQuerySelectedProgramOwnerDenial,
};
use worth_query_execution::facade::runtime::ExecutionAllocationPolicy;
use worth_query_installation::facade::ApplicationSchema;

use super::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationMutationRequestWithIdempotency,
};
use crate::application_entry::WorthQueryApplicationRequestMutationDenial;

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
    Intent: ApplicationMutationIntent<Schema> + Clone + Send + Sync,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::Input: Clone + Send + Sync,
    <Intent::Binding as ApplicationMutationBinding<Schema>>::ScopeBinding:
        ApplicationMutationScopeResolution<
            Schema,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::PrincipalIdentity,
        >,
{
    /// Resolves the commit owner from the program carried by this request's
    /// exact branch, then executes through that installed owner. No caller-
    /// supplied revision or activation receipt participates in selection.
    ///
    /// If the selected program removed this action, the initial installed
    /// owner is presented only so the authoritative occurrence gate can
    /// publish its established inactive-program denial. That presentation
    /// cannot commit the removed action or perform its external effects.
    ///
    /// A retried key replays its recorded outcome before any commit, even
    /// after the branch adopts another program. Only a host that does not
    /// roster the branch's current program refuses the retry, at owner
    /// resolution, before the replay is consulted.
    pub fn execute_in_program<Program>(
        self,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        WorthQueryApplicationMutationOutcome<
            <Intent::Binding as ApplicationMutationBinding<Schema>>::Denial,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::Result,
        >,
        WorthQueryApplicationRequestMutationDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
    {
        self.execute_in_program_report(application, allocation_policy)
            .into_outcome()
    }

    /// The same selected-program execution, with this attempt's sealed decision work.
    pub fn execute_in_program_report<Program>(
        self,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> super::WorthQueryApplicationMutationAttemptReport<
        Result<
            WorthQueryApplicationMutationOutcome<
                <Intent::Binding as ApplicationMutationBinding<Schema>>::Denial,
                <Intent::Binding as ApplicationMutationBinding<Schema>>::Result,
            >,
            WorthQueryApplicationRequestMutationDenial,
        >,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
    {
        let scope = self.request_scope().clone();
        let runtime = self.request.application;
        runtime.with_application_advancement(&scope, |phase| {
            let mut request = self;
            let mut decision_work = worth_query_execution::facade::primary_graph::WorthQueryMutationHandlerWork::NotStarted;
            let outcome = request.prepare_in_program_with_work(&phase, application, &mut decision_work, allocation_policy);
            match outcome {
                Ok(super::WorthQueryApplicationProgramMutationPreparation::Prepared(candidate)) => candidate.commit_report_in_advancement(&phase, allocation_policy).map(Ok),
                Ok(super::WorthQueryApplicationProgramMutationPreparation::Settled(outcome)) => super::WorthQueryApplicationMutationAttemptReport::new(Ok(outcome), decision_work),
                Err(denial) => super::WorthQueryApplicationMutationAttemptReport::new(Err(denial), decision_work),
            }
        }).unwrap_or_else(|cause| super::WorthQueryApplicationMutationAttemptReport::new(
            Err(WorthQueryApplicationRequestMutationDenial::ExecutionRequest(cause)),
            worth_query_execution::facade::primary_graph::WorthQueryMutationHandlerWork::NotStarted,
        ))
    }

    /// Capability counterpart to [`Self::execute_in_program`],
    /// including its fail-closed removed-action presentation.
    pub fn execute_capability_in_program<Program>(
        self,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        WorthQueryApplicationMutationOutcome<
            <Intent::Binding as ApplicationMutationBinding<Schema>>::Denial,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::Result,
        >,
        WorthQueryApplicationRequestMutationDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        Intent::Binding: ApplicationCapabilityMutationBinding<Schema>,
        <Intent::Binding as ApplicationMutationBinding<Schema>>::Input:
            worth_query_declaration::facade::application_capability::ApplicationCapabilityRequest<
                Schema,
                <Intent::Binding as ApplicationCapabilityMutationBinding<Schema>>::Capability,
                Scope = <<Intent::Binding as ApplicationMutationBinding<
                    Schema,
                >>::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
            >,
    {
        let request_scope = self.request_scope().clone();
        let runtime = self.request.application;
        runtime
            .with_application_advancement(&request_scope, |active_phase| {
                let phase = &active_phase;

                if !std::ptr::eq(application.runtime(), self.request.application) {
                    return Err(
                        WorthQueryApplicationRequestMutationDenial::ApplicationProgramMismatch,
                    );
                }
                let selected = self
                    .request
                    .application
                    .on_branch(self.request.branch)
                    .select()
                    .map_err(WorthQueryApplicationRequestMutationDenial::ProductSelection)?;
                let owner = application
                    .selected_program_owner(&selected)
                    .map_err(map_selected_program_owner_denial)?;
                let selected_owns_action = owner.contains_action::<Intent::Binding>();
                if !selected_owns_action && !application.contains_action::<Intent::Binding>() {
                    return Err(
                        WorthQueryApplicationRequestMutationDenial::ApplicationProgramRequired,
                    );
                }
                self.execute_with_preparation_and_commit(
                    phase,
                    move |request, identities, staged| {
                        super::authorization::prepare_capability_selected(
                            request, identities, staged, &selected,
                        )
                    },
                    |_, program, binding| {
                        if selected_owns_action {
                            owner.commit_program_action_in_advancement(
                                phase,
                                program,
                                binding.identities(),
                                |idempotency| binding.extension().apply(idempotency),
                                allocation_policy,
                            )
                        } else {
                            application.commit_program_action_in_advancement(
                                phase,
                                program,
                                binding.identities(),
                                |idempotency| binding.extension().apply(idempotency),
                                allocation_policy,
                            )
                        }
                    },
                    allocation_policy,
                )
            })
            .map_err(WorthQueryApplicationRequestMutationDenial::ExecutionRequest)?
    }
}

pub(super) fn map_selected_program_owner_denial(
    denial: WorthQuerySelectedProgramOwnerDenial,
) -> WorthQueryApplicationRequestMutationDenial {
    match denial {
        WorthQuerySelectedProgramOwnerDenial::ProductSelection(denial) => {
            WorthQueryApplicationRequestMutationDenial::ProductSelection(denial)
        }
        WorthQuerySelectedProgramOwnerDenial::Inspection(denial) => {
            WorthQueryApplicationRequestMutationDenial::ProgramSelection(denial)
        }
        WorthQuerySelectedProgramOwnerDenial::InstalledOwnerUnavailable => {
            WorthQueryApplicationRequestMutationDenial::ApplicationProgramRequired
        }
    }
}
