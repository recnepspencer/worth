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
    performed::WorthQueryPerformedMutationExecutionDenial as PerformedDenial,
    recovery::request_stop, staged::WorthQueryStagedMutation, WorthQueryApplicationMutationOutcome,
    WorthQueryApplicationMutationRequestWithIdempotency,
};
use crate::application_entry::WorthQueryApplicationRequestMutationDenial as RequestDenial;

type Outcome<Schema, Binding> = WorthQueryApplicationMutationOutcome<
    <Binding as ApplicationMutationBinding<Schema>>::Denial,
    <Binding as ApplicationMutationBinding<Schema>>::Result,
>;

/// What the pre-admission step learned before any funded opening.
pub(in crate::application_entry) enum RecordedReplay<Outcome, Refusal> {
    /// The key is recorded: its outcome answers the retry.
    Replayed(Outcome),
    /// The key is fresh, or the World-funded step stopped before it could
    /// say: the funded call answers under the caller's lease.
    Miss,
    /// The request's own refusal. Funding cannot change it, so it answers
    /// the call and `prepare` does not run again.
    Refused(Refusal),
}

/// A lane refusal the step can classify. An execution stop (opening,
/// resource, cancellation, deadline, panic) belongs to the World-funded step,
/// not to the request: the caller's lease funds the request, so the funded
/// call answers it. Every other refusal is the request's.
pub(in crate::application_entry) trait StepRefusal:
    From<RequestDenial>
{
    fn execution_stop(&self) -> bool;
}
impl StepRefusal for RequestDenial {
    fn execution_stop(&self) -> bool {
        request_stop(self).is_some()
    }
}
impl From<RequestDenial> for PerformedDenial {
    fn from(denial: RequestDenial) -> Self {
        Self::Mutation(denial)
    }
}
impl StepRefusal for PerformedDenial {
    fn execution_stop(&self) -> bool {
        match self {
            Self::Mutation(denial) => denial.execution_stop(),
            Self::Action(_)
            | Self::ForeignProgram
            | Self::UndeclaredOutputRoot
            | Self::MissingConnection
            | Self::Connection(_) => false,
        }
    }
}

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
    /// opens, charges or needs the caller's lease. The request's own refusal
    /// returns here too, as `Refused`, whatever the caller's lease could do;
    /// only a miss or an execution stop of this World-funded step opens the
    /// funded call ([`StepRefusal`]). `SystemAllocation` already prepares and
    /// resolves in the World's placement before any handler, so it has no
    /// step to add.
    ///
    /// The step reads clones of the one-shot preconditions and source
    /// expectation (the same evidence, not a new observation), so the request
    /// stays unstaged for the funded call.
    pub(in crate::application_entry) fn recorded_replay<Refusal: StepRefusal>(
        &self,
        allocation_policy: ExecutionAllocationPolicy<'_, '_>,
        prepare: impl FnOnce(
            &AdvancementPhase<'_>,
            &Self,
            &ApplicationMutationIdentities<'_, Schema, Intent::Binding>,
            WorthQueryStagedMutation<Schema, Intent>,
        ) -> Result<PreparedMutation<Schema, Intent::Binding>, Refusal>,
    ) -> RecordedReplay<Outcome<Schema, Intent::Binding>, Refusal> {
        if matches!(
            allocation_policy,
            ExecutionAllocationPolicy::SystemAllocation
        ) || self.unstaged.is_none()
        {
            return RecordedReplay::Miss;
        }
        let step = self
            .request
            .application
            .with_application_advancement(self.request_scope(), |phase| -> Result<_, Refusal> {
                self.require_workflow_transition()?;
                let identities = self.identities()?;
                let staged = WorthQueryStagedMutation {
                    preconditions: self.request.preconditions.clone(),
                    source: self.request.source.clone(),
                };
                let prepared = prepare(&phase, self, &identities, staged)?;
                let binding = WorthQueryMutationCommitBinding::new(&identities, prepared.extension);
                Ok(self.resolve_idempotency(&phase, &prepared.admission, binding.idempotency())?)
            })
            .map_err(|cause| Refusal::from(RequestDenial::ExecutionRequest(cause)))
            .and_then(|step| step);
        match step {
            Ok(Some(outcome)) => RecordedReplay::Replayed(outcome),
            Ok(None) => RecordedReplay::Miss,
            Err(refusal) if refusal.execution_stop() => RecordedReplay::Miss,
            Err(refusal) => RecordedReplay::Refused(refusal),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{PerformedDenial, RequestDenial, StepRefusal};
    use worth_query_execution::facade::application_contribution::{
        WorthQueryAdvancementDenial as Advancement,
        WorthQueryManagedComputationInterruption as Interruption,
    };

    #[test]
    fn only_the_steps_execution_stop_leaves_the_answer_to_the_funded_call() {
        let stop =
            || RequestDenial::ExecutionRequest(Advancement::Interrupted(Interruption::Cancelled));
        assert!(stop().execution_stop());
        assert!(PerformedDenial::from(stop()).execution_stop());
        for refusal in [
            RequestDenial::ApplicationProgramRequired,
            RequestDenial::RequiresWorkflowTransition,
            RequestDenial::PreparationSpent,
        ] {
            assert!(!refusal.execution_stop(), "{refusal:?}");
            let performed = PerformedDenial::from(refusal);
            assert!(!performed.execution_stop(), "{performed:?}");
        }
        assert!(!PerformedDenial::MissingConnection.execution_stop());
        assert!(!PerformedDenial::ForeignProgram.execution_stop());
    }
}
