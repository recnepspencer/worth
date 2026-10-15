use std::sync::atomic::Ordering;
use worth_proof::TransitionOutcome;
use worth_query::facade::foundation::{QueryExternalIdentityToken, WorthQueryEntityIdentity};
use worth_query::facade::runtime::{
    WorthQueryAspectMutationBuilder, WorthQueryExistingEntityTarget,
    WorthQueryExistingTruthBindingAuthorityLabel, WorthQueryExistingTruthTargetBinding,
    WorthQueryMutationAuthorityIdentity, WorthQueryRuntimeBackendPosture,
    WorthQueryRuntimeFacadeFamily, WorthQueryRuntimeFamilySupport, WorthQueryRuntimeSupportProfile,
    WorthQueryWriteCommand,
};
use worth_server::{
    WorthServerDirectMutationOutcome, WorthServerQueryHandoffDenialCode, WorthServerQueryOperation,
    WorthServerResponseInput, WorthServerSuccessKind,
};

use crate::shared_support_query_handoff_runtime::RealMutationWorkspaceProvider;
use crate::shared_support_worth_native_assertions::{
    family_contract_digest, operator_evidence_record, response_provenance_digest,
    worth_native_session,
};
use crate::shared_support_worth_native_runtime::{build_server, build_server_with_profiled_counting_workspace};

#[path = "direct_mutation_denials.rs"]
mod direct_mutation_denials;

#[test]
fn direct_single_mutation_returns_provenance_bearing_result_boundary() {
    let server = crate::shared_support_worth_native_runtime::build_server_with_workspace_provider(
        RealMutationWorkspaceProvider,
        true,
    );
    let mutation = direct_mutation_success(worth_native_session(&server).direct().mutate(
        &WorthServerQueryOperation::single_mutation("tasks.insert", insert_task("task-1")),
    ));

    assert_eq!(
        mutation
            .response_envelope()
            .success()
            .expect("success envelope")
            .payload()
            .kind(),
        WorthServerSuccessKind::DirectMutation
    );
    let result = mutation.mutation_result();
    let receipt = result
        .single_receipt()
        .expect("single mutation should expose a write receipt");
    let inspection = result
        .single_inspection()
        .expect("single mutation should expose a write inspection");
    assert_eq!(
        result.result_digest(),
        receipt
            .commit_evidence_identity()
            .terminal_projection_for_reporting()
    );
    assert_eq!(result.inspection_digest(), inspection.inspection_digest());
    assert_eq!(
        result.execution_provenance_digest(),
        receipt
            .execution_provenance_chain_digest()
            .expect("single receipt should preserve execution provenance")
    );
    assert_eq!(inspection.commit_identity(), receipt.commit_identity());
    assert_eq!(inspection.snapshot_identity(), receipt.snapshot_identity());
    assert!(mutation
        .canonical_digest()
        .contains(mutation.handoff_digest()));
}

#[test]
fn direct_batch_mutation_preserves_batch_receipt_and_inspection_digests() {
    let server = build_server_with_profiled_counting_workspace(
        WorthQueryRuntimeSupportProfile::scaffold_backend_profile()
            .with_posture(WorthQueryRuntimeBackendPosture::Primary),
    )
    .0;
    let mutation = direct_mutation_success(worth_native_session(&server).direct().mutate(
        &WorthServerQueryOperation::batch_mutation(
            "tasks.seed",
            vec![insert_task("task-a"), insert_task("task-b")],
        ),
    ));

    let result = mutation.mutation_result();
    let receipt = result
        .batch_receipt()
        .expect("batch mutation should expose a batch receipt");
    let inspection = result
        .batch_inspection()
        .expect("batch mutation should expose a batch inspection");
    assert_eq!(receipt.write_count(), 2);
    assert_eq!(inspection.write_receipt_count(), 2);
    assert_eq!(result.result_digest(), receipt.batch_digest());
    assert_eq!(result.inspection_digest(), inspection.inspection_digest());
    assert_eq!(
        result.execution_provenance_digest(),
        receipt
            .execution_provenance()
            .expect("batch receipt should preserve execution provenance")
            .execution_provenance_chain_digest()
    );
}

