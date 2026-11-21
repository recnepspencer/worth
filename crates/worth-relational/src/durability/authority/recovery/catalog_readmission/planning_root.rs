//! A pending transition's planning root is readmitted once per resumed plan.
use super::{Denial, RecoveredRelationalRuntimeAuthority};
use crate::branch::{AdmittedRelationalBranchBasis, RecordedRelationalComponentReference};
use crate::runtime::RelationalRuntime;
use worth_execution::ExecutionRequest;

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

impl RecoveredRelationalRuntimeAuthority {
    /// The root is the predecessor the transition-source recovery admitted.
    /// Caller DATA cannot choose it.
    pub fn readmit_home_program_planning_root(
        &self,
        request: &ExecutionRequest<'_, '_>,
        runtime: &RelationalRuntime,
    ) -> Result<RelationalHomeProgramPlanningRoot, Denial> {
        request.consult().map_err(Denial::Request)?;
        let planning = self
            .home_program_predecessor
            .as_ref()
            .ok_or(Denial::PredecessorRequired)?;
        let basis = self
            .admit_recorded_component(request, runtime, &planning.source)
            .map_err(Denial::Component)?
            .into_basis();
        Ok(RelationalHomeProgramPlanningRoot {
            runtime: self.runtime_instance_id,
            source: planning.source.clone(),
            basis,
        })
    }
}
