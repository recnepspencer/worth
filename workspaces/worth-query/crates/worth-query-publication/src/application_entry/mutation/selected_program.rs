use worth_query_declaration::facade::application_operation::{
    ApplicationCapabilityMutationBinding, ApplicationMutationBinding, ApplicationMutationIntent,
    ApplicationMutationScopeBinding, ApplicationMutationScopeResolution,
};
use worth_query_declaration::facade::application_program::ApplicationProgramDefinition;
use worth_query_execution::facade::application_installation::{
    WorthQueryProgramApplicationRuntime, WorthQueryProgramOwner,
    WorthQuerySelectedProgramOwnerDenial,
};
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
    pub fn execute_in_selected_program<Program>(
        self,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
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
        if !std::ptr::eq(application.runtime(), self.request.application) {
            return Err(WorthQueryApplicationRequestMutationDenial::ApplicationProgramMismatch);
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
            return Err(WorthQueryApplicationRequestMutationDenial::ApplicationProgramRequired);
        }
        self.execute_with_preparation_and_commit(
            move |request| super::authorization::prepare_selected(request, &selected),
            |_, program, idempotency| {
                if selected_owns_action {
                    owner.compare_and_commit_program_action::<Intent::Binding>(program, idempotency)
                } else {
                    application
                        .compare_and_commit_program_action::<Intent::Binding>(program, idempotency)
                }
            },
        )
    }

    /// Capability counterpart to [`Self::execute_in_selected_program`],
    /// including its fail-closed removed-action presentation.
    pub fn execute_capability_in_selected_program<Program>(
        self,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
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
        if !std::ptr::eq(application.runtime(), self.request.application) {
            return Err(WorthQueryApplicationRequestMutationDenial::ApplicationProgramMismatch);
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
            return Err(WorthQueryApplicationRequestMutationDenial::ApplicationProgramRequired);
        }
        self.execute_with_preparation_and_commit(
            move |request| super::authorization::prepare_capability_selected(request, &selected),
            |_, program, idempotency| {
                if selected_owns_action {
                    owner.compare_and_commit_program_action::<Intent::Binding>(program, idempotency)
                } else {
                    application
                        .compare_and_commit_program_action::<Intent::Binding>(program, idempotency)
                }
            },
        )
    }
}

pub(super) fn map_selected_program_owner_denial(
    denial: WorthQuerySelectedProgramOwnerDenial,
) -> WorthQueryApplicationRequestMutationDenial {
    match denial {
        WorthQuerySelectedProgramOwnerDenial::ProductSelection(denial) => {
            WorthQueryApplicationRequestMutationDenial::ProductSelection(denial)
        }
        WorthQuerySelectedProgramOwnerDenial::Inspection(_)
        | WorthQuerySelectedProgramOwnerDenial::InstalledOwnerUnavailable => {
            WorthQueryApplicationRequestMutationDenial::ApplicationProgramRequired
        }
    }
}
