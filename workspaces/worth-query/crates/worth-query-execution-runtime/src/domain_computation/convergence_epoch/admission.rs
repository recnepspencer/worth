use worth_query_admission::facade::domain_computation::WorthQueryAdmittedConvergenceContract;
use worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority;

use super::{
    WorthQueryAdmittedDirectConvergenceEpoch, WorthQueryAdmittedWorkflowConvergenceEpoch,
    WorthQueryConvergenceEpochCounters, WorthQueryConvergenceEpochDenial,
    WorthQueryConvergenceEpochDenialKind as Kind,
};
use crate::domain_computation::{
    WorthQueryAdmittedDirectRun, WorthQueryAdmittedWorkflowRun,
    WorthQueryExecutionBoundOperationAuthority, WorthQueryExecutionRuntime,
};

pub struct WorthQueryDirectConvergenceAdmissionRejection {
    denial: WorthQueryConvergenceEpochDenial,
    contract: WorthQueryAdmittedConvergenceContract,
    managed_run: WorthQueryAdmittedDirectRun,
    graph: WorthQueryInstalledGraphParticipationAuthority,
}

impl WorthQueryDirectConvergenceAdmissionRejection {
    pub fn denial(&self) -> &WorthQueryConvergenceEpochDenial {
        &self.denial
    }

    pub fn into_parts(
        self,
    ) -> (
        WorthQueryAdmittedConvergenceContract,
        WorthQueryAdmittedDirectRun,
        WorthQueryInstalledGraphParticipationAuthority,
    ) {
        (self.contract, self.managed_run, self.graph)
    }
}

pub struct WorthQueryWorkflowConvergenceAdmissionRejection {
    denial: WorthQueryConvergenceEpochDenial,
    contract: WorthQueryAdmittedConvergenceContract,
    managed_run: WorthQueryAdmittedWorkflowRun,
    graph: WorthQueryInstalledGraphParticipationAuthority,
}

impl WorthQueryWorkflowConvergenceAdmissionRejection {
    pub fn denial(&self) -> &WorthQueryConvergenceEpochDenial {
        &self.denial
    }

    pub fn into_parts(
        self,
    ) -> (
        WorthQueryAdmittedConvergenceContract,
        WorthQueryAdmittedWorkflowRun,
        WorthQueryInstalledGraphParticipationAuthority,
    ) {
        (self.contract, self.managed_run, self.graph)
    }
}

impl WorthQueryExecutionRuntime {
    pub fn admit_direct_convergence_epoch(
        &self,
        operation: &WorthQueryExecutionBoundOperationAuthority,
        contract: WorthQueryAdmittedConvergenceContract,
        managed_run: WorthQueryAdmittedDirectRun,
        graph: WorthQueryInstalledGraphParticipationAuthority,
    ) -> Result<
        WorthQueryAdmittedDirectConvergenceEpoch,
        WorthQueryDirectConvergenceAdmissionRejection,
    > {
        super::iteration_owner::direct::admit_epoch(self, operation, contract, managed_run, graph)
    }

    pub fn admit_workflow_convergence_epoch(
        &self,
        operation: &WorthQueryExecutionBoundOperationAuthority,
        contract: WorthQueryAdmittedConvergenceContract,
        managed_run: WorthQueryAdmittedWorkflowRun,
        graph: WorthQueryInstalledGraphParticipationAuthority,
    ) -> Result<
        WorthQueryAdmittedWorkflowConvergenceEpoch,
        WorthQueryWorkflowConvergenceAdmissionRejection,
    > {
        super::iteration_owner::workflow::admit_epoch(self, operation, contract, managed_run, graph)
    }
}

pub(super) fn direct_rejection(
    kind: Kind,
    detail: &'static str,
    counters: WorthQueryConvergenceEpochCounters,
    contract: WorthQueryAdmittedConvergenceContract,
    managed_run: WorthQueryAdmittedDirectRun,
    graph: WorthQueryInstalledGraphParticipationAuthority,
) -> WorthQueryDirectConvergenceAdmissionRejection {
    WorthQueryDirectConvergenceAdmissionRejection {
        denial: WorthQueryConvergenceEpochDenial::new(kind, detail, counters),
        contract,
        managed_run,
        graph,
    }
}

pub(super) fn workflow_rejection(
    kind: Kind,
    detail: &'static str,
    counters: WorthQueryConvergenceEpochCounters,
    contract: WorthQueryAdmittedConvergenceContract,
    managed_run: WorthQueryAdmittedWorkflowRun,
    graph: WorthQueryInstalledGraphParticipationAuthority,
) -> WorthQueryWorkflowConvergenceAdmissionRejection {
    WorthQueryWorkflowConvergenceAdmissionRejection {
        denial: WorthQueryConvergenceEpochDenial::new(kind, detail, counters),
        contract,
        managed_run,
        graph,
    }
}
