//! Corrupt only a real Query-prepared participant at this Native receiver.

use super::*;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasisFieldsPort as _;
impl RelationalAuthoredCoupledCandidate<'_> {
    /// Closed corruption, never an alternate assembly door. Every argument came
    /// through the real installed writer; Native's ordinary admission still runs.
    #[doc(hidden)]
    pub fn append_corrupt_participant_for_test(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        posture: RelationalCoupledProducerPosture,
        content: &crate::content::RelationalContentAttachments,
        mut batches: Vec<crate::transactions::data::primitives::WorkerIntentBatch>,
        fault: u8,
    ) -> Result<(), RelationalCoupledAssemblyFailure> {
        if fault == 4 {
            self.append_publication(execution, posture, content, batches)?;
            return self
                .replay_continuation_probe(execution)
                .map_err(Into::into);
        }
        if matches!(fault, 7..=9) {
            return self.continued_round_probe(execution, posture, content, batches, fault);
        }
        if matches!(fault, 5 | 6) {
            corruption_commands::delete_entity(
                &posture,
                &self.parts,
                &self.mutation.outcome.created_entities,
                self.authored_source(),
                &mut batches,
                fault == 6,
            )?;
            return self.append_publication(execution, posture, content, batches);
        }
        let posture = corruption_commands::corrupt_predecessor_for_test(
            execution,
            &self.runtime.history,
            || {
                self.binding
                    .owners
                    .before()
                    .owner_inner()
                    .root
                    .schema_authority()
                    .registry()
            },
            posture,
            &mut batches,
            fault,
        )?;
        self.append_publication(execution, posture, content, batches)
    }
}
