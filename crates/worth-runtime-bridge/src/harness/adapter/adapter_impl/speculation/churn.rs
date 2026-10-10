use super::{
    churn_certification::{
        SpeculationBranchIsolationMatrix, SpeculationBranchIsolationRow,
        SpeculationChurnCertification, SpeculationChurnCounterSnapshot,
        SpeculationPreviewReplayBundleSet, SpeculationResourceBoundReport,
    },
    shared, BridgeHarnessError, SpeculationHarnessExecution,
};
use crate::harness::fixtures::BridgeHarnessFixture;

pub(super) fn execute_churn_certification(
    runtime_bridge: &crate::facade::RuntimeBridge,
    fixture: &BridgeHarnessFixture,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<SpeculationHarnessExecution, BridgeHarnessError> {
    let baseline_authoritative_route_digest =
        shared::first_commit_routing_digest(runtime_bridge, fixture, resource_request)?;
    let branch_executions = execute_churn_branches(runtime_bridge, fixture, resource_request)?;
    let final_authoritative_route_digest =
        shared::first_commit_routing_digest(runtime_bridge, fixture, resource_request)?;

    let resource_bound_report =
        build_churn_resource_bound_report(runtime_bridge, branch_executions.as_slice());
    let counter_snapshot = SpeculationChurnCounterSnapshot::from_churn_report(
        branch_executions.len(),
        &resource_bound_report,
    );

    Ok(SpeculationHarnessExecution::Churn {
        certification: SpeculationChurnCertification::new(
            SpeculationPreviewReplayBundleSet::from_replay_bundles(
                branch_executions
                    .iter()
                    .map(|execution| execution.replay_bundle.clone()),
            ),
            SpeculationBranchIsolationMatrix::new(
                branch_executions
                    .into_iter()
                    .map(|execution| execution.branch_isolation_row)
                    .collect(),
                baseline_authoritative_route_digest,
                final_authoritative_route_digest,
            ),
            resource_bound_report,
            counter_snapshot,
        ),
    })
}

struct SpeculationChurnBranchExecution {
    replay_bundle: crate::facade::BridgePreviewReplayBundle,
    branch_isolation_row: SpeculationBranchIsolationRow,
    preview_artifact_count: usize,
    replay_bundle_width: usize,
}

fn execute_churn_branches(
    runtime_bridge: &crate::facade::RuntimeBridge,
    fixture: &BridgeHarnessFixture,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<Vec<SpeculationChurnBranchExecution>, BridgeHarnessError> {
    (0..3)
        .map(|index| execute_one_churn_branch(runtime_bridge, fixture, index, resource_request))
        .collect()
}

fn execute_one_churn_branch(
    runtime_bridge: &crate::facade::RuntimeBridge,
    fixture: &BridgeHarnessFixture,
    index: usize,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<SpeculationChurnBranchExecution, BridgeHarnessError> {
    let session_id = format!("harness:speculation-churn:{index}");
    let preview_session_identity =
        crate::speculation::BridgePreviewSessionIdentity::admit_bridge_owned(session_id.clone());
    let preview_declaration_identity =
        crate::facade::BridgePreviewSessionDeclarationIdentity::admit_bridge_owned(
            session_id.clone(),
        );
    let binding_identity =
        crate::facade::BridgeSpeculativeBranchBindingIdentity::admit_bridge_owned(format!(
            "{session_id}:binding"
        ));
    let truth_branch_identity =
        crate::truth_identity_fixtures::truth_branch_fixture(format!("branch-{index}"));
    let signal_branch_identity =
        crate::facade::BridgeSignalBranchIdentity::admit_bridge_owned(format!("signal:{index}"));
    let snapshot_identity =
        crate::truth_identity_fixtures::truth_snapshot_fixture(format!("{session_id}:snapshot"));
    let admitted = runtime_bridge
        .admit_preview_session(
            preview_session_identity.clone(),
            shared::preview_declaration(
                preview_declaration_identity,
                binding_identity,
                truth_branch_identity.clone(),
                signal_branch_identity,
                snapshot_identity,
            ),
        )
        .map_err(|error| {
            BridgeHarnessError::new(format!("speculation churn admission failed: {error}"))
        })?;
    let (active, execution_record) =
        runtime_bridge.activate_preview_session(admitted, index + 1, 1, 1);
    let (_discarded, discard_record) = runtime_bridge
        .discard_preview_session(
            active,
            &execution_record,
            vec![
                crate::facade::BridgePreviewResidueClass::PreviewDiagnosticsRetained,
                crate::facade::BridgePreviewResidueClass::TemporaryDiagnosticsResidue,
            ],
        )
        .map_err(|error| {
            BridgeHarnessError::new(format!("speculation churn discard failed: {error}"))
        })?;
    let replay_bundle = runtime_bridge
        .replay_preview_bundle(&preview_session_identity)
        .map_err(|error| {
            BridgeHarnessError::new(format!("speculation churn replay failed: {error}"))
        })?;
    let authoritative_route_digest_after_discard =
        shared::first_commit_routing_digest(runtime_bridge, fixture, resource_request)?;

    Ok(SpeculationChurnBranchExecution {
        replay_bundle: replay_bundle.clone(),
        branch_isolation_row: SpeculationBranchIsolationRow::new(
            preview_session_identity,
            truth_branch_identity,
            execution_record.record_identity().clone(),
            discard_record.record_identity().clone(),
            replay_bundle.lifecycle_outcome(),
            authoritative_route_digest_after_discard,
        ),
        preview_artifact_count: execution_record.counters().preview_artifact_count(),
        replay_bundle_width: replay_bundle.counters().replay_bundle_width(),
    })
}

fn build_churn_resource_bound_report(
    runtime_bridge: &crate::facade::RuntimeBridge,
    branch_executions: &[SpeculationChurnBranchExecution],
) -> SpeculationResourceBoundReport {
    SpeculationResourceBoundReport::new(
        runtime_bridge
            .diagnostics()
            .preview_execution_records()
            .len(),
        runtime_bridge.diagnostics().preview_discard_records().len(),
        runtime_bridge
            .diagnostics()
            .preview_promotion_records()
            .len(),
        branch_executions
            .iter()
            .map(|execution| execution.preview_artifact_count)
            .max()
            .unwrap_or(0),
        branch_executions
            .iter()
            .map(|execution| execution.replay_bundle_width)
            .max()
            .unwrap_or(0),
        branch_executions.len() + 2,
    )
}
