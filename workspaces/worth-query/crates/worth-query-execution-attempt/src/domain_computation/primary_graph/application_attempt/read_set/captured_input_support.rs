//! An unconsumed input prelude compares every completed fact on its actual proposal.
use super::{
    denial, WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryCompleteApplicationReadSet, WorthQueryProjectedApplicationMutation,
};
use worth_relational::facade::runtime::{
    RelationalAppliedCandidateReadIdentity, RelationalPinnedRead,
};

impl<Schema, Operation, Input, Scope>
    WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        WorthQueryProjectedApplicationMutation,
    >
{
    /// Equality neither moves the original admission nor issues a write target.
    /// Only the occurrence owner may supersede an unconsumed capture on false.
    pub fn captured_inputs_remain_equal(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        current: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    ) -> Result<bool, WorthQueryApplicationAttemptDenial> {
        self.admission
            .validate_current_authority()
            .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)?;
        let foreign = || {
            denial(
                WorthQueryApplicationAttemptDenialKind::ProjectionAdmissionMismatch,
                self.admission.operation(),
            )
        };
        let captured = self.lease.applied_projection().ok_or_else(foreign)?;
        if captured.candidate_identity().proposal() != current.candidate_identity().proposal() {
            return Err(foreign());
        }
        self.lease.handle().with_runtime(execution, |runtime| {
            for fact in self.facts.iter() {
                self.admission
                    .validate_current_authority()
                    .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)?;
                if !fact.support_equal_after_own_effect(
                    execution,
                    runtime,
                    self.lease.snapshot(),
                    captured,
                    current,
                )? {
                    return Ok(false);
                }
            }
            Ok(true)
        })
    }
}
