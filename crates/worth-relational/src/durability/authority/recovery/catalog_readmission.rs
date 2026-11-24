//! Retain actual original roots admitted for a recovered Home program.
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::durability::authority::RelationalRecoveredPublicationCustody;
use std::sync::Mutex;
use worth_execution::ExecutionArray;

mod planning_root;
pub(crate) mod refusal;
mod runtime_admission;
pub use planning_root::RelationalHomeProgramPlanningRoot;

/// Actual original bases and pins, not a fabricated publication or live service.
#[derive(Debug)]
pub struct ReadmittedRelationalHomeProgramCatalogs {
    predecessor: AdmittedRelationalBranchBasis,
    current: AdmittedRelationalBranchBasis,
    prefix: ExecutionArray<Mutex<Option<RelationalRecoveredPublicationCustody>>>,
}
impl ReadmittedRelationalHomeProgramCatalogs {
    pub fn predecessor(&self) -> &AdmittedRelationalBranchBasis {
        &self.predecessor
    }
    pub fn current(&self) -> &AdmittedRelationalBranchBasis {
        &self.current
    }
    pub fn durable_native_steps(&self) -> usize {
        self.prefix.len()
    }
}
