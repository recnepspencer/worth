#[allow(unused_imports)] use crate::compat_http_phase_two::ordinary_context::*;
#[allow(unused_imports)]
use {worth_testkit_server::shared_support_compat_http_phase_two_runtime as compat_http_phase_two_runtime};
#[allow(unused_imports)]
use {worth_testkit_server::shared_support_direct_context_runtime as direct_context_runtime};
#[allow(unused_imports)]
use {worth_testkit_server::shared_support_query_handoff_runtime as query_handoff_runtime};
#[allow(unused_imports)]
use {worth_testkit_server::shared_support_test_bootstrap_policy as test_bootstrap_policy};

#[allow(unused_imports)]
use {worth_testkit_server::shared_support_worth_native_assertions as worth_native_assertions};
#[allow(unused_imports)]
use {worth_testkit_server::shared_support_worth_native_runtime as worth_native_runtime};

use {worth_proof::TransitionOutcome};
use {worth_query::facade::runtime::WorthQueryInspection, worth_query::facade::runtime::WorthQueryRuntimeRemaskProjection, worth_query::facade::runtime::WorthQueryRuntimeRemaskReasonKind};
use {worth_server::WorthServerCompatibilityExecutionInput, worth_server::WorthServerCompatibilityInspection, worth_server::WorthServerCompatibilityRead, worth_server::WorthServerCompatibilityState, worth_server::WorthServerQueryHandoffDenialCode};

use {worth_testkit_server::shared_support_compat_http_phase_two_runtime::branch_head_execution_input, worth_testkit_server::shared_support_compat_http_phase_two_runtime::build_phase_two_server, worth_testkit_server::shared_support_compat_http_phase_two_runtime::build_phase_two_server_with_workspace_provider, worth_testkit_server::shared_support_compat_http_phase_two_runtime::compat_execution_input, worth_testkit_server::shared_support_compat_http_phase_two_runtime::prepared_read_request, worth_testkit_server::shared_support_compat_http_phase_two_runtime::read_input, worth_testkit_server::shared_support_compat_http_phase_two_runtime::worth_native_named_read};
use {worth_testkit_server::shared_support_direct_context_runtime::RemaskWorkspaceProvider};
use {worth_testkit_server::shared_support_query_handoff_runtime::runtime_synthetic::PanicOnReadTestWorkspaceProvider};
use {worth_testkit_server::shared_support_worth_native_assertions::direct_provenance_digest, worth_testkit_server::shared_support_worth_native_assertions::family_contract_digest, worth_testkit_server::shared_support_worth_native_assertions::response_provenance_digest};

#[test]
fn compat_http_read_matches_worth_native_direct_read_on_narrow_canonical_artifacts() {
    let server = build_phase_two_server();
    let (session, declaration) = worth_native_named_read(&server, "users.profile");

    let direct = match session.direct().read(&declaration) {
        worth_proof::TransitionOutcome::Success(value) => value,
        other => panic!("expected direct read success, got {other:?}"),
    };
    let compat = compat_read_success(
        server
            .compat_http()
            .read(compat_execution_input(&server, "users.profile")),
    );

    assert_eq!(
        compat.declaration_digest(),
        declaration.declaration_digest()
    );
    assert_eq!(compat.handoff_digest(), direct.handoff_digest());
    assert_eq!(
        compat.direct_context().basis_digest(),
        direct.direct_context().basis_digest()
    );
    assert_eq!(
        family_contract_digest(compat.support_posture()),
        family_contract_digest(direct.support_posture())
    );
    assert_eq!(
        direct_provenance_digest(compat.direct_context().provenance()),
        direct_provenance_digest(direct.direct_context().provenance())
    );
    assert_eq!(
        response_provenance_digest(compat.response_envelope()),
        response_provenance_digest(direct.response_envelope())
    );
    assert_eq!(
        compat.validator().entity_tag(),
        format!("\"{}\"", compat.validator().canonical_digest())
    );
    assert_eq!(compat.basis_request().requested_basis_digest(), None);
    assert_eq!(compat.conditional_read().if_match(), None);
    assert_eq!(compat.conditional_read().if_none_match(), None);
    assert_private_cache_policy(compat.cache_policy());
}

