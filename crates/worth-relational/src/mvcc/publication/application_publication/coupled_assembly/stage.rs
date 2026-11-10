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
    /// candidate pin. Stages apply once each, in the admitted roster order, and
    /// before any producer part: a stage after a producer wave would need the
    /// envelope-wide final qualification, which this entry does not provide.
    pub fn append_authored_stage(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        source: crate::identity::data::EntityId,
        content: &crate::content::RelationalContentAttachments,
        batches: Vec<WorkerIntentBatch>,
    ) -> Result<(), RelationalCoupledAssemblyFailure> {
        if self.next_authored_stage() != Some(source) || !self.parts.is_empty() {
            return Err(part::reject(Cause::UnadmittedAuthoredStage, source).into());
        }
        let applied = self.apply_suffix(
            execution,
            part::SuffixWriter::Authored { source },
            content,
            batches,
        )?;
        self.stages.push(CoupledAuthoredStagePart {
            source,
            input: applied.read,
            commands: applied.plan,
            records: applied.records,
        });
        self.authored_fragments = self
            .mutation
            .outcome
            .effect
            .publication
            .patch_fragments
            .len();
        let latest = self.stages.last().expect("the actual stage just appended");
        self.advance_pin(
            execution,
            &applied.delta_state,
            &latest.records,
            &latest.input,
        )
    }
}
