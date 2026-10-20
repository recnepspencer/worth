//! Response replay ownership at the real Server boundary, not native Query custody proof.
use serde::Serialize;
use serde_json::json;
use std::sync::{Arc, Mutex};
use worth_proof::TransitionOutcome;
use worth_server::{
    surfaces::WorthNativeSurface, WorthServer, WorthServerCompletedProductOperation,
    WorthServerConfig, WorthServerDurableProductMutationContract, WorthServerMiddlewareConfig,
    WorthServerOperationRegistration, WorthServerProductAdapterCertificationCode,
    WorthServerProductAdapterExecutionError, WorthServerProductAdapterRegistry,
    WorthServerProductApplicationAdapter, WorthServerProductApplicationAdapterRegistration,
    WorthServerProductAuthorityScope, WorthServerProductIdempotencyKey,
    WorthServerProductIdempotencyRetention, WorthServerProductOperationBaseDigest,
    WorthServerProductOperationBasisKind, WorthServerProductOperationDeclaration,
    WorthServerProductOperationDenial, WorthServerProductOperationErrorMaps,
    WorthServerProductOperationOutcome, WorthServerProductOperationPayload,
    WorthServerProductOperationSuccess, WorthServerProductOperationSupportSnapshot,
    WorthServerProductResultContract, WorthServerProductResultValue, WorthServerProductRetryOwner,
    WorthServerProductSessionCreationRequest, WorthServerScheduledProductOperation,
    WorthServerWorthNativeProductMutationCommand, WorthServerWorthNativeSessionInput,
};

#[derive(Clone)]
struct ControlledApplication(Arc<Mutex<ApplicationState>>);
struct ApplicationState {
    allowed: bool,
    posture: &'static str,
}
impl ControlledApplication {
    fn new() -> Self {
        Self(Arc::new(Mutex::new(ApplicationState {
            allowed: true,
            posture: "pending",
        })))
    }
    fn set(&self, allowed: bool, posture: &'static str) {
        *self.0.lock().unwrap() = ApplicationState { allowed, posture };
    }
}
#[derive(Serialize)]
struct ApplicationReply {
    posture: &'static str,
}
impl WorthServerProductResultValue for ApplicationReply {
    fn result_schema_identity(&self) -> &str {
        "controlled.application-reply.v1"
    }
    fn result_schema_version(&self) -> u32 {
        1
    }
}
fn result_contract() -> WorthServerProductResultContract {
    WorthServerProductResultContract::canonical_json("controlled.application-reply.v1", 1, 1024)
        .unwrap()
}
impl WorthServerProductApplicationAdapter for ControlledApplication {
    fn execute(
        &self,
        _: &WorthServerScheduledProductOperation,
    ) -> Result<WorthServerProductOperationSuccess, WorthServerProductAdapterExecutionError> {
        let state = self.0.lock().unwrap();
        if !state.allowed {
            return Err(WorthServerProductAdapterExecutionError::denied(
                WorthServerProductOperationDenial::new(
                    "controlled.authentication_denied",
                    "application authentication was revoked",
                ),
            ));
        }
        WorthServerProductOperationSuccess::publish_json(
            "reply",
            &result_contract(),
            &ApplicationReply {
                posture: state.posture,
            },
        )
        .map_err(WorthServerProductAdapterExecutionError::invalid_result_artifact)
    }
}
fn declaration(name: &str) -> WorthServerProductOperationDeclaration {
    WorthServerProductOperationDeclaration::product_mutation(
        name,
        "controlled.command.v1",
        result_contract(),
        WorthServerProductOperationBasisKind::ProductSessionDerived,
        WorthServerProductOperationSupportSnapshot::production_admitted("controlled-application"),
        "draft",
    )
    .with_error_map(WorthServerProductOperationErrorMaps::passthrough())
}
fn registration(
    app: &ControlledApplication,
    declaration: WorthServerProductOperationDeclaration,
) -> WorthServerProductApplicationAdapterRegistration {
    WorthServerProductApplicationAdapterRegistration::new(
        "controlled-application",
        Arc::new(app.clone()),
    )
    .with_operation(declaration)
}

