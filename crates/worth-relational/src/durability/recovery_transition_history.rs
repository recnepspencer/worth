//! Authenticated transition journal gap followed by the selected retained tail.
use crate::branch::transition::{PositionedBranchTransition, RelationalBranchTransitionSequence};
use crate::durability::branch_transition_artifact::RelationalBranchTransitionArtifact;
use crate::durability::data::{DurabilityError, RecoveryFailureClass, RelationalRecoveryStop};
use crate::durability::published_medium::RelationalTransitionHistoryReader;
use std::sync::Arc;

pub(crate) fn reconstruct_transition_history(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    after: RelationalBranchTransitionSequence,
    covered: RelationalBranchTransitionSequence,
    tail: Vec<Arc<PositionedBranchTransition>>,
    medium: Option<RelationalTransitionHistoryReader>,
) -> Result<Vec<PositionedBranchTransition>, RelationalRecoveryStop> {
    let mut transitions = Vec::new();
    if covered > after {
        let medium = medium.ok_or_else(|| {
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
                    crate::durability::medium::RelationalDurableTransitionReadFailure::Frame(_) => {
                        RecoveryFailureClass::CorruptBranchTransitionArtifact
                    }
                    _ => RecoveryFailureClass::DurableIoFailure,
                };
                DurabilityError::new(
                    class,
                    format!("cannot read branch transition {number}: {failure:?}"),
                )
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
                        crate::durability::medium_transition_tail::artifact_refusal_stop(
                            sequence, failure,
                        )
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
