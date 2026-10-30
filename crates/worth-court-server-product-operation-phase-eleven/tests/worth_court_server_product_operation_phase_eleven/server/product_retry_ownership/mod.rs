// Response replay ownership at the real Server boundary, not native Query custody proof.
#[allow(unused_imports)]
use crate::product_retry_ownership::ordinary_context::*;
use serde_json::json;

#[test]
fn application_custody_reenters_fresh_admission_instead_of_freezing_pending_or_denied_responses() {
    let application =
        crate::product_retry_ownership::ordinary_context::ControlledApplication::new();
    let server = server(&application);
    let session = session(&server);
    for operation in ["controlled.native", "controlled.cached"] {
        application.set(true, "pending");
        let mutation = session
            .product_sessions()
            .open_mutation(
                worth_server::WorthServerProductSessionCreationRequest::for_operation(operation)
                    .with_basis_digest("basis:original")
                    .with_expiry_seconds(300),
            )
            .unwrap();
        let execute = || {
            session
                .product_operations()
                .execute_mutation(
                    worth_server::WorthServerWorthNativeProductMutationCommand::new(
                        operation,
                        worth_server::WorthServerProductOperationPayload::json(
                            "controlled.command.v1",
                            json!({"custody":"original"}),
                        ),
                    )
                    .within(&mutation)
                    .against_base_digest(
                        worth_server::WorthServerProductOperationBaseDigest::new("basis:original")
                            .unwrap(),
                    )
                    .idempotent(
                        worth_server::WorthServerProductIdempotencyKey::new("original-command")
                            .unwrap(),
                    ),
                )
                .unwrap()
        };
        let initial = execute();
        assert_eq!(posture(&initial), "pending");
        application.set(true, "completed");
        let next = execute();
        if operation == "controlled.cached" {
            assert_eq!(posture(&next), "pending");
            assert!(next.retry_diagnostics().is_previously_committed());
            assert!(next
                .retry_diagnostics()
                .adapter_execution_skipped_by_retry());
            continue;
        }
        assert_eq!(posture(&next), "completed");
        assert!(next.retry_receipt().is_none());
        assert!(next.retry_diagnostics().adapter_execution_attempted());
        application.set(false, "completed");
        let denied = execute();
        match denied.outcome() {
            worth_server::WorthServerProductOperationOutcome::Denied(reason) => {
                assert_eq!(reason.reason_key(), "controlled.authentication_denied")
            }
            other => {
                panic!("cached protected response bypassed application authentication: {other:?}")
            }
        }
        assert!(denied.retry_receipt().is_none());
        application.set(true, "completed");
        assert_eq!(
            posture(&execute()),
            "completed",
            "a denial must not freeze subsequent fresh admission"
        );
    }
}

#[test]
fn product_reads_ignore_write_retry_keys_and_reenter_application_authorization() {
    let application =
        crate::product_retry_ownership::ordinary_context::ControlledApplication::new();
    let server = server(&application);
    let session = session(&server);
    let preview = session
        .product_sessions()
        .open_preview(
            worth_server::WorthServerProductSessionCreationRequest::for_operation(
                "controlled.read",
            )
            .with_basis_digest("basis:original")
            .with_expiry_seconds(300),
        )
        .unwrap();
    let read = || {
        session
            .product_operations()
            .execute(
                worth_server::WorthServerProductOperationInput::new(
                    "controlled.read",
                    worth_server::WorthServerProductOperationPayload::json(
                        "controlled.command.v1",
                        json!({"custody":"original"}),
                    ),
                )
                .with_product_session_identity(preview.identity().as_str())
                .with_basis_digest("basis:original")
                .with_idempotency_key(
                    worth_server::WorthServerProductIdempotencyKey::new("read-key").unwrap(),
                ),
            )
            .unwrap()
    };
    let initial = read();
    assert_eq!(posture(&initial), "pending");
    assert!(initial.retry_receipt().is_none());
    application.set(false, "pending");
    let denied = read();
    match denied.outcome() {
        worth_server::WorthServerProductOperationOutcome::Denied(reason) => {
            assert_eq!(reason.reason_key(), "controlled.authentication_denied")
        }
        other => panic!("read retry bypassed fresh application authorization: {other:?}"),
    }
    assert!(denied.retry_receipt().is_none());
    application.set(true, "completed");
    assert_eq!(posture(&read()), "completed");
}

#[test]
fn retry_owner_changes_declaration_identity_without_changing_the_default_contract() {
    let app = crate::product_retry_ownership::ordinary_context::ControlledApplication::new();
    let default = declaration("controlled.native");
    assert_eq!(
        default.retry_owner(),
        worth_server::WorthServerProductRetryOwner::ServerReceipt
    );
    let explicit = default
        .clone()
        .with_retry_owner(worth_server::WorthServerProductRetryOwner::ServerReceipt);
    let owned = default
        .clone()
        .with_retry_owner(worth_server::WorthServerProductRetryOwner::ApplicationCustody);
    let digest = |declaration| {
        worth_server::WorthServerProductAdapterRegistry::build(vec![registration(
            &app,
            declaration,
        )])
        .unwrap()
        .receipts()[0]
            .canonical_digest()
            .to_owned()
    };
    assert_eq!(digest(default.clone()), digest(explicit));
    assert_ne!(digest(default), digest(owned));
}

#[test]
fn application_retry_custody_rejects_reads_coordination_and_durable_executor_contracts() {
    let app = crate::product_retry_ownership::ordinary_context::ControlledApplication::new();
    let support = || {
        worth_server::WorthServerProductOperationSupportSnapshot::production_admitted(
            "controlled-application",
        )
    };
    let declarations = [
        worth_server::WorthServerProductOperationDeclaration::product_read(
            "controlled.read",
            "controlled.command.v1",
            result_contract(),
            worth_server::WorthServerProductOperationBasisKind::ProductSessionDerived,
            support(),
        ),
        worth_server::WorthServerProductOperationDeclaration::product_session_coordination(
            "controlled.coordinate",
            "controlled.command.v1",
            result_contract(),
            worth_server::WorthServerProductOperationBasisKind::ProductSessionDerived,
            support(),
            "coordination",
        ),
        worth_server::WorthServerProductOperationDeclaration::durable_product_mutation(
            "controlled.durable",
            "controlled.command.v1",
            result_contract(),
            support(),
            worth_server::WorthServerDurableProductMutationContract::atomic(
                worth_server::WorthServerProductAuthorityScope::new("controlled-authority")
                    .unwrap(),
                worth_server::WorthServerProductIdempotencyRetention::Indefinite,
            ),
        ),
    ];
    for declaration in declarations {
        let error = worth_server::WorthServerProductAdapterRegistry::build(vec![registration(
            &app,
            declaration
                .with_error_map(worth_server::WorthServerProductOperationErrorMaps::passthrough())
                .with_retry_owner(worth_server::WorthServerProductRetryOwner::ApplicationCustody),
        )])
        .unwrap_err();
        assert_eq!(
            error.certification_code(),
            Some(worth_server::WorthServerProductAdapterCertificationCode::IncompatibleRetryOwner)
        );
    }
}
