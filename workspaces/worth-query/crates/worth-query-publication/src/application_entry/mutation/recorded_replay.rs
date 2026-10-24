//! A recorded outcome answers its retry before the caller's lease funds any work.
use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBinding, ApplicationMutationIdentities, ApplicationMutationIntent,
    ApplicationMutationScopeResolution,
};
use worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase as AdvancementPhase;
use worth_query_execution::facade::runtime::ExecutionAllocationPolicy;
use worth_query_installation::facade::ApplicationSchema;

use super::{
    authorization::PreparedMutation, commit_binding::WorthQueryMutationCommitBinding,
    staged::WorthQueryStagedMutation, WorthQueryApplicationMutationOutcome,
    WorthQueryApplicationMutationRequestWithIdempotency,
};

type Outcome<Schema, Binding> = WorthQueryApplicationMutationOutcome<
    <Binding as ApplicationMutationBinding<Schema>>::Denial,
    <Binding as ApplicationMutationBinding<Schema>>::Result,
>;

impl<Schema, Intent, SourcePreparation>
    WorthQueryApplicationMutationRequestWithIdempotency<
        '_,
        '_,
        '_,
        '_,
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
    /// The one pre-admission step of every lane that prepares fresh under a
    /// caller's lease: the lane's own checks and authorization (`prepare`),
    /// then the key's resolution, in the World's placement and before the
    /// funded opening. A recorded outcome returns here, so its retry never
    /// opens, charges or needs the caller's lease. A miss, or any refusal on
    /// the way, returns `None`; the lane then opens its funded call exactly as
    /// before, and that call reports the original refusal. `SystemAllocation`
    /// already prepares and resolves in the World's placement before any
    /// handler, so it has no step to add.
    ///
    /// The step reads clones of the one-shot preconditions and source
    /// expectation (the same evidence, not a new observation), so the request
    /// stays unstaged for the funded call.
    pub(in crate::application_entry) fn recorded_replay<Refusal>(
        &self,
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
        prepare: impl FnOnce(
            &AdvancementPhase<'_>,
            &Self,
            &ApplicationMutationIdentities<'_, Schema, Intent::Binding>,
            WorthQueryStagedMutation<Schema, Intent>,
        ) -> Result<PreparedMutation<Schema, Intent::Binding>, Refusal>,
    ) -> Option<Outcome<Schema, Intent::Binding>> {
        if matches!(
            allocation_policy,
            ExecutionAllocationPolicy::SystemAllocation
        ) || self.unstaged.is_none()
        {
            return None;
        }
        self.request
            .application
            .with_application_advancement(self.request_scope(), |phase| {
                self.require_workflow_transition().ok()?;
                let identities = self.identities().ok()?;
                let staged = WorthQueryStagedMutation {
                    preconditions: self.request.preconditions.clone(),
                    source: self.request.source.clone(),
                };
                let prepared = prepare(&phase, self, &identities, staged).ok()?;
                let binding = WorthQueryMutationCommitBinding::new(&identities, prepared.extension);
                self.resolve_idempotency(&phase, &prepared.admission, binding.idempotency())
                    .ok()
                    .flatten()
            })
            .ok()
            .flatten()
    }
}
