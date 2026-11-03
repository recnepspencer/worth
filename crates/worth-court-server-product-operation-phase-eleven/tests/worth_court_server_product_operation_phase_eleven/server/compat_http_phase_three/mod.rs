#[allow(unused_imports)]
use worth_testkit_server::shared_support_compat_http_phase_three_runtime as compat_http_phase_three_runtime;
#[allow(unused_imports)]
use worth_testkit_server::shared_support_query_handoff_runtime as query_handoff_runtime;
#[allow(unused_imports)]
use worth_testkit_server::shared_support_test_bootstrap_policy as test_bootstrap_policy;

#[allow(unused_imports)]
use worth_testkit_server::shared_support_worth_native_assertions as worth_native_assertions;
#[allow(unused_imports)]
use worth_testkit_server::shared_support_worth_native_runtime as worth_native_runtime;

use serde_json::json;

use {
    worth_testkit_server::shared_support_compat_http_phase_three_runtime::build_phase_three_server,
    worth_testkit_server::shared_support_compat_http_phase_three_runtime::build_phase_three_server_with_workspace_provider,
    worth_testkit_server::shared_support_compat_http_phase_three_runtime::compat_mutation_denied,
    worth_testkit_server::shared_support_compat_http_phase_three_runtime::compat_mutation_execution_input,
    worth_testkit_server::shared_support_compat_http_phase_three_runtime::compat_mutation_success,
    worth_testkit_server::shared_support_compat_http_phase_three_runtime::direct_mutation_success,
    worth_testkit_server::shared_support_compat_http_phase_three_runtime::insert_task,
    worth_testkit_server::shared_support_compat_http_phase_three_runtime::mutation_input,
    worth_testkit_server::shared_support_compat_http_phase_three_runtime::mutation_request_input_for_workspace,
    worth_testkit_server::shared_support_compat_http_phase_three_runtime::prepared_mutation_request,
    worth_testkit_server::shared_support_compat_http_phase_three_runtime::single_insert_body,
};
use {
    worth_testkit_server::shared_support_worth_native_assertions::family_contract_digest,
    worth_testkit_server::shared_support_worth_native_assertions::response_provenance_digest,
    worth_testkit_server::shared_support_worth_native_assertions::worth_native_session,
};

#[test]
fn compat_http_single_insert_matches_worth_native_mutation_on_shared_query_artifacts() {
    let server = build_phase_three_server();
    let body = single_insert_body("task-1");
    let compat = compat_mutation_success(server.compat_http().mutate(
        compat_mutation_execution_input(&server, "tasks.insert", body),
    ));
    let direct = direct_mutation_success(worth_native_session(&server).direct().mutate(
        &worth_server::WorthServerQueryOperation::single_mutation(
            "tasks.insert",
            insert_task("task-1"),
        ),
    ));

    assert_eq!(
        compat
            .envelope()
            .response_envelope()
            .success()
            .expect("compat response should succeed")
            .payload()
            .kind(),
        worth_server::WorthServerSuccessKind::QueryMutation
    );
    assert_eq!(
        family_contract_digest(compat.envelope().support_posture()),
        family_contract_digest(direct.support_posture())
    );
    assert_eq!(
        compat.mutation_result().result_digest(),
        direct.mutation_result().result_digest()
    );
    assert_eq!(
        compat.mutation_result().inspection_digest(),
        direct.mutation_result().inspection_digest()
    );
    assert_eq!(
        response_provenance_digest(compat.envelope().response_envelope()),
        response_provenance_digest(direct.response_envelope())
    );
    // The success helper requires the explicit unkeyed Direct outcome.
    // D9 retired the process retry receipt and its fresh/replayed flag.
}