#[test]
fn application_custody_reenters_fresh_admission_instead_of_freezing_pending_or_denied_responses() {
    let application = ControlledApplication::new();
    let server = server(&application);
    let session = session(&server);
    for operation in ["controlled.native", "controlled.cached"] {
        application.set(true, "pending");
        let mutation = session
            .product_sessions()
            .open_mutation(
                WorthServerProductSessionCreationRequest::for_operation(operation)
                    .with_basis_digest("basis:original")
                    .with_expiry_seconds(300),
            )
            .unwrap();
        let execute = || {
            session
                .product_operations()
                .execute_mutation(
                    WorthServerWorthNativeProductMutationCommand::new(
                        operation,
                        WorthServerProductOperationPayload::json(
                            "controlled.command.v1",
                            json!({"custody":"original"}),
                        ),
                    )
                    .within(&mutation)
                    .against_base_digest(
                        WorthServerProductOperationBaseDigest::new("basis:original").unwrap(),
                    )
                    .idempotent(WorthServerProductIdempotencyKey::new("original-command").unwrap()),
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
            WorthServerProductOperationOutcome::Denied(reason) => {
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
fn posture(result: &WorthServerCompletedProductOperation) -> &str {
    match result.outcome() {
        WorthServerProductOperationOutcome::Success(success) => {
            success.result_artifact().body().value()["posture"]
                .as_str()
                .unwrap()
        }
        other => panic!("expected application reply: {other:?}"),
    }
}

#[test]
fn product_reads_ignore_write_retry_keys_and_reenter_application_authorization() {
    let application = ControlledApplication::new();
    let server = server(&application);
    let session = session(&server);
    let preview = session
        .product_sessions()
        .open_preview(
            WorthServerProductSessionCreationRequest::for_operation("controlled.read")
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
                    WorthServerProductOperationPayload::json(
                        "controlled.command.v1",
                        json!({"custody":"original"}),
                    ),
                )
                .with_product_session_identity(preview.identity().as_str())
                .with_basis_digest("basis:original")
                .with_idempotency_key(WorthServerProductIdempotencyKey::new("read-key").unwrap()),
            )
            .unwrap()
    };
    let initial = read();
    assert_eq!(posture(&initial), "pending");
    assert!(initial.retry_receipt().is_none());
    application.set(false, "pending");
    let denied = read();
    match denied.outcome() {
        WorthServerProductOperationOutcome::Denied(reason) => {
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
    let app = ControlledApplication::new();
    let default = declaration("controlled.native");
    assert_eq!(
        default.retry_owner(),
        WorthServerProductRetryOwner::ServerReceipt
    );
    let explicit = default
        .clone()
        .with_retry_owner(WorthServerProductRetryOwner::ServerReceipt);
    let owned = default
        .clone()
        .with_retry_owner(WorthServerProductRetryOwner::ApplicationCustody);
    let digest = |declaration| {
        WorthServerProductAdapterRegistry::build(vec![registration(&app, declaration)])
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
    let app = ControlledApplication::new();
    let support = || {
        WorthServerProductOperationSupportSnapshot::production_admitted("controlled-application")
    };
    let declarations = [
        WorthServerProductOperationDeclaration::product_read(
            "controlled.read",
            "controlled.command.v1",
            result_contract(),
            WorthServerProductOperationBasisKind::ProductSessionDerived,
            support(),
        ),
        WorthServerProductOperationDeclaration::product_session_coordination(
            "controlled.coordinate",
            "controlled.command.v1",
            result_contract(),
            WorthServerProductOperationBasisKind::ProductSessionDerived,
            support(),
            "coordination",
        ),
        WorthServerProductOperationDeclaration::durable_product_mutation(
            "controlled.durable",
            "controlled.command.v1",
            result_contract(),
            support(),
            WorthServerDurableProductMutationContract::atomic(
                WorthServerProductAuthorityScope::new("controlled-authority").unwrap(),
                WorthServerProductIdempotencyRetention::Indefinite,
            ),
        ),
    ];
    for declaration in declarations {
        let error = WorthServerProductAdapterRegistry::build(vec![registration(
            &app,
            declaration
                .with_error_map(WorthServerProductOperationErrorMaps::passthrough())
                .with_retry_owner(WorthServerProductRetryOwner::ApplicationCustody),
        )])
        .unwrap_err();
        assert_eq!(
            error.certification_code(),
            Some(WorthServerProductAdapterCertificationCode::IncompatibleRetryOwner)
        );
    }
}

fn server(application: &ControlledApplication) -> WorthServer {
    let registration = registration(
        application,
        declaration("controlled.native")
            .with_retry_owner(WorthServerProductRetryOwner::ApplicationCustody),
    )
    .with_operation(declaration("controlled.cached"))
    .with_operation(
        WorthServerProductOperationDeclaration::product_read(
            "controlled.read",
            "controlled.command.v1",
            result_contract(),
            WorthServerProductOperationBasisKind::ProductSessionDerived,
            WorthServerProductOperationSupportSnapshot::production_admitted(
                "controlled-application",
            ),
        )
        .with_error_map(WorthServerProductOperationErrorMaps::passthrough()),
    );
    let config = WorthServerConfig::builder()
        .with_bind_address(([127, 0, 0, 1], 8080).into())
        .with_middleware_config(
            WorthServerMiddlewareConfig::builder()
                .with_query_mutation_enabled(true)
                .build()
                .unwrap(),
        )
        .build()
        .unwrap();
    WorthServer::builder()
        .with_config(config)
        .register_operations(WorthServerOperationRegistration::phase_two_defaults())
        .register_surface(WorthNativeSurface::enabled())
        .register_product_adapter(registration)
        .build()
        .unwrap()
}
fn session(server: &WorthServer) -> worth_server::WorthServerWorthNativeSession {
    let TransitionOutcome::Success(session) = server.worth_native().session(
        WorthServerWorthNativeSessionInput::builder()
            .with_authenticated_principal_id("principal-7")
            .with_tenant_id("tenant-a")
            .with_workspace_id("workspace-42")
            .with_branch_id("branch-9")
            .build()
            .unwrap(),
    ) else {
        panic!("test session admission failed")
    };

    session
}
