//! A pending transition's planning root is readmitted once per resumed plan.
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::branch::recorded_basis::RecordedRelationalComponentReference;

/// The recorded planning predecessor that this transition-source recovery
/// admitted, readmitted from its exact recovered image. A resumed successor
/// compiles against `basis()`, and catalog readmission takes this same root
/// instead of readmitting it again, so the two cannot diverge.
#[derive(Debug)]
pub struct RelationalHomeProgramPlanningRoot {
    pub(super) runtime: u64,
    pub(super) source: RecordedRelationalComponentReference,
    pub(super) basis: AdmittedRelationalBranchBasis,
}
impl RelationalHomeProgramPlanningRoot {
    pub fn basis(&self) -> &AdmittedRelationalBranchBasis {
        &self.basis
    }
}

mod runtime_admission;