#[test]
fn compat_http_basis_localization_denies_repeated_and_preview_basis_requests() {
    let baseline_server = build_phase_two_server();
    let admitted_basis_digest = compat_read_success(
        baseline_server
            .compat_http()
            .read(compat_execution_input(&baseline_server, "users.profile")),
    )
    .direct_context()
    .basis_digest()
    .expect("compat read should expose its retained basis digest")
    .to_string();
    let server = build_phase_two_server_with_workspace_provider(PanicOnReadTestWorkspaceProvider);
    let repeated_basis = worth_server::WorthServerCompatibilityExecutionInput::new(
        prepared_read_request(
            &server,
            read_input("users.profile")
                .with_query_pair("basis", "basis-a")
                .with_query_pair("basis", "basis-b")
                .build()
                .expect("repeated basis input should validate structurally"),
        ),
        "users.profile",
    );
    let preview_basis = worth_server::WorthServerCompatibilityExecutionInput::new(
        prepared_read_request(
            &server,
            worth_server::WorthServerCompatibilityRequestInput::builder()
                .with_authenticated_principal_id("principal-7")
                .with_tenant_id("tenant-a")
                .with_workspace_id("workspace-42")
                .with_preview_id("preview-1")
                .with_route_family(worth_server::WorthServerCompatHttpRouteFamily::Read)
                .with_method("GET")
                .with_path("/compat/reads/users.profile")
                .with_query_pair("basis", "basis-preview")
                .build()
                .expect("preview basis input should validate structurally"),
        ),
        "users.profile",
    );
    let drifted_basis = worth_server::WorthServerCompatibilityExecutionInput::new(
        prepared_read_request(
            &server,
            read_input("users.profile")
                .with_query_pair("basis", "basis:drifted")
                .build()
                .expect("drifted basis input should validate structurally"),
        ),
        "users.profile",
    );

    let repeated_denial = compat_denial(server.compat_http().read(repeated_basis));
    let preview_denial = compat_denial(server.compat_http().read(preview_basis));
    let drifted_denial = compat_denial(server.compat_http().read(drifted_basis));

    assert_eq!(
        repeated_denial.code(),
        worth_server::WorthServerQueryHandoffDenialCode::CompatibilityBasisRequestInvalid
    );
    assert_eq!(
        repeated_denial.detail(),
        "compatibility read admits at most one canonical basis query parameter"
    );
    assert_eq!(
        preview_denial.code(),
        worth_server::WorthServerQueryHandoffDenialCode::CompatibilityBasisRequestUnsupported
    );
    assert_eq!(
        preview_denial.detail(),
        "preview-targeted compatibility reads do not admit an additional explicit basis digest"
    );
    assert_eq!(
        drifted_denial.code(),
        worth_server::WorthServerQueryHandoffDenialCode::CompatibilityBasisRequestInvalid
    );
    assert_eq!(
        drifted_denial.detail(),
        format!(
            "compatibility basis request `basis:drifted` drifted from the admitted retained basis `{admitted_basis_digest}`"
        )
    );
}

#[test]
fn compat_http_state_and_inspection_stay_distinct_under_the_same_named_read() {
    let server = build_phase_two_server();
    let state = compat_state_success(
        server
            .compat_http()
            .state(compat_execution_input(&server, "users.profile")),
    );
    let inspection = compat_inspection_success(
        server
            .compat_http()
            .inspect(compat_execution_input(&server, "users.profile")),
    );

    assert_ne!(state.canonical_digest(), inspection.canonical_digest());
    assert_ne!(
        std::mem::discriminant(state.support_posture()),
        std::mem::discriminant(inspection.support_posture())
    );
    assert_eq!(
        state.direct_context().basis_digest(),
        Some(state.runtime_state().basis_for_reporting())
    );
    match inspection.inspection_result().inspection() {
        worth_query::facade::runtime::WorthQueryInspection::LiveView(live) => {
            assert_eq!(
                inspection.direct_context().basis_digest(),
                Some(
                    live.basis_binding_identity()
                        .terminal_projection_for_reporting()
                )
            );
        }
        other => panic!("expected live inspection result, got {other:?}"),
    }
}

