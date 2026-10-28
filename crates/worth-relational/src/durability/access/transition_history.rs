use crate::branch::{PositionedBranchTransition, RelationalBranchTransitionSequence};
use crate::durability::branch_transition_artifact::RelationalBranchTransitionArtifact;
use crate::durability::data::{DurabilityError, RecoveryFailureClass, RelationalRecoveryStop};

impl super::DurabilityAccess<'_> {
    /// Reconstruct transitions strictly after the selected basis. Read only
    /// the gap between that basis and the retained live tail from the medium.
    /// Cost: O(missing gap + retained tail), never O(covered history).
    pub(crate) fn reconstruct_branch_transition_history(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        after: RelationalBranchTransitionSequence,
    ) -> Result<Vec<PositionedBranchTransition>, RelationalRecoveryStop> {
        let (covered, tail) = self.runtime.history.retained_branch_transitions();
        let mut transitions = Vec::new();
        if covered > after {
            let medium = self
                .runtime
                .durability
                .transition_history()
                .ok_or_else(|| {
                    DurabilityError::new(
                        RecoveryFailureClass::DurableIoFailure,
                        "covered transition history requires its durable medium",
                    )
                })?;
            for number in after.0 + 1..=covered.0 {
                execution
                    .consult()
                    .map_err(|stop| RelationalRecoveryStop::Request(stop.into()))?;
                let sequence = RelationalBranchTransitionSequence(number);
                let artifact = medium.read_transition(sequence).map_err(|failure| {
                    let class = match failure {
                        crate::durability::medium::RelationalDurableTransitionReadFailure::Frame(_) =>
                            RecoveryFailureClass::CorruptBranchTransitionArtifact,
                        _ => RecoveryFailureClass::DurableIoFailure,
                    };
                    DurabilityError::new(class,
                        format!("cannot read branch transition {number}: {failure:?}"))
                })?;
                if artifact.sequence() != sequence {
                    return Err(DurabilityError::new(
                        RecoveryFailureClass::CorruptBranchTransitionArtifact,
                        format!("journal returned the wrong transition for sequence {number}"),
                    )
                    .into());
                }
                transitions.push(
                    RelationalBranchTransitionArtifact::readmit(artifact.as_bytes(), execution)
                        .map_err(|failure| {
                            super::medium_transition_tail::artifact_refusal_stop(sequence, failure)
                        })?,
                );
            }
        }
        transitions.extend(
            tail.into_iter()
                .filter(|transition| transition.sequence() > after)
                .map(|transition| transition.as_ref().clone()),
        );
        Ok(transitions)
    }
}
