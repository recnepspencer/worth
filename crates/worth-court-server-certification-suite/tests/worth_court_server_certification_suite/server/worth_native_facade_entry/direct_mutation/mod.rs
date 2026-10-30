#[allow(unused_imports)]
use crate::worth_native_facade_entry::ordinary_context::direct_mutation::*;
use std::sync::atomic::Ordering;
use worth_proof::TransitionOutcome;
use {
    worth_query::facade::foundation::QueryExternalIdentityToken,
    worth_query::facade::foundation::WorthQueryEntityIdentity,
};
use {
    worth_query::facade::runtime::WorthQueryAspectMutationBuilder,
    worth_query::facade::runtime::WorthQueryExistingEntityTarget,
    worth_query::facade::runtime::WorthQueryExistingTruthBindingAuthorityLabel,
    worth_query::facade::runtime::WorthQueryExistingTruthTargetBinding,
    worth_query::facade::runtime::WorthQueryMutationAuthorityIdentity,
    worth_query::facade::runtime::WorthQueryRuntimeBackendPosture,
    worth_query::facade::runtime::WorthQueryRuntimeFacadeFamily,
    worth_query::facade::runtime::WorthQueryRuntimeFamilySupport,
    worth_query::facade::runtime::WorthQueryRuntimeSupportProfile,
    worth_query::facade::runtime::WorthQueryWriteCommand,
};
use {
    worth_server::WorthServerDirectMutationOutcome,
    worth_server::WorthServerQueryHandoffDenialCode, worth_server::WorthServerQueryOperation,
    worth_server::WorthServerResponseInput, worth_server::WorthServerSuccessKind,
};

use worth_testkit_server::shared_support_query_handoff_runtime::runtime_real_mutation::RealMutationWorkspaceProvider;
use {
    worth_testkit_server::shared_support_worth_native_assertions::family_contract_digest,
    worth_testkit_server::shared_support_worth_native_assertions::operator_evidence_record,
    worth_testkit_server::shared_support_worth_native_assertions::response_provenance_digest,
    worth_testkit_server::shared_support_worth_native_assertions::worth_native_session,
};
use {
    worth_testkit_server::shared_support_worth_native_runtime::build_server,
    worth_testkit_server::shared_support_worth_native_runtime::build_server_with_profiled_counting_workspace,
};

#[test]
fn direct_single_mutation_returns_provenance_bearing_result_boundary() {
    let server = worth_testkit_server::shared_support_worth_native_runtime::build_server_with_workspace_provider(
        RealMutationWorkspaceProvider,
        true,
    );
    let mutation = direct_mutation_success(worth_native_session(&server).direct().mutate(
        &worth_server::WorthServerQueryOperation::single_mutation(
            "tasks.insert",
            insert_task("task-1"),
        ),
    ));

    assert_eq!(
        mutation
            .response_envelope()
            .success()
            .expect("success envelope")
            .payload()
            .kind(),
        worth_server::WorthServerSuccessKind::DirectMutation
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
        worth_query::facade::runtime::WorthQueryRuntimeSupportProfile::scaffold_backend_profile()
            .with_posture(worth_query::facade::runtime::WorthQueryRuntimeBackendPosture::Primary),
    )
    .0;
    let mutation = direct_mutation_success(worth_native_session(&server).direct().mutate(
        &worth_server::WorthServerQueryOperation::batch_mutation(
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
        worth_query::facade::runtime::WorthQueryRuntimeSupportProfile::scaffold_backend_profile()
            .with_family_support(
                worth_query::facade::runtime::WorthQueryRuntimeFamilySupport::unsupported(
                    worth_query::facade::runtime::WorthQueryRuntimeFacadeFamily::Write,
                    "write is intentionally denied in this hostile mutation profile",
                ),
            ),
    );
    let denial = direct_mutation_denied(worth_native_session(&server).direct().mutate(
        &worth_server::WorthServerQueryOperation::single_mutation(
            "tasks.insert",
            insert_task("task-1"),
        ),
    ));

    assert_eq!(
        denial.code(),
        worth_server::WorthServerQueryHandoffDenialCode::UnsupportedQueryFacadeFamily
    );
    assert!(denial
        .detail()
        .contains("query workspace does not admit `write` facade family"));
    assert_eq!(
        attempted_writes.load(std::sync::atomic::Ordering::Relaxed),
        0,
        "unsupported write should deny direct mutation before any write attempt"
    );
    let denial_response = server.responses().shape_with_defaults(
        worth_server::WorthServerResponseInput::query_handoff_denied(denial.clone()),
    );
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
        worth_query::facade::runtime::WorthQueryRuntimeSupportProfile::scaffold_backend_profile()
            .with_family_support(
                worth_query::facade::runtime::WorthQueryRuntimeFamilySupport::unsupported(
                    worth_query::facade::runtime::WorthQueryRuntimeFacadeFamily::Inspect,
                    "inspect is intentionally denied in this hostile mutation profile",
                ),
            ),
    );
    let denial = direct_mutation_denied(worth_native_session(&server).direct().mutate(
        &worth_server::WorthServerQueryOperation::single_mutation(
            "tasks.insert",
            insert_task("task-1"),
        ),
    ));

    assert_eq!(
        denial.code(),
        worth_server::WorthServerQueryHandoffDenialCode::UnsupportedQueryFacadeFamily
    );
    assert!(denial
        .detail()
        .contains("query workspace does not admit `inspect` facade family"));
    assert_eq!(
        attempted_writes.load(std::sync::atomic::Ordering::Relaxed),
        0,
        "unsupported inspect should deny direct mutation before any write attempt"
    );
}

#[test]
fn direct_mutation_preserves_mutation_lane_support_and_operator_classification() {
    let server = build_dual_surface_mutation_server();
    let direct = direct_mutation_success(worth_native_session(&server).direct().mutate(
        &worth_server::WorthServerQueryOperation::single_mutation(
            "tasks.insert",
            insert_task("task-1"),
        ),
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
#[path = "direct_mutation_denials/mod.rs"]
pub(crate) mod direct_mutation_denials;
