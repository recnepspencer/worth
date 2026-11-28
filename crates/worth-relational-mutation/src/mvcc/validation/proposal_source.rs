//! Exactly one source of proposed meaning enters the ordinary judging round.
use crate::authority::commit::semantic_plan::retained_entity_payload::PreparedRetainedEntityPayload;

pub enum RelationalProposalSource {
    Transaction,
    RetainedEntitySelection(PreparedRetainedEntityPayload),
}
