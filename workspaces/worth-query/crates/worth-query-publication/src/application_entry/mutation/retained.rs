use std::cell::RefCell;
use std::sync::Arc;

use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIntent, ApplicationMutationScopeBinding,
    ApplicationMutationScopeResolution,
};
use worth_query_declaration::facade::application_program::ApplicationProgramDefinition;
use worth_query_execution::facade::application_installation::WorthQueryProgramOwner;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationCommitReceipt,
    WorthQueryApplicationEffectProgram, WorthQueryApplicationIdempotencyBinding,
    WorthQueryApplicationReadObservation as RetainedRead,
    WorthQueryApplicationRetainedCommitOutcome, WorthQueryPrimaryGraphApplicationRuntime,
};
use worth_query_installation::facade::ApplicationSchema;

use super::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationMutationRequestWithIdempotency,
};
use crate::application_entry::{
    WorthQueryApplicationReadObservation, WorthQueryApplicationRequestMutationDenial,
};

type Binding<Schema, Intent> = <Intent as ApplicationMutationIntent<Schema>>::Binding;
type Scope<Schema, Intent> = <<Binding<Schema, Intent> as ApplicationMutationBinding<Schema>>::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope;
type Outcome<Schema, Intent> = WorthQueryApplicationMutationOutcome<
    <Binding<Schema, Intent> as ApplicationMutationBinding<Schema>>::Denial,
    <Binding<Schema, Intent> as ApplicationMutationBinding<Schema>>::Result,
>;

pub enum WorthQueryApplicationRetainedMutationOutcome<Denial, Result> {
    Committed {
        receipt: WorthQueryApplicationCommitReceipt,
        result: Result,
        retained: WorthQueryApplicationReadObservation,
    },
    Other(WorthQueryApplicationMutationOutcome<Denial, Result>),
}

impl<Denial, Result> WorthQueryApplicationRetainedMutationOutcome<Denial, Result> {
    pub const fn retained_read(&self) -> Option<&WorthQueryApplicationReadObservation> {
        match self {
            Self::Committed { retained, .. } => Some(retained),
            Self::Other(_) => None,
        }
    }
}

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
    pub fn execute_retained(
        self,
    ) -> Result<
        WorthQueryApplicationRetainedMutationOutcome<
            <Intent::Binding as ApplicationMutationBinding<Schema>>::Denial,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::Result,
        >,
        WorthQueryApplicationRequestMutationDenial,
    > {
        if self
            .request
            .application
            .requires_application_program::<Intent::Binding>()
        {
            return Err(WorthQueryApplicationRequestMutationDenial::ApplicationProgramRequired);
        }
        self.execute_retained_with_commit(
            super::authorization::prepare,
            |application, program, idempotency| {
                application.compare_and_commit_application_retained(program, idempotency)
            },
        )
    }

    pub fn execute_retained_in_program<Owner>(
        self,
        application: &'application Owner,
    ) -> Result<
        WorthQueryApplicationRetainedMutationOutcome<
            <Intent::Binding as ApplicationMutationBinding<Schema>>::Denial,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::Result,
        >,
        WorthQueryApplicationRequestMutationDenial,
    >
    where
        Owner: WorthQueryProgramOwner<Schema>,
    {
        if !std::ptr::eq(application.runtime(), self.request.application) {
            return Err(WorthQueryApplicationRequestMutationDenial::ApplicationProgramMismatch);
        }
        if !application.contains_action::<Intent::Binding>() {
            return Err(WorthQueryApplicationRequestMutationDenial::ApplicationProgramRequired);
        }
        self.execute_retained_with_commit(
            super::authorization::prepare,
            |_, program, idempotency| {
                application.compare_and_commit_program_action_retained::<Intent::Binding>(
                    program,
                    idempotency,
                )
            },
        )
    }

    /// Executes retained work under the branch-selected installed program.
    /// A removed action is presented through the initial owner only so the
    /// occurrence gate can return its inactive-program denial; it cannot
    /// commit or consume the retained work.
    pub fn execute_retained_in_selected_program<Program>(
        self,
        application: &'application worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime<Schema, Program>,
    ) -> Result<
        WorthQueryApplicationRetainedMutationOutcome<
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
            .map_err(super::selected_program::map_selected_program_owner_denial)?;
        let selected_owns_action = owner.contains_action::<Intent::Binding>();
        if !selected_owns_action && !application.contains_action::<Intent::Binding>() {
            return Err(WorthQueryApplicationRequestMutationDenial::ApplicationProgramRequired);
        }
        self.execute_retained_with_commit(
            move |request| super::authorization::prepare_selected(request, &selected),
            |_, program, idempotency| {
                if selected_owns_action {
                    owner.compare_and_commit_program_action_retained::<Intent::Binding>(
                        program,
                        idempotency,
                    )
                } else {
                    application.compare_and_commit_program_action_retained::<Intent::Binding>(
                        program,
                        idempotency,
                    )
                }
            },
        )
    }

    fn execute_retained_with_commit(
        self,
        prepare: impl FnOnce(
            &mut Self,
        ) -> Result<
            super::authorization::PreparedMutation<Schema, Intent::Binding>,
            WorthQueryApplicationRequestMutationDenial,
        >,
        commit: impl FnOnce(
            &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
            WorthQueryApplicationEffectProgram<
                Schema,
                <Intent::Binding as ApplicationMutationBinding<Schema>>::Operation,
                <Intent::Binding as ApplicationMutationBinding<Schema>>::Input,
                Scope<Schema, Intent>,
            >,
            WorthQueryApplicationIdempotencyBinding,
        ) -> WorthQueryApplicationRetainedCommitOutcome,
    ) -> Result<
        WorthQueryApplicationRetainedMutationOutcome<
            <Intent::Binding as ApplicationMutationBinding<Schema>>::Denial,
            <Intent::Binding as ApplicationMutationBinding<Schema>>::Result,
        >,
        WorthQueryApplicationRequestMutationDenial,
    > {
        let retained: RefCell<Option<Arc<RetainedRead>>> = RefCell::new(None);
        let outcome: Outcome<Schema, Intent> = self.execute_with_preparation_and_commit(
            prepare,
            |application, program, idempotency| match commit(application, program, idempotency) {
                WorthQueryApplicationRetainedCommitOutcome::Committed {
                    receipt,
                    retained: observation,
                } => {
                    retained.replace(Some(observation));
                    WorthQueryApplicationCommitOutcome::Committed(receipt)
                }
                WorthQueryApplicationRetainedCommitOutcome::Other(outcome) => outcome,
            },
        )?;
        Ok(match outcome {
            WorthQueryApplicationMutationOutcome::Committed { receipt, result } => {
                let observation = retained
                    .into_inner()
                    .expect("only the opt-in performed commit can return Committed");
                WorthQueryApplicationRetainedMutationOutcome::Committed {
                    receipt,
                    result,
                    retained: WorthQueryApplicationReadObservation::new(observation),
                }
            }
            other => WorthQueryApplicationRetainedMutationOutcome::Other(other),
        })
    }
}
