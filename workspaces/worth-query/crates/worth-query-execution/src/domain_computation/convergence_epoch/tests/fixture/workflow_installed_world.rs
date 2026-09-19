use std::collections::BTreeMap;
use std::sync::{mpsc::Receiver, Arc};

use worth_query_admission::facade::convergence_epoch::WorthQueryAdmittedConvergenceContract;
use worth_query_admission::facade::resource_admission::{
    WorthQueryAdmittedWorkflowResourcePlan, WorthQueryExecutionResourceAdmissionCounters,
    WorthQueryExecutionResourceSupportSnapshot,
};
use worth_query_admission::integration::admit_execution_resource_plan;
use worth_query_declaration::facade::domain_computation::WorthQueryExecutionResourceRequest;
use worth_query_installation::facade::{
    WorthQueryArtifactFamily, WorthQueryInstallationGeneration,
    WorthQueryInstalledGraphParticipationAuthority, WorthQueryOperationGraphAccess,
    WorthQueryOperationWorkflowContract,
};

use crate::domain_computation::artifact_owner::WorthQueryMoveOnlyArtifactHandle;
use crate::domain_computation::managed_run::tests::causal_fixture;
use crate::domain_computation::operation_binding::{
    WorthQueryExecutionCommitPosture, WorthQueryInstalledOperationExecutionSupport,
};
use crate::domain_computation::provider_session::graph_provider::bounded_step::provider_anchor::WorthQueryGraphProviderAnchor;
use crate::domain_computation::{
    WorthQueryAdmittedWorkflowRun, WorthQueryExecutionBoundOperationAuthority,
    WorthQueryExecutionRuntime, WorthQueryExecutionRuntimeInstaller,
    WorthQueryIteratingWorkflowConvergenceEpoch,
};

use super::admitted_basis::admitted_basis;
use super::fixture_identity::{
    CandidateFamily, GRAPH_ROLE, OWNER, WORKFLOW_OPERATION_SLOT, WORKFLOW_STAGE,
};
use super::package::admitted_workflow_package;
use super::provider::{
    execution_support_with_limit, ConvergentProvider, FixtureDisposition, FixtureGraph,
    FixtureReportHistoryProbe, FixtureYieldRecoveryArtifact, FixtureYieldRecoveryProbe,
};
use super::resource_contract::resource_contract;

pub(crate) fn workflow_epoch_fixture(
    disposition: FixtureDisposition,
) -> WorthQueryIteratingWorkflowConvergenceEpoch {
    workflow_admission_fixture(disposition).admit()
}

pub(crate) struct WorkflowAdmissionFixture {
    pub runtime: WorthQueryExecutionRuntime,
    pub operation: WorthQueryExecutionBoundOperationAuthority,
    pub contract: WorthQueryAdmittedConvergenceContract,
    pub managed: WorthQueryAdmittedWorkflowRun,
    pub graph: WorthQueryInstalledGraphParticipationAuthority,
    pub bridge: worth_runtime_bridge::facade::RuntimeBridge,
}

impl WorkflowAdmissionFixture {
    pub(crate) fn admit(self) -> WorthQueryIteratingWorkflowConvergenceEpoch {
        let admitted = match self.runtime.admit_workflow_convergence_epoch(
            &self.operation,
            self.contract,
            self.managed,
            self.graph,
        ) {
            Ok(epoch) => epoch,
            Err(_) => panic!("exact installed workflow authorities must admit convergence epoch"),
        };
        match admitted.start() {
            Ok(epoch) => epoch,
            Err(_) => panic!("installed workflow artifacts must start convergence"),
        }
    }
}

pub(crate) fn workflow_admission_fixture(
    disposition: FixtureDisposition,
) -> WorkflowAdmissionFixture {
    workflow_admission_fixture_with_provider(disposition, ConvergentProvider::new(disposition))
}

pub(crate) fn workflow_admission_fixture_with_report_history_probe(
    disposition: FixtureDisposition,
) -> (WorkflowAdmissionFixture, FixtureReportHistoryProbe) {
    let probe = FixtureReportHistoryProbe::default();
    let provider = ConvergentProvider::new(disposition).with_report_history_probe(probe.clone());
    (
        workflow_admission_fixture_with_provider(disposition, provider),
        probe,
    )
}

