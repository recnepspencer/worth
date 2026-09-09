use worth_query_admission::facade::basis::basis_lifecycle;
use worth_query_installation::facade::ApplicationSchema;
use worth_runtime_bridge::facade::{
    BridgeDeliveryIntent, BridgeDiagnosticsTier, BridgeReplayMode, BridgeTruthViewSelector,
    HistoricalEvaluationDeclaration, SnapshotReadPacket,
};

use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationCommitDenialStage as DenialStage, WorthQueryApplicationCommitOutcome,
};
use crate::domain_computation::primary_graph::application_attempt::provider_execution::provider_denial::denied;
use super::WorthQueryPreparedApplicationCommit;
use crate::domain_computation::operation_binding::WorthQueryApplicationOperationBindingInput;
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationOperation, WorthQueryPrimaryGraphApplicationRuntime,
};
use crate::domain_computation::primary_graph::application_attempt::{
    provider_binding::WorthQueryPreparedApplicationProviderAttempt,
    snapshot_lease::WorthQueryApplicationSnapshotLease,
    WorthQueryApplicationIdempotencyBinding,
};
use crate::domain_computation::primary_graph::application_attempt::provider_execution::WorthQueryApplicationAttemptBasis;
use crate::domain_computation::{
    WorthQueryExecutionBoundOperationAuthority, WorthQueryManagedRunTerminalKind,
    WorthQueryManagedTruthReadRequest,
};

#[cfg(test)]
#[path = "running/affinity_tests.rs"]
mod affinity_tests;
#[cfg(test)]
#[path = "running/independent_axis_tests.rs"]
mod independent_axis_tests;
pub(in super::super) mod progression;

pub(in crate::domain_computation::primary_graph::application_attempt::provider_execution) struct WorthQueryRunningApplicationCommit<
    Schema,
    Operation,
    Input,
    Scope,
> {
    admission: crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation<
        Schema,
        Operation,
        Input,
        Scope,
    >,
    lease: WorthQueryApplicationSnapshotLease,
    provider_attempt: WorthQueryPreparedApplicationProviderAttempt,
    authorization: crate::domain_computation::authorization::WorthQueryProviderCommitAuthorization,
    idempotency: WorthQueryApplicationIdempotencyBinding,
    running: crate::domain_computation::WorthQueryRunningDirectRun,
    mutation_run: crate::domain_computation::provider_session::WorthQueryMutationRunBinding,
    attempt_basis: WorthQueryApplicationAttemptBasis,
    aftermath_causality: Option<
        crate::domain_computation::application_aftermath::WorthQueryPendingAftermathCausality,
    >,
}

pub(in crate::domain_computation::primary_graph::application_attempt::provider_execution) fn start_managed_application_commit<
    Schema,
    Operation,
    Input,
    Scope,
>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    prepared: WorthQueryPreparedApplicationCommit<Schema, Operation, Input, Scope>,
) -> Result<
    WorthQueryRunningApplicationCommit<Schema, Operation, Input, Scope>,
    WorthQueryApplicationCommitOutcome,
>
where
    Schema: ApplicationSchema,
{
    let WorthQueryPreparedApplicationCommit {
        mut admission,
        lease,
        provider_attempt,
        authorization,
        idempotency,
        aftermath_causality,
    } = prepared;
    let snapshot = lease.snapshot();
    let attempt_basis = WorthQueryApplicationAttemptBasis::capture(
        application,
        &admission,
        snapshot,
        lease.product(),
    )
    .map_err(|_| denied(DenialStage::ManagedRunAdmission))?;
    let operation = bind_execution_operation(application, &admission, &lease)?;
    let reserved = admission
        .graph_work_mut()
        .take_operation_capacity()
        .ok_or_else(|| denied(DenialStage::ResourceAdmission))?;
    let attempt = application
        .runtime
        .start_reserved_direct_resource_attempt(&operation, reserved)
        .map_err(|_| denied(DenialStage::ResourceAdmission))?;
    let read_request = WorthQueryManagedTruthReadRequest::new(
        lease.basis_descriptor().clone(),
        SnapshotReadPacket::new(Vec::new()),
    );
    let request_bridge = application.bridge.ordinary().fork_managed_request_lane();
    let running = application
        .runtime
        .managed_run_admission(&request_bridge, &application.product_runtime.source)
        .admit_direct(&operation, attempt, read_request)
        .map_err(|_| denied(DenialStage::ManagedRunAdmission))?
        .start();
    let Some(mutation_run) = admission.graph_work().bind_mutation_run(&running) else {
        let _ = running
            .terminate_for_convergence(WorthQueryManagedRunTerminalKind::Failed)
            .cleanup();
        return Err(denied(DenialStage::ManagedRunAdmission));
    };
    Ok(WorthQueryRunningApplicationCommit {
        admission,
        lease,
        provider_attempt,
        authorization,
        idempotency,
        running,
        mutation_run,
        attempt_basis,
        aftermath_causality,
    })
}

fn bind_execution_operation<Schema, Operation, Input, Scope>(
    application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    admission: &WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    lease: &WorthQueryApplicationSnapshotLease,
) -> Result<WorthQueryExecutionBoundOperationAuthority, WorthQueryApplicationCommitOutcome>
where
    Schema: ApplicationSchema,
{
    let snapshot = lease.snapshot();
    application
        .primary_provider
        .observe_managed_application_bridge_plan();
    let branch = admission.graph_work().branch().truth().clone();
    let bridge_snapshot = lease
        .product()
        .bridge_source_observation()
        .snapshot_identity()
        .clone();
    application
        .bridge
        .ordinary()
        .plan_truth_view_packet(
            HistoricalEvaluationDeclaration::new(
                BridgeTruthViewSelector::branch_snapshot(branch, bridge_snapshot),
                BridgeReplayMode::Disabled,
                BridgeDiagnosticsTier::Standard,
                BridgeDeliveryIntent::PrepareSignalEvaluation,
            ),
            SnapshotReadPacket::new(Vec::new()),
        )
        .map_err(|_| denied(DenialStage::BridgePlanning))?;
    let basis = basis_lifecycle()
        .branch_snapshot(
            admission.graph_work_branch().0.clone(),
            format!(
                "relational-snapshot:{}:{}",
                snapshot.snapshot_id().0,
                snapshot.version_id().0
            ),
        )
        .for_mutation_preparation()
        .map_err(|_| denied(DenialStage::BasisAdmission))?
        .admit()
        .map_err(|_| denied(DenialStage::BasisAdmission))?;
    Ok(
        WorthQueryExecutionBoundOperationAuthority::bind_application(
            WorthQueryApplicationOperationBindingInput {
                runtime: &application.runtime,
                owner: application.installed_schema.owner(),
                installed_operation_fingerprint: admission.retain_installed_operation_fingerprint(),
                operation_slot: admission.operation().into(),
                resource_binding_identity: admission.retain_resource_binding_identity(),
                basis: &basis,
                contracts: admission.allowed_graph_contract(),
                graph: &application.primary_graph_authority,
                support: application.primary_provider.application_resource_support(),
                graph_work_session: admission.graph_work_session_identity(),
                graph_work_managed_run: admission.graph_work_managed_run_identity(),
                operation_attempt: admission.admission_identity(),
                schema_binding: admission.binding_identity(),
                snapshot,
                product: lease.product(),
            },
        ),
    )
}
