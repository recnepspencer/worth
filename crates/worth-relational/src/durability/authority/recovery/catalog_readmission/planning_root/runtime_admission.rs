use worth_relational_durability::internal_port::durability::authority::recovery::catalog_readmission::planning_root::RelationalHomeProgramPlanningRootOwnerPort as _;
use super::super::refusal::RelationalHomeProgramCatalogReadmissionDenial as Denial;
use super::RelationalHomeProgramPlanningRoot;
use crate::durability::authority::recovered_runtime::RecoveredRelationalRuntimeAuthority;
use crate::runtime::RelationalRuntime;
use worth_execution::ExecutionRequest;

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
        Ok(RelationalHomeProgramPlanningRoot::from_readmitted_root(
            self.runtime_instance_id,
            planning.source.clone(),
            basis,
        ))
    }
}
