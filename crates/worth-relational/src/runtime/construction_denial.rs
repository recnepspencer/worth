//! Configuration refusals precede issuance of a live runtime identity.
use crate::validation::DuplicateCustomInvariantRegistration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalRuntimeConstructionDenial {
    Execution(crate::execution::RelationalExecutionDenialCause),
    InvariantExecutionPoint {
        registration: usize,
        execution_point: crate::validation::data::InvariantExecutionPoint,
    },
    CustomInvariant(DuplicateCustomInvariantRegistration),
    ManagedIndexSource(crate::derived_relation::DerivedRelationDenial),
    MainBranch(crate::branch::RelationalBranchObservationConstructionDenial),
    SymbolTable(&'static str),
}

impl std::fmt::Display for RelationalRuntimeConstructionDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "runtime construction refused: {self:?}")
    }
}
impl std::error::Error for RelationalRuntimeConstructionDenial {}
