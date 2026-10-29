#[allow(unused_imports)] use crate::operation_readiness_preconditions::ordinary_context::*;
use {worth_server::WorthServerCompatibilityMutationPreconditionContext, worth_server::WorthServerOperationReadinessDenialCode};

#[allow(unused_imports)]
use {worth_testkit_server::shared_support_operation_request_runtime as operation_request_runtime};

use {worth_testkit_server::shared_support_operation_request_runtime::compat_prepared_request, worth_testkit_server::shared_support_operation_request_runtime::compat_request_input, worth_testkit_server::shared_support_operation_request_runtime::operation_request_test_server};

#[test]
fn stale_or_invalid_preconditions_deny_before_planning() {
    let server = operation_request_test_server();
    let drifted_request = compat_prepared_request(
        &server,
        None,
        worth_server::WorthServerCompatHttpRouteFamily::Mutation,
        "POST",
        "/compat/mutations/tasks.insert",
        Some("basis:drifted"),
        None,
    );

    let denial = server
        .operation_readiness()
        .evaluate_compatibility_mutation_preconditions(
            worth_server::WorthServerCompatibilityMutationPreconditionContext::new(
                &drifted_request,
                "tasks.insert",
                "mutation-request-digest",
                "basis:observed",
            ),
        )
        .expect_err("drifted basis must deny before any planning");

    assert_eq!(
        denial.code(),
        worth_server::WorthServerOperationReadinessDenialCode::PreconditionFailed
    );
    assert_eq!(
        denial
            .facts()
            .and_then(|facts| facts.expected_basis_digest()),
        Some("basis:drifted")
    );
    assert_eq!(
        denial
            .facts()
            .and_then(|facts| facts.observed_basis_digest()),
        Some("basis:observed")
    );

    let malformed_basis = server
        .operation_readiness()
        .evaluate_compatibility_mutation_preconditions(
            worth_server::WorthServerCompatibilityMutationPreconditionContext::new(
                &prepared_request(
                    &server,
                    compat_request_input(
                        worth_server::WorthServerCompatHttpRouteFamily::Mutation,
                        "POST",
                        "/compat/mutations/tasks.insert",
                    )
                    .with_query_pair("basis", "not a digest"),
                ),
                "tasks.insert",
                "mutation-request-digest",
                "basis:observed",
            ),
        )
        .expect_err("malformed basis precondition should deny");
    assert_eq!(
        malformed_basis.code(),
        worth_server::WorthServerOperationReadinessDenialCode::InvalidPreconditionInput
    );

    let missing_idempotency_binding = server
        .operation_readiness()
        .evaluate_compatibility_mutation_preconditions(
            worth_server::WorthServerCompatibilityMutationPreconditionContext::new(
                &prepared_request(
                    &server,
                    compat_request_input(
                        worth_server::WorthServerCompatHttpRouteFamily::Mutation,
                        "POST",
                        "/compat/mutations/tasks.insert",
                    )
                    .with_header(
                        "x-idempotency-binding",
                        "compat-http-idempotency-binding-v1|key:idem-editor|request:mutation-request-digest",
                    ),
                ),
                "tasks.insert",
                "mutation-request-digest",
                "basis:observed",
            ),
        )
        .expect_err("binding precondition without idempotency key should deny");
    assert_eq!(
        missing_idempotency_binding.code(),
        worth_server::WorthServerOperationReadinessDenialCode::InvalidPreconditionInput
    );

    let branch_rebind = server
        .operation_readiness()
        .evaluate_compatibility_mutation_preconditions(
            worth_server::WorthServerCompatibilityMutationPreconditionContext::new(
                &prepared_request(
                    &server,
                    compat_request_input(
                        worth_server::WorthServerCompatHttpRouteFamily::Mutation,
                        "POST",
                        "/compat/mutations/tasks.insert",
                    )
                    .with_query_pair("base-branch", "branch-elsewhere"),
                ),
                "tasks.insert",
                "mutation-request-digest",
                "basis:observed",
            ),
        )
        .expect_err("conflicting branch/base posture should deny");
    assert_eq!(
        branch_rebind.code(),
        worth_server::WorthServerOperationReadinessDenialCode::PreconditionFailed
    );

    let foreign_session = server
        .operation_readiness()
        .evaluate_compatibility_mutation_preconditions(
            worth_server::WorthServerCompatibilityMutationPreconditionContext::new(
                &prepared_request(
                    &server,
                    compat_request_input(
                        worth_server::WorthServerCompatHttpRouteFamily::Mutation,
                        "POST",
                        "/compat/mutations/tasks.insert",
                    )
                    .with_header("x-product-session", "session-2"),
                ),
                "tasks.insert",
                "mutation-request-digest",
                "basis:observed",
            )
            .with_observed_product_session_identity("session-1"),
        )
        .expect_err("foreign product session should deny");
    assert_eq!(
        foreign_session.code(),
        worth_server::WorthServerOperationReadinessDenialCode::PreconditionFailed
    );

    let valid = server
        .operation_readiness()
        .evaluate_compatibility_mutation_preconditions(
            worth_server::WorthServerCompatibilityMutationPreconditionContext::new(
                &compat_prepared_request(
                    &server,
                    None,
                    worth_server::WorthServerCompatHttpRouteFamily::Mutation,
                    "POST",
                    "/compat/mutations/tasks.insert",
                    None,
                    None,
                ),
                "tasks.insert",
                "mutation-request-digest",
                "basis:observed",
            ),
        )
        .expect("missing optional preconditions should remain admissible");
    assert_eq!(
        valid.request_identity_digest(),
        "compat-http-mutation-request-precondition-v1|basis:none|if-match:none"
    );
}
