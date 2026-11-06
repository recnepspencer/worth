use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::Arc;

use worth_query_admission::facade::domain_computation::WorthQueryAdmittedConvergenceContract;
use worth_query_installation::facade::WorthQueryInstalledGraphParticipationAuthority;

use super::super::super::super::{
    admission::{workflow_rejection, WorthQueryWorkflowConvergenceAdmissionRejection},
    WorthQueryConvergenceEpochDenialKind as Kind,
};
use super::super::super::core::{
    WorthQueryConvergenceEpochCore, WorthQueryConvergenceEpochLifecycle,
};
use super::super::WorthQueryAdmittedWorkflowConvergenceEpoch;
use super::WorkflowAdmittedEpochAssociation;
use crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor;
use {
    crate::domain_computation::provider_session::graph_provider::convergence_port::WorthQueryConvergenceDomainProvider,
    crate::domain_computation::WorthQueryAdmittedWorkflowRun,
    crate::domain_computation::WorthQueryExecutionBoundOperationAuthority,
    crate::domain_computation::WorthQueryExecutionRuntime,
};

pub(in crate::domain_computation::convergence_epoch) fn admit_epoch(
    runtime: &WorthQueryExecutionRuntime,
    operation: &WorthQueryExecutionBoundOperationAuthority,
    contract: WorthQueryAdmittedConvergenceContract,
    managed_run: WorthQueryAdmittedWorkflowRun,
    graph: WorthQueryInstalledGraphParticipationAuthority,
) -> Result<
    WorthQueryAdmittedWorkflowConvergenceEpoch,
    WorthQueryWorkflowConvergenceAdmissionRejection,
> {
    let mut lifecycle = WorthQueryConvergenceEpochLifecycle::begin(
        WorkflowAdmissionLifecycleEvent::operation_checked(),
    );
    if !operation.belongs_to(runtime) {
        return Err(workflow_rejection(
            Kind::ForeignQueryRuntime,
            "convergence operation belongs to another Query runtime",
            lifecycle.into_counters(),
            contract,
            managed_run,
            graph,
        ));
    }
    if !operation.belongs_to_current_installation(runtime) {
        return Err(workflow_rejection(
            Kind::StaleInstallationGeneration,
            "convergence operation belongs to a stale installation generation",
            lifecycle.into_counters(),
            contract,
            managed_run,
            graph,
        ));
    }
    lifecycle.record(WorkflowAdmissionLifecycleEvent::contract_checked());
    if !operation.admits_convergence_contract(&contract) {
        return Err(workflow_rejection(
            Kind::ContractOperationMismatch,
            "admitted convergence contract does not belong to the bound operation",
            lifecycle.into_counters(),
            contract,
            managed_run,
            graph,
        ));
    }
    lifecycle.record(WorkflowAdmissionLifecycleEvent::managed_run_checked());
    if !managed_run.belongs_to_operation(operation) {
        return Err(workflow_rejection(
            Kind::ManagedRunOperationMismatch,
            "managed workflow run does not belong to the bound convergence operation",
            lifecycle.into_counters(),
            contract,
            managed_run,
            graph,
        ));
    }
    lifecycle.record(WorkflowAdmissionLifecycleEvent::graph_checked());
    let provider = match bind_provider(operation, &contract, graph) {
        Ok(provider) => provider,
        Err((kind, detail, graph)) => {
            return Err(workflow_rejection(
                kind,
                detail,
                lifecycle.into_counters(),
                contract,
                managed_run,
                graph,
            ))
        }
    };
    let core = WorthQueryConvergenceEpochCore::new(
        operation.binding_identity(),
        managed_run.logical_run_identity(),
        managed_run.identity(),
        provider.graph.authority_identity(),
        contract,
        lifecycle,
    );
    Ok(WorthQueryAdmittedWorkflowConvergenceEpoch {
        association: WorkflowAdmittedEpochAssociation {
            core,
            managed_run,
            graph: provider.graph,
            provider: provider.provider,
        },
    })
}

pub(in crate::domain_computation::convergence_epoch) struct WorkflowAdmissionLifecycleEvent {
    kind: WorkflowAdmissionLifecycleEventKind,
}

pub(in crate::domain_computation::convergence_epoch) enum WorkflowAdmissionLifecycleEventKind {
    OperationChecked,
    ContractChecked,
    ManagedRunChecked,
    GraphChecked,
}

impl WorkflowAdmissionLifecycleEvent {
    fn operation_checked() -> Self {
        Self {
            kind: WorkflowAdmissionLifecycleEventKind::OperationChecked,
        }
    }

    fn contract_checked() -> Self {
        Self {
            kind: WorkflowAdmissionLifecycleEventKind::ContractChecked,
        }
    }

    fn managed_run_checked() -> Self {
        Self {
            kind: WorkflowAdmissionLifecycleEventKind::ManagedRunChecked,
        }
    }

    fn graph_checked() -> Self {
        Self {
            kind: WorkflowAdmissionLifecycleEventKind::GraphChecked,
        }
    }

    pub(in crate::domain_computation::convergence_epoch) fn into_kind(
        self,
    ) -> WorkflowAdmissionLifecycleEventKind {
        self.kind
    }
}

struct BoundProvider {
    graph: WorthQueryInstalledGraphParticipationAuthority,
    provider: Arc<dyn WorthQueryConvergenceDomainProvider>,
}

fn bind_provider(
    operation: &WorthQueryExecutionBoundOperationAuthority,
    contract: &WorthQueryAdmittedConvergenceContract,
    graph: WorthQueryInstalledGraphParticipationAuthority,
) -> Result<
    BoundProvider,
    (
        Kind,
        &'static str,
        WorthQueryInstalledGraphParticipationAuthority,
    ),
> {
    if !operation.admits_convergence_graph(contract, &graph) {
        return Err((
            Kind::GraphOperationMismatch,
            "installed convergence graph does not belong to the operation evidence topology",
            graph,
        ));
    }
    let Some(anchor) = graph.retain_provider_anchor::<WorthQueryGraphProviderAnchor>() else {
        return Err((
            Kind::MissingConvergenceProvider,
            "installed graph authority does not retain the execution provider anchor",
            graph,
        ));
    };
    let Some(provider) = anchor.convergence_provider() else {
        return Err((
            Kind::MissingConvergenceProvider,
            "installed graph provider does not supply convergence semantics",
            graph,
        ));
    };
    let families_match = match catch_unwind(AssertUnwindSafe(|| {
        provider.convergence_families().matches(contract)
    })) {
        Ok(matches) => matches,
        Err(_) => {
            return Err((
                Kind::ConvergenceProviderFamilyInspectionPanicked,
                "installed graph provider panicked while exposing convergence families",
                graph,
            ))
        }
    };
    if !families_match {
        return Err((
            Kind::ConvergenceProviderFamilyMismatch,
            "installed graph provider semantic families do not match the convergence contract",
            graph,
        ));
    }
    Ok(BoundProvider { graph, provider })
}
