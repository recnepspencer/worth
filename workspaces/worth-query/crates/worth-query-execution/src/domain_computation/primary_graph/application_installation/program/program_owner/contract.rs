use super::*;
use crate::domain_computation::primary_graph::WorthQueryAdvancementPhase;

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
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Self: Sized,
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
        Binding::Input: Clone + Send + Sync + 'static,
    {
        let request = program.request_scope().clone();
        self.runtime()
            .with_application_advancement(&request, |active_phase| {
                self.commit_program_action_in_advancement(
                    &active_phase,
                    program,
                    identities,
                    extend,
                    allocation_policy,
                )
            })
            .unwrap_or_else(|denial| denial.into_commit_outcome())
    }

    /// Commits an internal phase without opening another request.
    fn commit_program_action_in_advancement<Binding>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
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
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationCommitOutcome
    where
        Self: Sized,
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
        Binding::Input: Clone + Send + Sync + 'static,
    {
        commit_program_action::<Schema, Binding, Self>(
            phase,
            self,
            program,
            ApplicationCommitCausality::Ordinary,
            extend(WorthQueryApplicationIdempotencyBinding::for_mutation_identities(identities)),
            allocation_policy,
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
        let request = program.request_scope().clone();
        self.runtime()
            .with_application_advancement(&request, |active_phase| {
                commit_program_action::<Schema, Binding, Self>(
                    &active_phase,
                    self,
                    program,
                    ApplicationCommitCausality::undo(handoff),
                    extend(
                        WorthQueryApplicationIdempotencyBinding::for_mutation_identities(
                            identities,
                        ),
                    ),
                    crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
            })
            .unwrap_or_else(|denial| denial.into_commit_outcome())
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
        let request = program.request_scope().clone();
        self.runtime()
            .with_application_advancement(&request, |active_phase| {
                commit_program_action::<Schema, Binding, Self>(
                    &active_phase,
                    self,
                    program,
                    ApplicationCommitCausality::redo(handoff),
                    extend(
                        WorthQueryApplicationIdempotencyBinding::for_mutation_identities(
                            identities,
                        ),
                    ),
                    crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
            })
            .unwrap_or_else(|denial| denial.into_commit_outcome())
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
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationRetainedCommitOutcome
    where
        Self: Sized,
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
        Binding::Input: Clone + Send + Sync + 'static,
    {
        let request = program.request_scope().clone();
        self.runtime()
            .with_application_advancement(&request, |active_phase| {
                self.commit_program_action_retained_in_advancement(
                    &active_phase,
                    program,
                    identities,
                    extend,
                    allocation_policy,
                )
            })
            .unwrap_or_else(|denial| {
                self.runtime()
                    .retained_commit_outcome(denial.into_commit_outcome())
            })
    }

    /// Commits an internal phase without opening another request.
    fn commit_program_action_retained_in_advancement<Binding>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
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
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> WorthQueryApplicationRetainedCommitOutcome
    where
        Self: Sized,
        Schema: ApplicationSchema,
        Binding: ApplicationMutationBinding<Schema>,
        Binding::Input: Clone + Send + Sync + 'static,
    {
        commit_program_action_retained::<Schema, Binding, Self>(
            phase,
            self,
            program,
            extend(WorthQueryApplicationIdempotencyBinding::for_mutation_identities(identities)),
            allocation_policy,
        )
    }
}
