//! Who may present a program for one program-gated commit.
//!
//! Exactly two owners exist: the runtime published with a host's initial
//! program, and a typed handle onto another program that host rostered. The
//! trait is sealed so no third owner can appear, and every program-gated
//! commit entry point accepts owners only through it. Owning a program is not
//! activating it: the shared implementations resolve the occurrence's active
//! program before any effect, and refuse a presented program that is not it.

use std::any::TypeId;

use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationScopeBinding,
};
use worth_query_declaration::facade::application_program::{
    ApplicationProgramDefinition, ApplicationProgramRevision,
};
use worth_query_installation::facade::ApplicationSchema;

use super::supported_program::WorthQuerySupportedProgramHandle;
use super::WorthQueryProgramApplicationRuntime;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationEffectProgram, WorthQueryApplicationIdempotencyBinding,
    WorthQueryApplicationRetainedCommitOutcome, WorthQueryPrimaryGraphApplicationRuntime,
};

mod occurrence_commit;

use occurrence_commit::{commit_program_action, commit_program_action_retained};

mod sealed {
    /// Closes the set of program owners at the two the platform publishes.
    pub trait WorthQueryProgramOwnership {}
}

/// One owner of a rostered program on one published host.
pub trait WorthQueryProgramOwner<Schema>: sealed::WorthQueryProgramOwnership {
    /// The plain application runtime this owner commits through.
    fn owned_runtime(&self) -> &WorthQueryPrimaryGraphApplicationRuntime<Schema>;

    /// The canonical revision this owner presents.
    fn owned_revision(&self) -> &ApplicationProgramRevision;

    /// Whether the owned program acts through one mutation binding.
    fn owns_action(&self, binding: TypeId) -> bool;

    /// Whether one mutation binding is an output source rather than an action.
    fn owns_output_source(&self, binding: TypeId) -> bool;

    fn runtime(&self) -> &WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        self.owned_runtime()
    }

    fn contains_action<Binding: 'static>(&self) -> bool {
        self.owns_action(TypeId::of::<Binding>())
    }

    /// Commits one action through the program this owner presents, only when
    /// that program is the one this occurrence activated.
    fn compare_and_commit_program_action<Binding>(
        &self,
        program: WorthQueryApplicationEffectProgram<
            Schema,
            Binding::Operation,
            Binding::Input,
            <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Self: Sized,
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
        Binding::Input: Clone + Send + Sync + 'static,
    {
        commit_program_action::<Schema, Binding, Self>(self, program, idempotency)
    }

    /// Commits one action through the presented program and retains its
    /// client-observed result.
    fn compare_and_commit_program_action_retained<Binding>(
        &self,
        program: WorthQueryApplicationEffectProgram<
            Schema,
            Binding::Operation,
            Binding::Input,
            <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
        idempotency: WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryApplicationRetainedCommitOutcome
    where
        Self: Sized,
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
        Binding::Input: Clone + Send + Sync + 'static,
    {
        commit_program_action_retained::<Schema, Binding, Self>(self, program, idempotency)
    }
}

impl<Schema, Program> sealed::WorthQueryProgramOwnership
    for WorthQueryProgramApplicationRuntime<Schema, Program>
{
}

impl<Schema, Program> WorthQueryProgramOwner<Schema>
    for WorthQueryProgramApplicationRuntime<Schema, Program>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    fn owned_runtime(&self) -> &WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        self.runtime()
    }

    fn owned_revision(&self) -> &ApplicationProgramRevision {
        self.installed_program().revision()
    }

    fn owns_action(&self, binding: TypeId) -> bool {
        self.declares_action_binding(binding)
    }

    fn owns_output_source(&self, binding: TypeId) -> bool {
        self.requires_output_source(binding)
    }
}

impl<Schema, Program> sealed::WorthQueryProgramOwnership
    for WorthQuerySupportedProgramHandle<'_, Schema, Program>
{
}

impl<Schema, Program> WorthQueryProgramOwner<Schema>
    for WorthQuerySupportedProgramHandle<'_, Schema, Program>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
{
    fn owned_runtime(&self) -> &WorthQueryPrimaryGraphApplicationRuntime<Schema> {
        self.runtime()
    }

    fn owned_revision(&self) -> &ApplicationProgramRevision {
        self.record().revision()
    }

    fn owns_action(&self, binding: TypeId) -> bool {
        self.record().declares_action(binding)
    }

    fn owns_output_source(&self, binding: TypeId) -> bool {
        self.record().requires_output_source(binding)
    }
}

/// The denial every owner reaches for when it presents meaning this host
/// cannot attribute to an admitted rostered program.
pub(super) fn program_required() -> WorthQueryApplicationCommitDenial {
    WorthQueryApplicationCommitDenial::application_program_required()
}
