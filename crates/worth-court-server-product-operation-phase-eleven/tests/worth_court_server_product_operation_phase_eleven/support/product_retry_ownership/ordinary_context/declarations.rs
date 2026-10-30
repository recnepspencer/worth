// Response replay ownership at the real Server boundary, not native Query custody proof.
pub(crate) use serde::Serialize;
pub(crate) use serde_json::json;
pub(crate) use worth_proof::TransitionOutcome;
pub(crate) use {std::sync::Arc, std::sync::Mutex};
pub(crate) use {
    worth_server::surfaces::WorthNativeSurface, worth_server::WorthServer,
    worth_server::WorthServerCompletedProductOperation, worth_server::WorthServerConfig,
    worth_server::WorthServerDurableProductMutationContract,
    worth_server::WorthServerMiddlewareConfig, worth_server::WorthServerOperationRegistration,
    worth_server::WorthServerProductAdapterCertificationCode,
    worth_server::WorthServerProductAdapterExecutionError,
    worth_server::WorthServerProductAdapterRegistry,
    worth_server::WorthServerProductApplicationAdapter,
    worth_server::WorthServerProductApplicationAdapterRegistration,
    worth_server::WorthServerProductAuthorityScope, worth_server::WorthServerProductIdempotencyKey,
    worth_server::WorthServerProductIdempotencyRetention,
    worth_server::WorthServerProductOperationBaseDigest,
    worth_server::WorthServerProductOperationBasisKind,
    worth_server::WorthServerProductOperationDeclaration,
    worth_server::WorthServerProductOperationDenial,
    worth_server::WorthServerProductOperationErrorMaps,
    worth_server::WorthServerProductOperationOutcome,
    worth_server::WorthServerProductOperationPayload,
    worth_server::WorthServerProductOperationSuccess,
    worth_server::WorthServerProductOperationSupportSnapshot,
    worth_server::WorthServerProductResultContract, worth_server::WorthServerProductResultValue,
    worth_server::WorthServerProductRetryOwner,
    worth_server::WorthServerProductSessionCreationRequest,
    worth_server::WorthServerScheduledProductOperation,
    worth_server::WorthServerWorthNativeProductMutationCommand,
    worth_server::WorthServerWorthNativeSessionInput,
};

#[derive(Clone)]
pub(crate) struct ControlledApplication(pub(crate) Arc<Mutex<ApplicationState>>);
pub(crate) struct ApplicationState {
    pub(crate) allowed: bool,
    pub(crate) posture: &'static str,
}
impl ControlledApplication {
    pub(crate) fn new() -> Self {
        Self(std::sync::Arc::new(std::sync::Mutex::new(
            ApplicationState {
                allowed: true,
                posture: "pending",
            },
        )))
    }
    pub(crate) fn set(&self, allowed: bool, posture: &'static str) {
        *self.0.lock().unwrap() = ApplicationState { allowed, posture };
    }
}
#[derive(Serialize)]
pub(crate) struct ApplicationReply {
    pub(crate) posture: &'static str,
}
impl WorthServerProductResultValue for ApplicationReply {
    fn result_schema_identity(&self) -> &str {
        "controlled.application-reply.v1"
    }
    fn result_schema_version(&self) -> u32 {
        1
    }
}
pub(crate) fn result_contract() -> WorthServerProductResultContract {
    worth_server::WorthServerProductResultContract::canonical_json(
        "controlled.application-reply.v1",
        1,
        1024,
    )
    .unwrap()
}
impl WorthServerProductApplicationAdapter for ControlledApplication {
    fn execute(
        &self,
        _: &WorthServerScheduledProductOperation,
    ) -> Result<WorthServerProductOperationSuccess, WorthServerProductAdapterExecutionError> {
        let state = self.0.lock().unwrap();
        if !state.allowed {
            return Err(
                worth_server::WorthServerProductAdapterExecutionError::denied(
                    worth_server::WorthServerProductOperationDenial::new(
                        "controlled.authentication_denied",
                        "application authentication was revoked",
                    ),
                ),
            );
        }
        worth_server::WorthServerProductOperationSuccess::publish_json(
            "reply",
            &result_contract(),
            &ApplicationReply {
                posture: state.posture,
            },
        )
        .map_err(worth_server::WorthServerProductAdapterExecutionError::invalid_result_artifact)
    }
}
pub(crate) fn declaration(name: &str) -> WorthServerProductOperationDeclaration {
    worth_server::WorthServerProductOperationDeclaration::product_mutation(
        name,
        "controlled.command.v1",
        result_contract(),
        worth_server::WorthServerProductOperationBasisKind::ProductSessionDerived,
        worth_server::WorthServerProductOperationSupportSnapshot::production_admitted(
            "controlled-application",
        ),
        "draft",
    )
    .with_error_map(worth_server::WorthServerProductOperationErrorMaps::passthrough())
}
pub(crate) fn registration(
    app: &ControlledApplication,
    declaration: WorthServerProductOperationDeclaration,
) -> WorthServerProductApplicationAdapterRegistration {
    worth_server::WorthServerProductApplicationAdapterRegistration::new(
        "controlled-application",
        std::sync::Arc::new(app.clone()),
    )
    .with_operation(declaration)
}
pub(crate) fn posture(result: &WorthServerCompletedProductOperation) -> &str {
    match result.outcome() {
        worth_server::WorthServerProductOperationOutcome::Success(success) => {
            success.result_artifact().body().value()["posture"]
                .as_str()
                .unwrap()
        }
        other => panic!("expected application reply: {other:?}"),
    }
}

pub(crate) fn server(application: &ControlledApplication) -> WorthServer {
    let registration = registration(
        application,
        declaration("controlled.native")
            .with_retry_owner(worth_server::WorthServerProductRetryOwner::ApplicationCustody),
    )
    .with_operation(declaration("controlled.cached"))
    .with_operation(
        worth_server::WorthServerProductOperationDeclaration::product_read(
            "controlled.read",
            "controlled.command.v1",
            result_contract(),
            worth_server::WorthServerProductOperationBasisKind::ProductSessionDerived,
            worth_server::WorthServerProductOperationSupportSnapshot::production_admitted(
                "controlled-application",
            ),
        )
        .with_error_map(worth_server::WorthServerProductOperationErrorMaps::passthrough()),
    );
    let config = worth_server::WorthServerConfig::builder()
        .with_bind_address(([127, 0, 0, 1], 8080).into())
        .with_middleware_config(
            worth_server::WorthServerMiddlewareConfig::builder()
                .with_query_mutation_enabled(true)
                .build()
                .unwrap(),
        )
        .build()
        .unwrap();
    worth_server::WorthServer::builder()
        .with_config(config)
        .register_operations(worth_server::WorthServerOperationRegistration::phase_two_defaults())
        .register_surface(worth_server::surfaces::WorthNativeSurface::enabled())
        .register_product_adapter(registration)
        .build()
        .unwrap()
}
pub(crate) fn session(server: &WorthServer) -> worth_server::WorthServerWorthNativeSession {
    let worth_proof::TransitionOutcome::Success(session) = server.worth_native().session(
        worth_server::WorthServerWorthNativeSessionInput::builder()
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
