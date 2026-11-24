//! Proposal-owned projection of exact applied commands.
use super::*;
use crate::mvcc::validation::validated_proposal::ValidatedRelationalProposal;

impl ValidatedRelationalProposal {
    /// Application-authored touches after checking the sealed Native restoration inverse.
    /// Complete mutation inspection remains available through `mutation_touches`.
    pub fn application_mutation_touches(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<ValidatedMutationTouches<'_>, ValidatedMutationTouchProjectionError> {
        preparation::project(self, execution, true)
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
                Some(group.authored_commands.plan().merged_intents.as_slice())
            }
            _ => None,
        };
        preparation::project_commands(self, execution, true, commands)
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
                group.stages.get(stage)?
            }
            _ => return None,
        };
        Some(preparation::project_commands(
            self,
            execution,
            true,
            Some(&part.commands.plan().merged_intents),
        ))
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
            .flat_map(|group| group.parts.iter())
            .rfind(|part| part.publication.producing_meaning() == Some(meaning))?;
        Some(preparation::project_commands(
            self,
            execution,
            false,
            Some(&part.commands.plan().merged_intents),
        ))
    }

    /// O(reached intents + prior/supplied fields), plus canonical sorting.
    /// Native retained before/after state supplies presence; no history is read.
    pub fn mutation_touches(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<ValidatedMutationTouches<'_>, ValidatedMutationTouchProjectionError> {
        preparation::project(self, execution, false)
    }
}
