//! The one program-gated commit path both owners share.
//!
//! Membership is checked against what the owner presents; activation is
//! checked against what the occurrence carries. Both must agree before any
//! effect is prepared, so a handle onto a rostered peer program cannot commit
//! on an occurrence that never activated it.

use std::any::TypeId;

use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationScopeBinding,
};
use worth_query_installation::facade::ApplicationSchema;

use super::{program_required, WorthQueryProgramOwner};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationEffectProgram,
    WorthQueryApplicationIdempotencyBinding, WorthQueryApplicationRetainedCommitOutcome,
};

type ActionProgram<Schema, Binding> = WorthQueryApplicationEffectProgram<
    Schema,
    <Binding as ApplicationMutationBinding<Schema>>::Operation,
    <Binding as ApplicationMutationBinding<Schema>>::Input,
    <<Binding as ApplicationMutationBinding<Schema>>::ScopeBinding as ApplicationMutationScopeBinding<
        Schema,
    >>::Scope,
>;

pub(super) fn commit_program_action<Schema, Binding, Owner>(
    owner: &Owner,
    program: ActionProgram<Schema, Binding>,
    idempotency: WorthQueryApplicationIdempotencyBinding,
) -> WorthQueryApplicationCommitOutcome
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
    Binding::Input: Clone + Send + Sync + 'static,
    Owner: WorthQueryProgramOwner<Schema> + ?Sized,
{
    let binding = TypeId::of::<Binding>();
    if !owner.owns_action(binding) || owner.owns_output_source(binding) {
        return WorthQueryApplicationCommitOutcome::Denied(program_required());
    }
    let runtime = owner.owned_runtime();
    let Some(support) = runtime.installed_program_support() else {
        return WorthQueryApplicationCommitOutcome::Denied(program_required());
    };
    let Some(presented) = support.present(owner.owned_revision()) else {
        return WorthQueryApplicationCommitOutcome::Denied(program_required());
    };
    runtime.compare_and_commit_application_for_program_action(&presented, program, idempotency)
}

pub(super) fn commit_program_action_retained<Schema, Binding, Owner>(
    owner: &Owner,
    program: ActionProgram<Schema, Binding>,
    idempotency: WorthQueryApplicationIdempotencyBinding,
) -> WorthQueryApplicationRetainedCommitOutcome
where
    Schema: ApplicationSchema,
    Binding: ApplicationMutationBinding<Schema>,
    Binding::Input: Clone + Send + Sync + 'static,
    Owner: WorthQueryProgramOwner<Schema> + ?Sized,
{
    let outcome = commit_program_action::<Schema, Binding, Owner>(
        owner,
        program.with_client_observation(),
        idempotency,
    );
    owner.owned_runtime().retained_commit_outcome(outcome)
}
