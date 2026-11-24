//! The accepted independent merge proof supplied to publication preparation.
use super::authority_context::PreparedAuthorityScope;
use worth_relational_mutation::internal_port::validation::engine::result::InvariantExecutionResult;

pub struct AcceptedMergeDraft {
    pub scope: PreparedAuthorityScope,
    pub invariants: InvariantExecutionResult,
}
