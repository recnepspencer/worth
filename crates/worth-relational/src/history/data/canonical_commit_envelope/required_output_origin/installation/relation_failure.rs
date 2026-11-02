//! Preserve request stops and contextualize relation custody refusals.
use super::*;

pub(in super::super) fn relation_stop(
    stop: crate::history::data::CodecRetirementClosureStop,
) -> TransactionCommitError {
    match stop {
        crate::history::data::CodecRetirementClosureStop::Request(cause) => origin_request(cause),
        crate::history::data::CodecRetirementClosureStop::Payload(detail) => {
            crate::mvcc::origin_denial(detail)
        }
    }
}

pub(super) fn relation_context(
    stop: crate::history::data::CodecRetirementClosureStop,
    stage: &str,
    basis: &AdmittedRelationalBranchBasis,
    candidate: &CanonicalCommitEnvelope,
) -> TransactionCommitError {
    match stop {
        crate::history::data::CodecRetirementClosureStop::Request(cause) => origin_request(cause),
        crate::history::data::CodecRetirementClosureStop::Payload(detail) => {
            crate::mvcc::origin_denial(format!(
                "{detail}; publication_stage={stage}; before={:?}; candidate={:?}",
                basis.descriptor(),
                candidate.commit
            ))
        }
    }
}