#[test]
fn direct_mutation_denies_before_write_when_write_family_is_unavailable() {
    let (server, attempted_writes) = build_server_with_profiled_counting_workspace(
        WorthQueryRuntimeSupportProfile::scaffold_backend_profile().with_family_support(
            WorthQueryRuntimeFamilySupport::unsupported(
                WorthQueryRuntimeFacadeFamily::Write,
                "write is intentionally denied in this hostile mutation profile",
            ),
        ),
    );
    let denial = direct_mutation_denied(worth_native_session(&server).direct().mutate(
        &WorthServerQueryOperation::single_mutation("tasks.insert", insert_task("task-1")),
    ));

    assert_eq!(
        denial.code(),
        WorthServerQueryHandoffDenialCode::UnsupportedQueryFacadeFamily
    );
    assert!(denial
        .detail()
        .contains("query workspace does not admit `write` facade family"));
    assert_eq!(
        attempted_writes.load(Ordering::Relaxed),
        0,
        "unsupported write should deny direct mutation before any write attempt"
    );
    let denial_response =
        server
            .responses()
            .shape_with_defaults(WorthServerResponseInput::query_handoff_denied(
                denial.clone(),
            ));
    let denial_evidence = operator_evidence_record(&server, denial_response);
    assert_eq!(
        denial_evidence
            .counter_receipt()
            .counter("response.query_handoff_denial.count")
            .expect("query handoff denial counter")
            .exact_value(),
        1
    );
    assert_eq!(
        denial_evidence
            .counter_receipt()
            .counter("response.query_mutation_success.count")
            .expect("mutation success counter")
            .exact_value(),
        0
    );
    assert_eq!(
        denial_evidence
            .counter_receipt()
            .counter("response.query_read_success.count")
            .expect("read success counter")
            .exact_value(),
        0
    );
}

#[test]
fn direct_mutation_denies_before_write_when_receipt_inspection_family_is_unavailable() {
    let (server, attempted_writes) = build_server_with_profiled_counting_workspace(
        WorthQueryRuntimeSupportProfile::scaffold_backend_profile().with_family_support(
            WorthQueryRuntimeFamilySupport::unsupported(
                WorthQueryRuntimeFacadeFamily::Inspect,
                "inspect is intentionally denied in this hostile mutation profile",
            ),
        ),
    );
    let denial = direct_mutation_denied(worth_native_session(&server).direct().mutate(
        &WorthServerQueryOperation::single_mutation("tasks.insert", insert_task("task-1")),
    ));

    assert_eq!(
        denial.code(),
        WorthServerQueryHandoffDenialCode::UnsupportedQueryFacadeFamily
    );
    assert!(denial
        .detail()
        .contains("query workspace does not admit `inspect` facade family"));
    assert_eq!(
        attempted_writes.load(Ordering::Relaxed),
        0,
        "unsupported inspect should deny direct mutation before any write attempt"
    );
}

#[test]
fn direct_mutation_preserves_mutation_lane_support_and_operator_classification() {
    let server = build_dual_surface_mutation_server();
    let direct = direct_mutation_success(worth_native_session(&server).direct().mutate(
        &WorthServerQueryOperation::single_mutation("tasks.insert", insert_task("task-1")),
    ));
    let compat = compat_mutation_success(server.compat_http().mutate(
        worth_server::WorthServerCompatibilityMutationExecutionInput::new(
            prepared_compat_mutation_request(&server, "tasks.insert"),
            "tasks.insert",
            serde_json::json!({
                "command": {
                    "family": "insert",
                    "collection": "Task",
                    "aspects": {
                        "identity.id": "task-1",
                        "title.value": "Title for task-1"
                    }
                }
            }),
        ),
    ));
    let direct_evidence = operator_evidence_record(&server, direct.response_envelope().clone());

    assert_eq!(
        family_contract_digest(direct.support_posture()),
        family_contract_digest(
            compat
                .envelope()
                .response_envelope()
                .success()
                .expect("compat mutation should succeed")
                .payload()
                .support_posture(),
        )
    );
    assert_eq!(
        response_provenance_digest(direct.response_envelope()),
        response_provenance_digest(compat.envelope().response_envelope())
    );
    assert_eq!(
        direct_evidence.classification(),
        &worth_server::WorthServerOperatorEvidenceClass::QueryMutationSucceeded
    );
    assert_eq!(
        direct_evidence
            .counter_receipt()
            .counter("response.query_mutation_success.count")
            .expect("mutation success counter")
            .exact_value(),
        1
    );
    assert_eq!(
        direct_evidence
            .counter_receipt()
            .counter("response.query_read_success.count")
            .expect("read success counter")
            .exact_value(),
        0
    );
}