pub(crate) fn workflow_yield_pending_admission_fixture() -> (
    WorkflowAdmissionFixture,
    Receiver<WorthQueryMoveOnlyArtifactHandle>,
) {
    let (sender, receiver) = std::sync::mpsc::channel();
    let disposition = FixtureDisposition::YieldThenConverged;
    let provider = ConvergentProvider::new(disposition).with_cleanup_artifact_handle_sender(sender);
    (
        workflow_admission_fixture_with_provider(disposition, provider),
        receiver,
    )
}

pub(crate) fn workflow_yield_recovery_admission_fixture(
) -> (WorkflowAdmissionFixture, FixtureYieldRecoveryProbe) {
    let disposition = FixtureDisposition::YieldThenSuspensionFailure;
    let probe = FixtureYieldRecoveryProbe::default();
    let provider = ConvergentProvider::new(disposition).with_yield_recovery_probe(probe.clone());
    (
        workflow_admission_fixture_with_provider(disposition, provider),
        probe,
    )
}

pub(crate) fn workflow_yield_denial_admission_fixture(
) -> (WorkflowAdmissionFixture, FixtureYieldRecoveryProbe) {
    let disposition = FixtureDisposition::YieldThenCheckpointUnavailable;
    let probe = FixtureYieldRecoveryProbe::default();
    let provider = ConvergentProvider::new(disposition).with_yield_recovery_probe(probe.clone());
    (
        workflow_admission_fixture_with_provider(disposition, provider),
        probe,
    )
}

pub(crate) fn workflow_yield_recovery_artifact_admission_fixture(
    behavior: FixtureYieldRecoveryArtifact,
) -> (
    WorkflowAdmissionFixture,
    Receiver<WorthQueryMoveOnlyArtifactHandle>,
    FixtureYieldRecoveryProbe,
) {
    let (sender, receiver) = std::sync::mpsc::channel();
    let disposition = FixtureDisposition::YieldThenSuspensionFailure;
    let probe = FixtureYieldRecoveryProbe::default();
    let provider = ConvergentProvider::new(disposition).with_yield_recovery_artifact(
        sender,
        behavior,
        probe.clone(),
    );
    (
        workflow_admission_fixture_with_provider(disposition, provider),
        receiver,
        probe,
    )
}

