//! Who may present a program for one program-gated commit.
//!
//! Exactly two owner categories exist: the runtime published with a host's
//! initial program, and handles onto programs that host rostered. A selected
//! owner is a branch-resolved form of either category. The trait is sealed so no
//! outside owner can appear, and every program-gated
//! commit entry point accepts owners only through it. Owning a program is not
//! activating it: the shared implementations resolve the occurrence's active
//! program before any effect, and refuse a presented program that is not it.

use std::any::TypeId;

use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIdentities, ApplicationMutationScopeBinding,
};
use worth_query_declaration::facade::application_program::{
    ApplicationProgramDefinition, ApplicationProgramRevision, ApplicationWorkflowSpec,
};
use worth_query_installation::facade::ApplicationSchema;

use super::supported_program::WorthQuerySupportedProgramHandle;
use super::{WorthQueryProgramApplicationRuntime, WorthQueryWorkflowApplicationRuntime};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationEffectProgram, WorthQueryApplicationIdempotencyBinding,
    WorthQueryApplicationRetainedCommitOutcome, WorthQueryPrimaryGraphApplicationRuntime,
};

mod occurrence_commit;
mod ownership;
mod selected_owner;
use crate::domain_computation::application_aftermath::ApplicationCommitCausality;

use occurrence_commit::{commit_program_action, commit_program_action_retained};

mod sealed {
    /// Closes program ownership to the two categories the platform publishes.
    pub trait WorthQueryProgramOwnership {}
}

/// A commit owner resolved from the program carried by one live, typed branch
/// selection. Its fields are private so a revision or lookalike activation
/// cannot manufacture commit authority.
pub struct WorthQuerySelectedProgramOwner<'runtime, Schema> {
    runtime: &'runtime WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    revision: &'runtime ApplicationProgramRevision,
    action_bindings: &'runtime [TypeId],
    output_source_bindings: &'runtime [TypeId],
    root_graph_types: &'runtime [TypeId],
}

#[derive(Debug)]
pub enum WorthQuerySelectedProgramOwnerDenial {
    ProductSelection(
        crate::domain_computation::primary_graph::WorthQueryProductBranchAdmissionDenial,
    ),
    Inspection(crate::domain_computation::primary_graph::WorthQuerySelectedProgramInspectionDenial),
    InstalledOwnerUnavailable,
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
    ///
    /// The commit's idempotency binding is built here from `identities`, the key
    /// and input identities the request encoded once, and names `Binding` as the
    /// mutation it runs, so the binding cannot disagree with the request. `extend`
    /// adds what only the caller knows, such as an accepted source expectation;
    /// pass `std::convert::identity` when there is nothing to add. A program built
    /// by another binding's handler is refused with `MutationBindingMismatch`, and
    /// one whose handler decided on a different input than `identities` is refused
    /// with `MutationInputMismatch`.
    fn compare_and_commit_program_action<Binding>(
        &self,
        program: WorthQueryApplicationEffectProgram<
            Schema,
            Binding::Operation,
            Binding::Input,
            <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
        identities: &ApplicationMutationIdentities<'_, Schema, Binding>,
        extend: impl FnOnce(
            WorthQueryApplicationIdempotencyBinding,
        ) -> WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Self: Sized,
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
        Binding::Input: Clone + Send + Sync + 'static,
    {
        commit_program_action::<Schema, Binding, Self>(
            self,
            program,
            ApplicationCommitCausality::Ordinary,
            extend(WorthQueryApplicationIdempotencyBinding::for_mutation_identities(identities)),
        )
    }

    /// Commits an admitted undo through the same occurrence and action gates.
    fn compare_and_commit_program_undo<Binding>(
        &self,
        program: WorthQueryApplicationEffectProgram<
            Schema,
            Binding::Operation,
            Binding::Input,
            <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
        identities: &ApplicationMutationIdentities<'_, Schema, Binding>,
        extend: impl FnOnce(
            WorthQueryApplicationIdempotencyBinding,
        ) -> WorthQueryApplicationIdempotencyBinding,
        handoff: &crate::domain_computation::application_aftermath::WorthQueryUndoProgressionHandoff,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Self: Sized,
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
        Binding::Input: Clone + Send + Sync + 'static,
    {
        commit_program_action::<Schema, Binding, Self>(
            self,
            program,
            ApplicationCommitCausality::undo(handoff),
            extend(WorthQueryApplicationIdempotencyBinding::for_mutation_identities(identities)),
        )
    }

    /// Commits an admitted redo through the same occurrence and action gates.
    fn compare_and_commit_program_redo<Binding>(
        &self,
        program: WorthQueryApplicationEffectProgram<
            Schema,
            Binding::Operation,
            Binding::Input,
            <Binding::ScopeBinding as ApplicationMutationScopeBinding<Schema>>::Scope,
        >,
        identities: &ApplicationMutationIdentities<'_, Schema, Binding>,
        extend: impl FnOnce(
            WorthQueryApplicationIdempotencyBinding,
        ) -> WorthQueryApplicationIdempotencyBinding,
        handoff: &crate::domain_computation::application_aftermath::WorthQueryRedoProgressionHandoff,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Self: Sized,
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
        Binding::Input: Clone + Send + Sync + 'static,
    {
        commit_program_action::<Schema, Binding, Self>(
            self,
            program,
            ApplicationCommitCausality::redo(handoff),
            extend(WorthQueryApplicationIdempotencyBinding::for_mutation_identities(identities)),
        )
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
        identities: &ApplicationMutationIdentities<'_, Schema, Binding>,
        extend: impl FnOnce(
            WorthQueryApplicationIdempotencyBinding,
        ) -> WorthQueryApplicationIdempotencyBinding,
    ) -> WorthQueryApplicationRetainedCommitOutcome
    where
        Self: Sized,
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
        Binding::Input: Clone + Send + Sync + 'static,
    {
        commit_program_action_retained::<Schema, Binding, Self>(
            self,
            program,
            extend(WorthQueryApplicationIdempotencyBinding::for_mutation_identities(identities)),
        )
    }
}

/// The denial every owner reaches for when it presents meaning this host
/// cannot attribute to an admitted rostered program.
pub(super) fn program_required() -> WorthQueryApplicationCommitDenial {
    WorthQueryApplicationCommitDenial::application_program_required()
}
