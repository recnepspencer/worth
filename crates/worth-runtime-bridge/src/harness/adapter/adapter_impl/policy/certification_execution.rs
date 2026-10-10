use crate::harness::adapter::types::BridgeHarnessError;
use crate::harness::fixtures::BridgeHarnessFixture;

use super::certification_digest_basis::{
    ambient_leak_diagnostics_digest, ambient_leak_policy_sequence_digest,
    ambient_leak_replay_sequence_digest, provenance_diagnostics_digest,
    provenance_policy_equivalence_digest, provenance_replay_digest, rejection_diagnostics_digest,
    rejection_failure_digest,
};
use super::matrices::{PolicyCertificationMatrix, RequestPolicyMatrix, RoutePolicyMatrix};
use super::shared_artifacts::{
    admitted_policy_bundle, admitted_policy_row, combined_counter_snapshot,
    first_commit_routing_digest, first_snapshot_identity, rejected_policy_bundle,
    rejection_policy_row, request_policy_row, route_policy_row,
};
use super::PolicyHarnessExecution;

pub(super) fn execute_provenance_certification(
    runtime_bridge: &crate::facade::RuntimeBridge,
    fixture: &BridgeHarnessFixture,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> Result<PolicyHarnessExecution, BridgeHarnessError> {
    let deterministic = admitted_policy_bundle(
        runtime_bridge,
        crate::facade::BridgePolicyDeclaration::new(
            crate::facade::BridgePolicyDeclarationIdentity::admit_bridge_owned(
                "policy-cert:deterministic-authoritative",
            ),
            crate::facade::BridgeRequestKind::Authoritative,
            crate::facade::BridgeExecutionPolicyClass::DeterministicCanonical,
            crate::facade::BridgeDiagnosticsTier::Standard,
            true,
            true,
        ),
    )?;
    let optimized = admitted_policy_bundle(
        runtime_bridge,
        crate::facade::BridgePolicyDeclaration::new(
            crate::facade::BridgePolicyDeclarationIdentity::admit_bridge_owned(
                "policy-cert:optimized-preview",
            ),
            crate::facade::BridgeRequestKind::Preview,
            crate::facade::BridgeExecutionPolicyClass::Optimized,
            crate::facade::BridgeDiagnosticsTier::Exhaustive,
            false,
            false,
        ),
    )?;
    let policy_digest = provenance_policy_equivalence_digest(&deterministic, &optimized);
    let replay_digest = provenance_replay_digest(&deterministic, &optimized);
    let diagnostics_digest = provenance_diagnostics_digest(&deterministic, &optimized);
    let routing_digest = first_commit_routing_digest(
        runtime_bridge,
        fixture,
        &deterministic.route_policy,
        resource_request,
    )?;
    let policy_matrix = PolicyCertificationMatrix::from_admitted_rows(vec![
        admitted_policy_row("deterministic_authoritative", &deterministic),
        admitted_policy_row("optimized_preview", &optimized),
    ]);
    let policy_provenance_report = runtime_bridge.summarize_policy_provenance_report(vec![
        runtime_bridge.summarize_policy_provenance_row(
            "deterministic_authoritative",
            &deterministic.contract,
            &deterministic.lowered,
            &deterministic.provenance,
            &deterministic.replay_bundle,
        ),
        runtime_bridge.summarize_policy_provenance_row(
            "optimized_preview",
            &optimized.contract,
            &optimized.lowered,
            &optimized.provenance,
            &optimized.replay_bundle,
        ),
    ]);
    let route_policy_matrix = RoutePolicyMatrix::new(vec![
        route_policy_row("deterministic_authoritative", &deterministic),
        route_policy_row("optimized_preview", &optimized),
    ]);
    let request_policy_matrix = RequestPolicyMatrix::new(vec![
        request_policy_row(
            runtime_bridge.summarize_policy_provenance_row(
                "deterministic_authoritative",
                &deterministic.contract,
                &deterministic.lowered,
                &deterministic.provenance,
                &deterministic.replay_bundle,
            ),
            &deterministic,
        ),
        request_policy_row(
            runtime_bridge.summarize_policy_provenance_row(
                "optimized_preview",
                &optimized.contract,
                &optimized.lowered,
                &optimized.provenance,
                &optimized.replay_bundle,
            ),
            &optimized,
        ),
    ]);
    let counter_snapshot = combined_counter_snapshot([&deterministic, &optimized], 0, 0, 0, 0, 0);

    Ok(PolicyHarnessExecution::Provenance {
        policy_digest,
        policy_matrix,
        policy_provenance_report,
        request_policy_matrix,
        route_policy_matrix,
        routing_digest,
        replay_digest,
        diagnostics_digest,
        counter_snapshot,
    })
}

pub(super) fn execute_rejection_certification(
    runtime_bridge: &crate::facade::RuntimeBridge,
) -> Result<PolicyHarnessExecution, BridgeHarnessError> {
    let optimized_authoritative_declaration = crate::facade::BridgePolicyDeclaration::new(
        crate::facade::BridgePolicyDeclarationIdentity::admit_bridge_owned(
            "policy-cert:rejection-optimized-authoritative",
        ),
        crate::facade::BridgeRequestKind::Authoritative,
        crate::facade::BridgeExecutionPolicyClass::Optimized,
        crate::facade::BridgeDiagnosticsTier::Standard,
        false,
        false,
    );
    let optimized_authoritative =
        rejected_policy_bundle(runtime_bridge, optimized_authoritative_declaration.clone())?;
    let replay_forbidden_source =
        crate::harness::fixtures::InMemoryRelationalBridgeSource::default();
    let replay_forbidden_runtime = crate::facade::RuntimeBridge::builder()
        .with_relational_source(replay_forbidden_source.clone())
        .with_truth_branch_head_source(replay_forbidden_source)
        .with_signal_sink(crate::harness::fixtures::RecordingSignalBridgeSink::default())
        .with_policy(crate::facade::BridgeRuntimePolicy::operational().with_replay_artifacts(false))
        .register_mapping(crate::facade::BridgeMappingRegistration::new(
            crate::facade::BridgeMappingId::admit_bridge_owned("policy-cert-registration"),
            crate::facade::TruthPatchScope::for_entity_field(
                crate::facade::MappingSelector::exact("user"),
                worth_foundational::facade::AspectKey::new("profile")
                    .expect("valid native aspect key"),
                worth_foundational::facade::FieldKey::new("name".to_owned())
                    .expect("valid native field key"),
            ),
            crate::snapshot::SnapshotReadContract::scalar(
                worth_foundational::facade::AspectKey::new("profile")
                    .expect("valid native aspect key"),
                worth_foundational::facade::ScalarAspectType::String,
            ),
            crate::facade::SignalInvalidationScope::admit_bridge_owned("signal.policy"),
            crate::facade::CoarseRoutingMode::Direct,
        ))
        .build()
        .map_err(|error| {
            BridgeHarnessError::new(format!(
                "policy rejection certification runtime build failed: {error}"
            ))
        })?;
    let replay_conflict_declaration = crate::facade::BridgePolicyDeclaration::new(
        crate::facade::BridgePolicyDeclarationIdentity::admit_bridge_owned(
            "policy-cert:rejection-replay-conflict",
        ),
        crate::facade::BridgeRequestKind::Preview,
        crate::facade::BridgeExecutionPolicyClass::Optimized,
        crate::facade::BridgeDiagnosticsTier::Standard,
        true,
        true,
    );
    let replay_conflict = rejected_policy_bundle(
        &replay_forbidden_runtime,
        replay_conflict_declaration.clone(),
    )?;
    let failure_digest = rejection_failure_digest(&optimized_authoritative, &replay_conflict);
    let diagnostics_digest =
        rejection_diagnostics_digest(&optimized_authoritative, &replay_conflict);
    let policy_matrix = PolicyCertificationMatrix::from_rejection_rows(vec![
        rejection_policy_row("optimized_authoritative", &optimized_authoritative),
        rejection_policy_row("replay_conflict", &replay_conflict),
    ]);
    let counter_snapshot = crate::facade::BridgePolicyCounters::from_rejections(
        &[
            &optimized_authoritative_declaration,
            &replay_conflict_declaration,
        ],
        &[&optimized_authoritative, &replay_conflict],
        0,
    );

    Ok(PolicyHarnessExecution::Rejection {
        policy_matrix,
        failure_digest,
        diagnostics_digest,
        counter_snapshot,
    })
}

pub(super) fn execute_ambient_leak_certification(
    runtime_bridge: &crate::facade::RuntimeBridge,
    fixture: &BridgeHarnessFixture,
) -> Result<PolicyHarnessExecution, BridgeHarnessError> {
    let preview_before = admitted_policy_bundle(
        runtime_bridge,
        crate::facade::BridgePolicyDeclaration::new(
            crate::facade::BridgePolicyDeclarationIdentity::admit_bridge_owned(
                "policy-cert:preview-before",
            ),
            crate::facade::BridgeRequestKind::Preview,
            crate::facade::BridgeExecutionPolicyClass::Optimized,
            crate::facade::BridgeDiagnosticsTier::Minimal,
            false,
            false,
        ),
    )?;
    let branch_local_resolution = runtime_bridge.resolve_truth_view_policy(
        &crate::facade::HistoricalEvaluationDeclaration::new(
            crate::facade::BridgeTruthViewSelector::branch_snapshot(
                crate::truth_identity_fixtures::truth_branch_fixture("analysis"),
                first_snapshot_identity(fixture),
            ),
            crate::facade::BridgeReplayMode::Enabled,
            crate::facade::BridgeDiagnosticsTier::Standard,
            crate::facade::BridgeDeliveryIntent::PrepareSignalEvaluation,
        ),
    );
    let authoritative_middle = admitted_policy_bundle(
        runtime_bridge,
        crate::facade::BridgePolicyDeclaration::new(
            crate::facade::BridgePolicyDeclarationIdentity::admit_bridge_owned(
                "policy-cert:authoritative-middle",
            ),
            crate::facade::BridgeRequestKind::Authoritative,
            crate::facade::BridgeExecutionPolicyClass::DeterministicCanonical,
            crate::facade::BridgeDiagnosticsTier::Standard,
            true,
            true,
        ),
    )?;
    let historical_resolution = runtime_bridge.resolve_truth_view_policy(
        &crate::facade::HistoricalEvaluationDeclaration::new(
            crate::facade::BridgeTruthViewSelector::branch_snapshot(
                crate::truth_identity_fixtures::truth_branch_fixture("history"),
                first_snapshot_identity(fixture),
            ),
            crate::facade::BridgeReplayMode::Enabled,
            crate::facade::BridgeDiagnosticsTier::Standard,
            crate::facade::BridgeDeliveryIntent::PrepareSignalEvaluation,
        ),
    );
    let preview_after = admitted_policy_bundle(
        runtime_bridge,
        crate::facade::BridgePolicyDeclaration::new(
            crate::facade::BridgePolicyDeclarationIdentity::admit_bridge_owned(
                "policy-cert:preview-after",
            ),
            crate::facade::BridgeRequestKind::Preview,
            crate::facade::BridgeExecutionPolicyClass::Optimized,
            crate::facade::BridgeDiagnosticsTier::Minimal,
            false,
            false,
        ),
    )?;

    let request_policy_matrix = RequestPolicyMatrix::with_truth_view_resolutions(
        branch_local_resolution,
        historical_resolution,
        vec![
            request_policy_row(
                runtime_bridge.summarize_policy_provenance_row(
                    "preview_before",
                    &preview_before.contract,
                    &preview_before.lowered,
                    &preview_before.provenance,
                    &preview_before.replay_bundle,
                ),
                &preview_before,
            ),
            request_policy_row(
                runtime_bridge.summarize_policy_provenance_row(
                    "authoritative_middle",
                    &authoritative_middle.contract,
                    &authoritative_middle.lowered,
                    &authoritative_middle.provenance,
                    &authoritative_middle.replay_bundle,
                ),
                &authoritative_middle,
            ),
            request_policy_row(
                runtime_bridge.summarize_policy_provenance_row(
                    "preview_after",
                    &preview_after.contract,
                    &preview_after.lowered,
                    &preview_after.provenance,
                    &preview_after.replay_bundle,
                ),
                &preview_after,
            ),
        ],
    );
    let policy_provenance_report = runtime_bridge.summarize_policy_provenance_report(vec![
        runtime_bridge.summarize_policy_provenance_row(
            "preview_before",
            &preview_before.contract,
            &preview_before.lowered,
            &preview_before.provenance,
            &preview_before.replay_bundle,
        ),
        runtime_bridge.summarize_policy_provenance_row(
            "authoritative_middle",
            &authoritative_middle.contract,
            &authoritative_middle.lowered,
            &authoritative_middle.provenance,
            &authoritative_middle.replay_bundle,
        ),
        runtime_bridge.summarize_policy_provenance_row(
            "preview_after",
            &preview_after.contract,
            &preview_after.lowered,
            &preview_after.provenance,
            &preview_after.replay_bundle,
        ),
    ]);
    let policy_matrix = PolicyCertificationMatrix::from_admitted_rows(vec![
        admitted_policy_row("preview_before", &preview_before),
        admitted_policy_row("authoritative_middle", &authoritative_middle),
        admitted_policy_row("preview_after", &preview_after),
    ]);
    let policy_digest =
        ambient_leak_policy_sequence_digest(&preview_before, &authoritative_middle, &preview_after);
    let replay_digest =
        ambient_leak_replay_sequence_digest(&preview_before, &authoritative_middle, &preview_after);
    let diagnostics_digest =
        ambient_leak_diagnostics_digest(&preview_before, &authoritative_middle, &preview_after);
    let counter_snapshot = combined_counter_snapshot(
        [&preview_before, &authoritative_middle, &preview_after],
        3,
        2,
        1,
        0,
        0,
    );

    Ok(PolicyHarnessExecution::AmbientLeak {
        policy_digest,
        policy_matrix,
        policy_provenance_report,
        request_policy_matrix,
        replay_digest,
        diagnostics_digest,
        counter_snapshot,
    })
}
