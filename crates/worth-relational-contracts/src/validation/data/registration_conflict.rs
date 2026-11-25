use crate::validation::data::execution::InvariantExecutionPoint;
use crate::validation::data::rule_id::CustomInvariantSemanticIdentity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DuplicateCustomInvariantRegistration {
    pub identity: CustomInvariantSemanticIdentity,
    pub execution_point: InvariantExecutionPoint,
}
