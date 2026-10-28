use worth_execution::ExecutionRequest;

use super::frame::BranchTransitionArtifactHeader;
use super::payload_wire::{TransitionPayloadRef, TransitionPayloadWire};
use super::{
    RelationalBranchTransitionArtifact, RelationalBranchTransitionArtifactRefusal as Refusal,
};
use crate::branch::{PositionedBranchTransition, RelationalBranchTransition};
use crate::durability::artifact_canonical_payload::is_canonical_payload;

impl RelationalBranchTransitionArtifact {
    /// The only door from artifact bytes back to a positioned transition.
    ///
    /// Phases, cheapest first, each refusing before the next runs: request
    /// liveness; frame, version, length and digest; payload decode; header
    /// agreement with the payload; a sequence of at least one; the canonical
    /// runtime frame of a fork's source head; and finally a canonical-form
    /// check that the payload is exactly the transition's encoding.
    ///
    /// Readmission checks the artifact alone. Whether the transition applies
    /// to a branch is decided when recovery replays it.
    ///
    /// Cost: O(payload bytes) for the digest, decode and canonical check.
    pub fn readmit(
        bytes: &[u8],
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<PositionedBranchTransition, Refusal> {
        crate::execution::check_request(execution).map_err(Refusal::Request)?;
        let (header, payload) = BranchTransitionArtifactHeader::verify(bytes)?;
        let wire: TransitionPayloadWire =
            rmp_serde::from_slice(payload).map_err(|error| Refusal::PayloadDecode {
                detail: error.to_string(),
            })?;
        check_header_agreement(&header, &wire)?;
        if wire.sequence.0 == 0 {
            return Err(Refusal::SequenceZero);
        }
        check_canonical_frame(&wire.transition)?;
        let positioned =
            PositionedBranchTransition::admit(wire.sequence, wire.follows, wire.transition);
        if !is_canonical_payload(&TransitionPayloadRef::new(&positioned), payload) {
            return Err(Refusal::NonCanonicalPayload);
        }
        Ok(positioned)
    }
}

fn check_header_agreement(
    header: &BranchTransitionArtifactHeader,
    wire: &TransitionPayloadWire,
) -> Result<(), Refusal> {
    if header.sequence != wire.sequence {
        return Err(Refusal::SequenceDisagreement {
            header: header.sequence,
            payload: wire.sequence,
        });
    }
    if header.follows != wire.follows {
        return Err(Refusal::FollowsDisagreement {
            header: header.follows,
            payload: wire.follows,
        });
    }
    Ok(())
}

/// A fork records its source head in the canonical runtime frame; a head
/// bound to any live runtime is not durable history.
fn check_canonical_frame(transition: &RelationalBranchTransition) -> Result<(), Refusal> {
    match transition {
        RelationalBranchTransition::Fork { source_head, .. }
            if source_head.runtime_instance_id()
                != crate::branch::CANONICAL_RUNTIME_INSTANCE_ID =>
        {
            Err(Refusal::SourceHeadOutsideCanonicalFrame {
                runtime_instance_id: source_head.runtime_instance_id(),
            })
        }
        _ => Ok(()),
    }
}