#[test]
fn compat_http_mutation_denies_forbidden_and_unsupported_families() {
    let server = build_phase_three_server();
    let forbidden =
        compat_mutation_denied(server.compat_http().mutate(compat_mutation_execution_input(
            &server,
            "tasks.verify-existing",
            json!({
                "command": {
                    "family": "verify_existing",
                    "authoritative_identity": "authority:task-1",
                    "resolved_entity_identity": "task-1",
                    "target_collection": "Task",
                    "asserted_aspects": { "title.value": "Expected title" }
                }
            }),
        )));
    let unsupported =
        compat_mutation_denied(server.compat_http().mutate(compat_mutation_execution_input(
            &server,
            "tasks.custom",
            json!({
                "command": {
                    "family": "merge",
                    "entity_identity": "task-1",
                    "aspects": { "title.value": "Unexpected" }
                }
            }),
        )));

    assert_eq!(
        forbidden.code(),
        worth_server::WorthServerQueryHandoffDenialCode::CompatibilityMutationFamilyForbidden
    );
    assert!(forbidden
        .detail()
        .contains("forbidden at the external server boundary"));
    assert_eq!(
        unsupported.code(),
        worth_server::WorthServerQueryHandoffDenialCode::CompatibilityMutationFamilyUnsupported
    );
    assert!(unsupported.detail().contains("`merge` is not supported"));
}

#[test]
fn compat_http_mutation_preconditions_deny_before_any_write_attempt() {
    let attempted_writes = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let server = build_phase_three_server_with_workspace_provider(
        worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledCountingTestWorkspaceProvider::new(
            worth_query::facade::runtime::WorthQueryRuntimeSupportProfile::scaffold_backend_profile(),
            attempted_writes.clone(),
        ),
    );
    let mismatch_basis = worth_server::WorthServerCompatibilityMutationExecutionInput::new(
        prepared_mutation_request(
            &server,
            mutation_input("tasks.insert")
                .with_query_pair("basis", "basis:drifted")
                .build()
                .expect("basis mutation input should validate structurally"),
        ),
        "tasks.insert",
        single_insert_body("task-1"),
    );
    let mismatch_validator = worth_server::WorthServerCompatibilityMutationExecutionInput::new(
        prepared_mutation_request(
            &server,
            mutation_input("tasks.insert")
                .with_header("if-match", "\"validator:wrong\"")
                .build()
                .expect("validator mutation input should validate structurally"),
        ),
        "tasks.insert",
        single_insert_body("task-2"),
    );

    let basis_denial = compat_mutation_denied(server.compat_http().mutate(mismatch_basis));
    let validator_denial = compat_mutation_denied(server.compat_http().mutate(mismatch_validator));

    assert_eq!(
        basis_denial.code(),
        worth_server::WorthServerQueryHandoffDenialCode::CompatibilityMutationPreconditionFailed
    );
    assert_eq!(
        basis_denial
            .facts()
            .and_then(|facts| facts.expected_basis_digest()),
        Some("basis:drifted")
    );
    assert_ne!(
        basis_denial
            .facts()
            .and_then(|facts| facts.observed_basis_digest()),
        basis_denial
            .facts()
            .and_then(|facts| facts.expected_basis_digest())
    );
    assert_eq!(
        validator_denial.code(),
        worth_server::WorthServerQueryHandoffDenialCode::CompatibilityMutationPreconditionFailed
    );
    assert_eq!(
        validator_denial
            .facts()
            .and_then(|facts| facts.expected_validator()),
        Some("\"validator:wrong\"")
    );
    assert_ne!(
        validator_denial
            .facts()
            .and_then(|facts| facts.observed_validator()),
        validator_denial
            .facts()
            .and_then(|facts| facts.expected_validator())
    );
    assert_eq!(
        attempted_writes.load(std::sync::atomic::Ordering::Relaxed),
        0,
        "compat mutation precondition failures must deny before any write executes"
    );
}

