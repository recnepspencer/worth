//! Preserve request stops and contextualize relation custody refusals.
use super::*;

pub fn relation_stop(
    stop: crate::history::data::CodecRetirementClosureStop,
) -> TransactionCommitError {
    match stop {
        crate::history::data::CodecRetirementClosureStop::Request(cause) => origin_request(cause),
        crate::history::data::CodecRetirementClosureStop::Payload(detail) => {
            crate::transactions::data::outcomes::transaction_commit_error::origin_denial(detail)
        }
    }
}

pub fn relation_context(
    stop: crate::history::data::CodecRetirementClosureStop,
    stage: &str,
    before: &RelationalBranchBasisDescriptor,
    candidate: &CanonicalCommitEnvelopeBody,
) -> TransactionCommitError {
    match stop {
        crate::history::data::CodecRetirementClosureStop::Request(cause) => origin_request(cause),
        crate::history::data::CodecRetirementClosureStop::Payload(detail) => {
            crate::transactions::data::outcomes::transaction_commit_error::origin_denial(format!(
                "{detail}; publication_stage={stage}; before={:?}; candidate={:?}",
                before, candidate.commit
            ))
        }
    }
}