fn prepared_compat_mutation_request(
    server: &worth_server::WorthServer,
    operation_name: &str,
) -> worth_server::WorthServerCompatibilityPreparedRequest {
    match server.compat_http().prepare_request(
        worth_server::WorthServerCompatibilityRequestInput::builder()
            .with_authenticated_principal_id("principal-7")
            .with_tenant_id("tenant-a")
            .with_workspace_id("workspace-42")
            .with_branch_id("branch-9")
            .with_route_family(worth_server::WorthServerCompatHttpRouteFamily::Mutation)
            .with_method("POST")
            .with_path(format!("/compat/mutations/{operation_name}"))
            .with_header("accept", "application/json")
            .with_body_content_type("application/json")
            .with_body_present(true)
            .build()
            .expect("compat mutation request should validate"),
    ) {
        TransitionOutcome::Success(prepared) => prepared,
        other => panic!("expected prepared compat mutation request, got {other:?}"),
    }
}

fn build_dual_surface_mutation_server() -> worth_server::WorthServer {
    worth_server::WorthServer::builder()
        .with_config(
            worth_server::WorthServerConfig::builder()
                .with_bind_address(([127, 0, 0, 1], 8080).into())
                .with_request_context_config(
                    worth_server::WorthServerRequestContextConfig::builder()
                        .with_preview_targeting_enabled(true)
                        .with_default_diagnostics_profile(
                            worth_server::request_context::DiagnosticRichnessProfile::Standard,
                        )
                        .build()
                        .expect("request context config should validate"),
                )
                .with_middleware_config(
                    worth_server::WorthServerMiddlewareConfig::builder()
                        .with_query_mutation_enabled(true)
                        .build()
                        .expect("middleware config should validate"),
                )
                .with_query_handoff_config(
                    worth_server::WorthServerQueryHandoffConfig::builder()
                        .with_workspace_provider(RealMutationWorkspaceProvider)
                        .build()
                        .expect("query handoff config should validate"),
                )
                .build()
                .expect("server config should validate"),
        )
        .register_operations(worth_server::WorthServerOperationRegistration::phase_two_defaults())
        .register_surface(worth_server::surfaces::WorthNativeSurface::enabled())
        .register_surface(worth_server::surfaces::CompatHttpSurface::phase_one_enabled())
        .build()
        .expect("dual-surface mutation server should build")
}

fn compat_mutation_success(
    outcome: worth_server::WorthServerCompatibilityMutationOutcome<
        worth_server::WorthServerCompatibilityMutation,
    >,
) -> worth_server::WorthServerCompatibilityMutation {
    match outcome {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected compatibility mutation success, got {other:?}"),
    }
}

fn insert_task(identity: &str) -> WorthQueryWriteCommand {
    WorthQueryAspectMutationBuilder::new()
        .aspect("identity.id", identity)
        .aspect("title.value", format!("Title for {identity}"))
        .build_insert("Task")
        .expect("insert command should build")
}

fn verify_existing_task(
    authoritative_identity: &str,
    resolved_entity_identity: &str,
) -> WorthQueryWriteCommand {
    WorthQueryAspectMutationBuilder::new()
        .aspect("title.value", "Expected title")
        .build_verify_existing(existing_binding(
            authoritative_identity,
            resolved_entity_identity,
        ))
        .expect("verify existing command should build")
}

fn existing_binding(
    authoritative_identity: &str,
    resolved_entity_identity: &str,
) -> WorthQueryExistingTruthTargetBinding {
    WorthQueryExistingTruthTargetBinding::from_entity_target(
        WorthQueryExistingEntityTarget::new(
            WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(
                WorthQueryExistingTruthBindingAuthorityLabel::new(authoritative_identity)
                    .expect("existing-truth authority label"),
            )
            .expect("existing-truth authority identity"),
            WorthQueryEntityIdentity::admit_authored_entity_token(QueryExternalIdentityToken::new(
                std::sync::Arc::from(resolved_entity_identity),
            )),
        )
        .expect("existing entity target should build")
        .in_target_collection("Task")
        .expect("existing entity target collection should build"),
    )
    .expect("existing entity binding should build")
}

fn direct_mutation_success(
    outcome: WorthServerDirectMutationOutcome,
) -> worth_server::WorthServerDirectMutation {
    match outcome {
        TransitionOutcome::Success(value) => value,
        other => panic!("expected successful direct mutation, got {other:?}"),
    }
}

fn direct_mutation_denied(
    outcome: WorthServerDirectMutationOutcome,
) -> worth_server::WorthServerQueryHandoffDenial {
    match outcome {
        TransitionOutcome::Denied(denial) => denial,
        other => panic!("expected denied direct mutation, got {other:?}"),
    }
}
