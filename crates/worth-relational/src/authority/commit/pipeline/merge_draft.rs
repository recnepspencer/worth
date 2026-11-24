//! The accepted independent merge proof supplied to publication preparation.
use super::authority_context::PreparedAuthorityScope;
use crate::validation::engine::InvariantExecutionResult;

pub(crate) struct AcceptedMergeDraft {
    pub(super) scope: PreparedAuthorityScope,
    pub(super) invariants: InvariantExecutionResult,
}
