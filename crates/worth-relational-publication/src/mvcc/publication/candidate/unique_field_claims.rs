//! Transfer validated key claims into the original candidate's owning payload.
use super::PreparedRelationalCommitCandidate;
use worth_relational_mutation::internal_port::validation::unique_field_probes::UniqueFieldProbeBatch;

pub fn attach(
    candidate: PreparedRelationalCommitCandidate,
    claims: UniqueFieldProbeBatch,
) -> PreparedRelationalCommitCandidate {
    candidate
        ._payload
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .as_mut()
        .expect("the newly prepared candidate holds its payload")
        .unique_fields = claims;
    candidate
}