#[test]
fn compat_http_conditional_reads_enforce_exact_validator_contracts() {
    let server = build_phase_two_server();
    let baseline = compat_read_success(
        server
            .compat_http()
            .read(compat_execution_input(&server, "users.profile")),
    );
    let head = compat_read_success(
        server
            .compat_http()
            .read(branch_head_execution_input(&server, "users.profile")),
    );
    let not_modified = worth_server::WorthServerCompatibilityExecutionInput::new(
        prepared_read_request(
            &server,
            read_input("users.profile")
                .with_header("if-none-match", baseline.validator().entity_tag())
                .build()
                .expect("conditional request should validate structurally"),
        ),
        "users.profile",
    );
    let mismatch = worth_server::WorthServerCompatibilityExecutionInput::new(
        prepared_read_request(
            &server,
            read_input("users.profile")
                .with_header("if-match", "\"basis:wrong\"")
                .build()
                .expect("precondition request should validate structurally"),
        ),
        "users.profile",
    );
    let simultaneous = worth_server::WorthServerCompatibilityExecutionInput::new(
        prepared_read_request(
            &server,
            read_input("users.profile")
                .with_header("if-match", baseline.validator().entity_tag())
                .with_header("if-none-match", "\"stale\"")
                .build()
                .expect("simultaneous conditional request should validate structurally"),
        ),
        "users.profile",
    );

    let not_modified_denial = compat_denial(server.compat_http().read(not_modified));
    let mismatch_denial = compat_denial(server.compat_http().read(mismatch));
    let simultaneous_denial = compat_denial(server.compat_http().read(simultaneous));

    assert_eq!(
        baseline.validator().entity_tag(),
        head.validator().entity_tag()
    );
    assert_eq!(
        baseline.direct_context().basis_digest(),
        head.direct_context().basis_digest()
    );
    assert_eq!(
        response_provenance_digest(baseline.response_envelope()),
        response_provenance_digest(head.response_envelope())
    );
    assert_private_cache_policy(baseline.cache_policy());
    assert_eq!(
        baseline.cache_policy().canonical_digest(),
        head.cache_policy().canonical_digest()
    );
    assert_eq!(
        not_modified_denial.code(),
        worth_server::WorthServerQueryHandoffDenialCode::CompatibilityConditionalReadNotModified
    );
    assert_eq!(
        not_modified_denial.detail(),
        "compatibility if-none-match validator already matches the canonical read validator"
    );
    assert_eq!(
        mismatch_denial.code(),
        worth_server::WorthServerQueryHandoffDenialCode::CompatibilityConditionalReadPreconditionFailed
    );
    assert_eq!(
        mismatch_denial.detail(),
        "compatibility if-match validator does not match the canonical read validator"
    );
    assert_eq!(
        simultaneous_denial.code(),
        worth_server::WorthServerQueryHandoffDenialCode::CompatibilityConditionalRequestInvalid
    );
    assert_eq!(
        simultaneous_denial.detail(),
        "compatibility read does not admit simultaneous if-match and if-none-match validators"
    );
}

#[test]
fn compat_http_cache_policy_stays_private_for_branch_and_remasked_reads() {
    let server = build_phase_two_server_with_workspace_provider(worth_testkit_server::shared_support_direct_context_runtime::RemaskWorkspaceProvider::new(
        worth_query::facade::runtime::WorthQueryRuntimeRemaskProjection::remasked(
            worth_query::facade::runtime::WorthQueryRuntimeRemaskReasonKind::PolicyDrift,
            "policy:test",
            "tenant-truth:test",
            "tenant-schema:test",
            "relationship-proof:test",
            "schema-context:test",
        ),
    ));
    let read = compat_read_success(
        server
            .compat_http()
            .read(compat_execution_input(&server, "users.profile")),
    );
    let state = compat_state_success(
        server
            .compat_http()
            .state(compat_execution_input(&server, "users.profile")),
    );
    let inspection = compat_inspection_success(
        server
            .compat_http()
            .inspect(compat_execution_input(&server, "users.profile")),
    );

    for cache_policy in [
        read.cache_policy(),
        state.cache_policy(),
        inspection.cache_policy(),
    ] {
        assert_private_cache_policy(cache_policy);
    }
    assert_eq!(
        state.direct_context().remask_posture().disposition(),
        worth_server::WorthServerDirectRemaskDisposition::Remasked
    );
    assert_eq!(
        inspection.direct_context().remask_posture().disposition(),
        worth_server::WorthServerDirectRemaskDisposition::Remasked
    );
}