#[test]
fn compat_http_idempotency_resolves_identical_retries_and_denies_conflicts() {
    use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome as Outcome;
    use worth_testkit_server::facade::application_mutation as application;
    let attempted_writes = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let home = application::open(
        attempted_writes.clone(),
        worth_query_host::facade::application_installation::ApplicationBuildIdentity::parse(env!(
            "WORTH_IMPLEMENTATION_BUILD_ID"
        ))
        .unwrap(),
    );
    let server = application::server(home.runtime());
    let first = match application::execute(&server, "idem-1", 8, "first").unwrap() {
        Outcome::Committed {
            receipt,
            result: Ok(value),
        } => {
            assert_eq!(value, json!(8));
            receipt
        }
        other => panic!("the original request commits: {other:?}"),
    };
    let retry = match application::execute(&server, "idem-1", 8, "retry").unwrap() {
        Outcome::RecoveredCommitted(receipt) => receipt,
        other => panic!("retry reconstructs the original result: {other:?}"),
    };
    assert_eq!(first.commit_reference(), retry.commit_reference());
    assert_eq!(first.changed_record_count(), retry.changed_record_count());
    assert_eq!(first.outcome_identity(), Some(retry.outcome_identity()));
    assert_eq!(first.emitted_effect_count(), retry.emitted_effect_count());
    assert_eq!(
        first.idempotency_binding().key_identity(),
        retry.idempotency_key_identity()
    );
    assert!(matches!(
        application::execute(&server, "idem-1", 10, "conflict").unwrap(),
        Outcome::IdempotencyIntentDrift
    ));
    assert_eq!(
        attempted_writes.load(std::sync::atomic::Ordering::Relaxed),
        1,
        "identical retries and changed intent never execute a second write"
    );
    assert_eq!(application::balance(home.runtime()), 8);
    match application::execute(&server, "idem-1", 8, "after-conflict").unwrap() {
        Outcome::RecoveredCommitted(again) => assert_eq!(again, retry),
        other => panic!("changed intent cannot alter the retained original result: {other:?}"),
    }
}

#[test]
fn compat_http_idempotency_scope_isolated_by_workspace_target() {
    use worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome as Outcome;
    use worth_testkit_server::facade::application_mutation as application;
    let attempted_writes = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let home = application::open(
        attempted_writes.clone(),
        worth_query_host::facade::application_installation::ApplicationBuildIdentity::parse(env!(
            "WORTH_IMPLEMENTATION_BUILD_ID"
        ))
        .unwrap(),
    );
    let server = application::server(home.runtime());
    let first = match application::execute(&server, "shared-key", 8, "first").unwrap() {
        Outcome::Committed {
            receipt,
            result: Ok(_),
        } => receipt,
        other => panic!("first workspace commits: {other:?}"),
    };
    let second = match application::execute_for_workspace(
        &server,
        "shared-key",
        9,
        "second",
        "workspace-77",
    )
    .unwrap()
    {
        Outcome::Committed {
            receipt,
            result: Ok(_),
        } => receipt,
        other => panic!("another workspace has independent idempotency scope: {other:?}"),
    };
    assert_ne!(first.commit_reference(), second.commit_reference());
    assert_eq!(
        attempted_writes.load(std::sync::atomic::Ordering::Relaxed),
        2,
        "the same idempotency key must not collide across workspace scopes"
    );
    assert_eq!(application::balance(home.runtime()), 9);
}

#[test]
fn compat_http_mutation_denies_before_write_when_inspect_family_is_unavailable() {
    let attempted_writes = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let server = build_phase_three_server_with_workspace_provider(
        worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::ProfiledCountingTestWorkspaceProvider::new(
            worth_query::facade::runtime::WorthQueryRuntimeSupportProfile::scaffold_backend_profile().with_family_support(
                worth_query::facade::runtime::WorthQueryRuntimeFamilySupport::unsupported(
                    worth_query::facade::runtime::WorthQueryRuntimeFacadeFamily::Inspect,
                    "inspect is intentionally denied in this hostile compat mutation profile",
                ),
            ),
            attempted_writes.clone(),
        ),
    );

    let denial = compat_mutation_denied(server.compat_http().mutate(
        compat_mutation_execution_input(&server, "tasks.insert", single_insert_body("task-1")),
    ));

    assert_eq!(
        denial.code(),
        worth_server::WorthServerQueryHandoffDenialCode::UnsupportedQueryFacadeFamily
    );
    assert!(denial
        .detail()
        .contains("does not admit `inspect` facade family"));
    assert_eq!(
        attempted_writes.load(std::sync::atomic::Ordering::Relaxed),
        0,
        "inspect family denial must happen before any compatibility mutation write"
    );
}