fn workflow_admission_fixture_with_provider(
    disposition: FixtureDisposition,
    provider: ConvergentProvider,
) -> WorkflowAdmissionFixture {
    let installer = WorthQueryExecutionRuntimeInstaller::new();
    let anchor = Arc::new(WorthQueryGraphProviderAnchor::install_convergent::<
        FixtureGraph,
        _,
    >(provider));
    let graph_support = anchor.resource_support().clone();
    let operation_limit = if matches!(disposition, FixtureDisposition::StageQueueContractMismatch) {
        4
    } else {
        8
    };
    let yieldable = matches!(
        disposition,
        FixtureDisposition::YieldThenCheckpointUnavailable
            | FixtureDisposition::YieldThenConverged
            | FixtureDisposition::YieldThenRestorePanic
            | FixtureDisposition::YieldThenCheckpointDropPanic
            | FixtureDisposition::YieldThenSuspensionFailure
    );
    let operation_executor =
        if matches!(disposition, FixtureDisposition::StageQueueContractMismatch) {
            execution_support_with_limit(operation_limit, yieldable)
        } else {
            graph_support.clone()
        };
    let operation_resources = resource_contract(&operation_executor);
    let stage_resources = resource_contract(&graph_support);
    let graph_access = if matches!(
        disposition,
        FixtureDisposition::ChunkedConverged(_) | FixtureDisposition::StageQueueContractMismatch
    ) {
        WorthQueryOperationGraphAccess::Project
    } else {
        WorthQueryOperationGraphAccess::Observe
    };
    let graph = WorthQueryInstalledGraphParticipationAuthority::install(
        installer.installation_runtime(),
        GRAPH_ROLE,
        "workflow-convergence-provider",
        false,
        Option::<String>::None,
        anchor,
    )
    .expect("workflow convergence graph authority must install");
    let (runtime, installation_authority) = installer
        .install(
            WorthQueryInstallationGeneration::initial(),
            [admitted_workflow_package(
                operation_resources,
                stage_resources,
                graph_access,
            )],
        )
        .expect("workflow convergence Query runtime must install")
        .into_parts();
    let operation = runtime
        .installed_packages()
        .domain_operation(OWNER, WORKFLOW_OPERATION_SLOT)
        .expect("fixture workflow operation must be installed");
    let artifact = runtime
        .installed_packages()
        .artifact_contract(
            OWNER,
            CandidateFamily::SEMANTIC_FAMILY,
            worth_query_installation::facade::WorthQueryArtifactSchemaVersion::new(1),
            worth_query_installation::facade::WorthQueryArtifactProtocolVersion::new(1),
        )
        .expect("fixture workflow convergence artifact must be installed");
    let convergence =
        worth_query_admission::facade::convergence_epoch::admit_convergence_epoch_contract(
            &operation, artifact,
        )
        .expect("installed workflow stage and artifact must admit convergence");
    assert_eq!(convergence.evidence_stage_identity(), Some(WORKFLOW_STAGE));
    let operation_support = WorthQueryExecutionResourceSupportSnapshot::new(
        operation_executor,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        None,
    );
    let stage_support = WorthQueryExecutionResourceSupportSnapshot::new(
        graph_support.clone(),
        Vec::new(),
        vec![(GRAPH_ROLE.to_owned(), graph_support)],
        Vec::new(),
        None,
    );
    let basis = admitted_basis();
    let bound = runtime
        .bind_standalone_test_domain_operation(
            &installation_authority,
            &operation,
            &basis,
            &[&graph],
            &[],
            WorthQueryExecutionCommitPosture::ReadOnly,
            WorthQueryInstalledOperationExecutionSupport::workflow(
                operation_support.clone(),
                [(WORKFLOW_STAGE.to_owned(), stage_support.clone())],
            ),
        )
        .expect("real installed workflow must bind to its exact graph authority");
    let operation_envelope = operation_support.executor().envelope();
    let operation_request = WorthQueryExecutionResourceRequest::bounded(
        operation_limit,
        operation_limit,
        operation_envelope.cancellation_safe_point().clone(),
    )
    .allow_yielded_state_posture(operation_envelope.yielded_state_posture())
    .allow_retained_progress_posture(operation_envelope.retained_progress_posture());
    let operation_plan = admit_execution_resource_plan(
        bound.binding_identity(),
        &operation.definition().semantics().resources,
        &operation_request,
        operation_support,
        WorthQueryExecutionResourceAdmissionCounters::default(),
    )
    .expect("fixture workflow operation resources must admit");
    let workflow = match &operation.definition().semantics().workflow {
        WorthQueryOperationWorkflowContract::Declared(workflow) => workflow,
        _ => panic!("fixture workflow must remain declared"),
    };
    let stage_envelope = stage_support.executor().envelope();
    let stage_request = WorthQueryExecutionResourceRequest::bounded(
        operation_limit,
        operation_limit,
        stage_envelope.cancellation_safe_point().clone(),
    )
    .allow_yielded_state_posture(stage_envelope.yielded_state_posture())
    .allow_retained_progress_posture(stage_envelope.retained_progress_posture());
    let stage_plan = admit_execution_resource_plan(
        &format!("{}:{WORKFLOW_STAGE}", bound.binding_identity()),
        &workflow
            .stages()
            .first()
            .expect("fixture stage must exist")
            .semantics()
            .resources,
        &stage_request,
        stage_support,
        WorthQueryExecutionResourceAdmissionCounters::default(),
    )
    .expect("fixture workflow stage resources must admit");
    let resources = WorthQueryAdmittedWorkflowResourcePlan::assemble(
        operation_plan,
        BTreeMap::from([(WORKFLOW_STAGE.to_owned(), stage_plan)]),
    );
    let attempt = runtime
        .start_workflow_resource_attempt(&bound, resources)
        .expect("fixture workflow resource attempt must start");
    let lower = causal_fixture::managed_admission_context();
    let managed = runtime
        .managed_run_admission(&lower.bridge, &lower.relational)
        .admit_workflow(&bound, attempt, lower.read_request())
        .expect("fixture workflow run must admit through Bridge and Relational authorities");
    WorkflowAdmissionFixture {
        runtime,
        operation: bound,
        contract: convergence,
        managed,
        graph,
        bridge: lower.bridge,
    }
}
