//! Proposal-owned projection of exact applied commands.
use worth_relational_publication::internal_port::mvcc::publication::application_publication::coupled_assembly::completion::RelationalCoupledAuthoredPublicationFieldsPort as _;
use super::*;
use crate::authority::commit::pipeline::{consult_stage, CommitStage};
use crate::mvcc::validation::validated_proposal::ValidatedRelationalProposal;
use crate::transactions::data::intents::mutation_intent::MutationIntent;

impl ValidatedRelationalProposal {
    /// Application-authored touches after checking the sealed Native restoration inverse.
    /// Complete mutation inspection remains available through `mutation_touches`.
    pub fn application_mutation_touches(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<ValidatedMutationTouches<'_>, ValidatedMutationTouchProjectionError> {
        self.project_touches(execution, true, None)
    }

    /// Actor-only authoring in an admitted coupled group. The complete proposal
    /// remains available through `mutation_touches` for Native inspection.
    pub fn authored_mutation_touches(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<ValidatedMutationTouches<'_>, ValidatedMutationTouchProjectionError> {
        let commands = match self
            .state
            .required_output_publication
            .as_ref()
            .map(|p| &p.publication)
        {
            Some(crate::mvcc::RequiredOutputPublication::CoupledAuthored { group }) => {
                Some(group.authored_commands().plan().merged_intents.as_slice())
            }
            _ => None,
        };
        self.project_touches(execution, true, commands)
    }

    /// Actor-only authoring of one later authored stage (`stage` 0 is the
    /// first stage after the prefix), from that stage's own applied commands.
    /// Each stage's invariants are judged on its own contribution, as each
    /// producer's are; `None` when the group has no such stage.
    pub fn authored_stage_mutation_touches(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        stage: usize,
    ) -> Option<Result<ValidatedMutationTouches<'_>, ValidatedMutationTouchProjectionError>> {
        let part = match self
            .state
            .required_output_publication
            .as_ref()
            .map(|p| &p.publication)
        {
            Some(crate::mvcc::RequiredOutputPublication::CoupledAuthored { group }) => {
                group.stages().get(stage)?
            }
            _ => return None,
        };
        Some(self.project_touches(execution, true, Some(&part.commands.plan().merged_intents)))
    }

    /// One concrete producer's touches use its own applied command contribution,
    /// under the final proposal's single invariant judgment and request budgets.
    /// The participant is the part whose producing meaning equals `meaning`
    /// exactly, implementation and epoch included; only that part is projected.
    /// O(parts) meaning comparisons, then O(that part's intents).
    pub fn coupled_producer_mutation_touches<'a>(
        &'a self,
        execution: &ExecutionRequest<'_, '_>,
        meaning: &crate::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaning,
    ) -> Option<Result<ValidatedMutationTouches<'a>, ValidatedMutationTouchProjectionError>> {
        let part = self
            .state
            .required_output_publication
            .iter()
            .filter_map(|p| match &p.publication {
                crate::mvcc::RequiredOutputPublication::CoupledAuthored { group } => Some(group),
                _ => None,
            })
            .flat_map(|group| group.parts().iter())
            .rfind(|part| part.publication.producing_meaning() == Some(meaning))?;
        Some(self.project_touches(execution, false, Some(&part.commands.plan().merged_intents)))
    }

    /// O(reached intents + prior/supplied fields), plus canonical sorting.
    /// Native retained before/after state supplies presence; no history is read.
    pub fn mutation_touches(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<ValidatedMutationTouches<'_>, ValidatedMutationTouchProjectionError> {
        self.project_touches(execution, false, None)
    }

    /// Sums the validated intents every touch projection on this thread examines
    /// while `observe` runs. A cost court compares two sizes; nothing is decided here.
    pub fn observe_touch_projection_work<R>(observe: impl FnOnce() -> R) -> (R, u64) {
        super::work_observation::observe(observe)
    }

    fn project_touches<'p>(
        &'p self,
        execution: &ExecutionRequest<'_, '_>,
        application_only: bool,
        commands: Option<&'p [MutationIntent]>,
    ) -> Result<ValidatedMutationTouches<'p>, ValidatedMutationTouchProjectionError> {
        let consult = || {
            consult_stage(execution, CommitStage::Validation)
                .map_err(|stop| collection::execution(stop.into()))
        };
        let inverse_create =
            |intent: &MutationIntent,
             checkpoint: &mut dyn FnMut(
                u64,
            )
                -> Result<(), ValidatedMutationTouchProjectionError>| {
                authored_restoration::owns_create(self, intent, checkpoint)
            };
        preparation::project(
            preparation::ProjectionInputs {
                plan: self.state.prepared.plan.view(),
                before: &self.state.prepared.working_state,
                after: self.state.proposed_mutation.state(),
                schema: &self.state.prepared.schema_authority,
                deltas: &self
                    .state
                    .proposed_mutation
                    .outcome
                    .effect
                    .publication
                    .canonical_deltas,
                work_budget: self.state.commit_work_budget.as_ref(),
                consult: &consult,
                inverse_create: application_only.then_some(&inverse_create as &_),
            },
            execution,
            commands,
            || {
                if application_only {
                    authored_restoration::validate(self, execution)?;
                }
                Ok(())
            },
        )
    }
}
