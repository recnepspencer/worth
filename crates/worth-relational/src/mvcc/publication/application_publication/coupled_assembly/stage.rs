//! Later authored stages apply over the candidate under the authored writer.
use super::*;
use crate::publication::data::AuthoredRetirementRefusal as Cause;
use crate::transactions::data::WorkerIntentBatch;

impl RelationalAuthoredCoupledCandidate<'_> {
    /// The source Native admitted for the next authored stage, if one remains.
    pub fn next_authored_stage(&self) -> Option<crate::identity::data::EntityId> {
        self.binding.owners.stages.get(self.stages.len()).copied()
    }

    /// Apply the next admitted authored stage's own commands over the current
    /// candidate pin. Stages apply once each, in the admitted authored roster
    /// order, including after producer parts. The complete envelope retains
    /// final decision-support qualification at its Query owner.
    pub fn append_authored_stage(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        source: crate::identity::data::EntityId,
        content: &crate::content::RelationalContentAttachments,
        batches: Vec<WorkerIntentBatch>,
    ) -> Result<(), RelationalCoupledAssemblyFailure> {
        self.append_authored_stage_outputs(execution, source, content, batches, Vec::new())
    }

    /// Plain authored correspondence is resolved from the same actual suffix
    /// allocation as its commands. It grants no producer or lifetime custody.
    pub fn append_authored_stage_outputs(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        source: crate::identity::data::EntityId,
        content: &crate::content::RelationalContentAttachments,
        batches: Vec<WorkerIntentBatch>,
        mut roles: Vec<RequiredOutputRoleTarget>,
    ) -> Result<(), RelationalCoupledAssemblyFailure> {
        if self.next_authored_stage() != Some(source) {
            return Err(part::reject(Cause::UnadmittedAuthoredStage, source).into());
        }
        let fragment_start = self
            .mutation
            .outcome
            .effect
            .publication
            .patch_fragments
            .len();
        let applied = self.apply_suffix(
            execution,
            part::SuffixWriter::Authored {
                source,
                roles: &mut roles,
            },
            content,
            batches,
        )?;
        self.authored_writes.retain(
            execution,
            self.stages.len() + 1,
            applied.plan.plan(),
            &applied.records,
        )?;
        let output_ids = ordinary_roles::resolve_applied_roles(
            execution,
            &roles,
            &self.mutation.outcome.created_entities,
        )?;
        self.stages.push(CoupledAuthoredStagePart {
            fragments: fragment_start
                ..self
                    .mutation
                    .outcome
                    .effect
                    .publication
                    .patch_fragments
                    .len(),
            roles,
            output_ids,
            source,
            input: applied.read,
            commands: applied.plan,
            records: applied.records,
        });
        let latest = self.stages.last().expect("the actual stage just appended");
        self.advance_pin(
            execution,
            &applied.delta_state,
            &latest.records,
            &latest.input,
        )
    }
}
