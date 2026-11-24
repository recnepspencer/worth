//! A pending transition's planning root is readmitted once per resumed plan.
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis;
use worth_relational_branch::internal_port::branch::recorded_basis::RecordedRelationalComponentReference;

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

#[doc(hidden)]
pub trait RelationalHomeProgramPlanningRootOwnerPort: Sized {
    fn from_readmitted_root(
        runtime: u64,
        source: RecordedRelationalComponentReference,
        basis: AdmittedRelationalBranchBasis,
    ) -> Self;
    fn is_from_runtime(&self, runtime: u64) -> bool;
    fn source(&self) -> &RecordedRelationalComponentReference;
}

impl RelationalHomeProgramPlanningRootOwnerPort for RelationalHomeProgramPlanningRoot {
    fn from_readmitted_root(
        runtime: u64,
        source: RecordedRelationalComponentReference,
        basis: AdmittedRelationalBranchBasis,
    ) -> Self {
        Self {
            runtime,
            source,
            basis,
        }
    }
    fn is_from_runtime(&self, runtime: u64) -> bool {
        self.runtime == runtime
    }
    fn source(&self) -> &RecordedRelationalComponentReference {
        &self.source
    }
}
