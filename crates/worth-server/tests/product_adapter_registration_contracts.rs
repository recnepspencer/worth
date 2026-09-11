use std::sync::Arc;

use axum::body::{to_bytes, Body};
use axum::http::{Method, Request, StatusCode};
use serde_json::json;
use tower::ServiceExt;
use worth_query_host::facade::{domain, primary_graph, runtime};
use worth_query_host::facade::{
    worth_query_application_schema, worth_query_aspect, worth_query_entity, worth_query_field,
    worth_query_principal_binding, worth_query_relation,
};
use worth_server::{
    CompatHttpSurface, WorthNativeSurface, WorthServer, WorthServerBuildError,
    WorthServerProductAdapterCertificationCode, WorthServerProductApplicationAdapterRegistration,
    WorthServerProductOperationBasisKind, WorthServerProductOperationDeclaration,
    WorthServerProductOperationErrorMaps, WorthServerProductOperationSupportSnapshot,
};

#[path = "support/product_adapter_phase_nine/fixture.rs"]
mod fixture;

use fixture::{base_config, build_server, result_contract, EditorAdapter};

worth_query_application_schema! {
    pub schema SessionBasisSchema {
        owner: worth_server_session_basis_test,
        version: (1, 0),
        members: |schema| {
            schema
                .entity(SessionExternalMapping::reference())
                .entity(SessionPrincipal::reference())
                .entity(SessionBasisRecord::reference())
                .aspect(SessionExternalMapping::reference(), SessionExternalIdentity::reference())
                .aspect(SessionPrincipal::reference(), SessionPrincipalFacts::reference())
                .aspect(SessionBasisRecord::reference(), SessionBasisFacts::reference())
                .field(SessionExternalMapping::reference(), SessionExternalIdentityField::reference())
                .field(SessionExternalMapping::reference(), SessionMappingStatusField::reference())
                .field(SessionPrincipal::reference(), SessionPrincipalIdentityField::reference())
                .field(SessionBasisRecord::reference(), SessionBasisValue::reference())
                .relation(
                    SessionMappingTarget::reference(),
                    SessionExternalMapping::reference(),
                    SessionPrincipal::reference(),
                )
                .principal_binding(SessionIdentityBinding::reference())
        }
    }
}

worth_query_entity!(pub SessionExternalMapping for SessionBasisSchema);
worth_query_entity!(pub SessionPrincipal for SessionBasisSchema);
worth_query_entity!(pub SessionBasisRecord for SessionBasisSchema);
worth_query_aspect!(pub SessionExternalIdentity for SessionBasisSchema, SessionExternalMapping; identity = AspectIdentity(0x91611101), revision = AspectContractRevision(1),);
worth_query_aspect!(pub SessionPrincipalFacts for SessionBasisSchema, SessionPrincipal; identity = AspectIdentity(0x91611102), revision = AspectContractRevision(1),);
worth_query_field!(pub SessionExternalIdentityField for SessionBasisSchema, SessionExternalMapping, SessionExternalIdentity: worth_query_host::facade::declaration::authentication::WorthQueryExternalPrincipalIdentity => worth_query_host::facade::declaration::authentication::WorthQueryExternalPrincipalIdentityBinding, read_only, equality);
worth_query_field!(pub SessionMappingStatusField for SessionBasisSchema, SessionExternalMapping, SessionExternalIdentity: worth_query_host::facade::declaration::authentication::WorthQueryPrincipalMappingStatus => worth_query_host::facade::declaration::authentication::WorthQueryPrincipalMappingStatusBinding, read_write, equality);
worth_query_field!(pub SessionPrincipalIdentityField for SessionBasisSchema, SessionPrincipal, SessionPrincipalFacts: u64 => worth_query_host::facade::declaration::application_schema::U64ApplicationValueBinding, read_only, equality);
worth_query_relation!(pub SessionMappingTarget in SessionBasisSchema, SessionExternalMapping => SessionPrincipal; integrity = same_context_unbounded_retain_dangling);
worth_query_principal_binding!(
    pub SessionIdentityBinding in SessionBasisSchema,
    mapping SessionExternalMapping {
        identity: SessionExternalIdentityField,
        status: SessionMappingStatusField,
        target: SessionMappingTarget => SessionPrincipal,
        principal_identity: SessionPrincipalIdentityField
    }
);
worth_query_aspect!(pub SessionBasisFacts for SessionBasisSchema, SessionBasisRecord; identity = AspectIdentity(0x91611103), revision = AspectContractRevision(1),);
worth_query_field!(pub SessionBasisValue for SessionBasisSchema, SessionBasisRecord, SessionBasisFacts: String => worth_query_host::facade::declaration::application_schema::StringApplicationValueBinding, read_only, equality);

#[test]
fn product_adapter_registration_rejects_incomplete_authority_or_basis_contract() {
    let result = build_broken_registration_server(
        WorthServerProductOperationDeclaration::product_mutation(
            "product_editor.apply",
            "product-editor.apply.v1",
            result_contract("product-editor.apply.result.v1"),
            WorthServerProductOperationBasisKind::DurableProductDerived,
            WorthServerProductOperationSupportSnapshot::production_admitted(" "),
            "",
        )
        .with_error_map(WorthServerProductOperationErrorMaps::passthrough()),
    );
    assert_registration_code(
        result,
        WorthServerProductAdapterCertificationCode::BlankSupportSnapshotRow,
    );
}

#[test]
fn product_adapter_registration_rejects_missing_error_map_explicitly() {
    let result =
        build_broken_registration_server(WorthServerProductOperationDeclaration::product_read(
            "product_editor.render",
            "product-editor.render.v1",
            result_contract("product-editor.render.result.v1"),
            WorthServerProductOperationBasisKind::DurableProductDerived,
            WorthServerProductOperationSupportSnapshot::production_admitted("render-supported"),
        ));
    assert_registration_code(
        result,
        WorthServerProductAdapterCertificationCode::MissingErrorMap,
    );
}

#[test]
fn product_adapter_registration_rejects_blank_schema_identity_explicitly() {
    let result = build_broken_registration_server(
        WorthServerProductOperationDeclaration::product_read(
            "product_editor.render",
            " ",
            result_contract("product-editor.render.result.v1"),
            WorthServerProductOperationBasisKind::DurableProductDerived,
            WorthServerProductOperationSupportSnapshot::production_admitted("render-supported"),
        )
        .with_error_map(WorthServerProductOperationErrorMaps::passthrough()),
    );
    assert_registration_code(
        result,
        WorthServerProductAdapterCertificationCode::BlankPayloadSchemaIdentity,
    );
}

#[test]
fn primary_graph_application_registration_requires_its_query_owner() {
    let result = build_broken_registration_server(
        WorthServerProductOperationDeclaration::product_read(
            "workflow_editor.render",
            "workflow-editor.render.v1",
            result_contract("workflow-editor.render.result.v1"),
            WorthServerProductOperationBasisKind::PrimaryGraphApplication,
            WorthServerProductOperationSupportSnapshot::production_admitted(
                "workflow-editor-render-supported",
            ),
        )
        .with_error_map(WorthServerProductOperationErrorMaps::passthrough()),
    );
    assert_registration_code(
        result,
        WorthServerProductAdapterCertificationCode::MissingQueryApplicationReadinessProvider,
    );
}

#[tokio::test]
async fn mutation_session_response_carries_registered_primary_graph_basis() {
    let application = Arc::new(primary_graph_application());
    let expected_basis = application
        .on_branch(application.current_world())
        .select()
        .expect("current product branch must select")
        .inspect_application_readiness()
        .expect("published Query application must expose readiness")
        .basis_token()
        .to_string();
    let declaration = WorthServerProductOperationDeclaration::product_mutation(
        "workflow_editor.apply",
        "workflow-editor.apply.v1",
        result_contract("workflow-editor.apply.result.v1"),
        WorthServerProductOperationBasisKind::PrimaryGraphApplication,
        WorthServerProductOperationSupportSnapshot::production_admitted(
            "workflow-editor-apply-supported",
        ),
        "draft",
    )
    .with_primary_graph_application(application)
    .with_error_map(WorthServerProductOperationErrorMaps::passthrough());
    let server = build_server(vec![WorthServerProductApplicationAdapterRegistration::new(
        "primary-graph-editor",
        Arc::new(EditorAdapter::default()),
    )
    .with_operation(declaration)]);
    let response = server
        .projected_router()
        .clone_axum_router()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/compat/mutations/product_session.open_mutation")
                .header("content-type", "application/json")
                .header("x-principal-id", "principal-7")
                .header("x-tenant-id", "tenant-a")
                .header("x-workspace-id", "workspace-42")
                .header("x-branch-id", "branch-9")
                .body(Body::from(
                    serde_json::to_vec(&json!({
                        "operation_name": "workflow_editor.apply",
                        "expiry_seconds": 60
                    }))
                    .expect("session request must serialize"),
                ))
                .expect("session request must build"),
        )
        .await
        .expect("session route must respond");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("session response must read");
    let body: serde_json::Value =
        serde_json::from_slice(&body).expect("session response must be JSON");
    assert_eq!(body["basis"].as_str(), Some(expected_basis.as_str()));
}

fn primary_graph_application(
) -> primary_graph::WorthQueryPrimaryGraphApplicationRuntime<SessionBasisSchema> {
    let declaration = SessionBasisSchema::declaration().expect("test schema must declare");
    let package = domain::WorthQueryPortableDomainPackage::new(
        domain::WorthQueryPortableDomainIdentity::new("worth_server_session_basis_test", 1, 0),
    )
    .application_schema(declaration.clone())
    .validate()
    .expect("test package must validate");
    let admitted = domain::WorthQueryInstallationAdmissionProfile::new("server", "session-basis")
        .admit(package)
        .expect("test package must admit");
    let installation = runtime::WorthQueryExecutionRuntimeInstaller::new()
        .install(
            domain::WorthQueryInstallationGeneration::initial(),
            [admitted],
        )
        .expect("test package must install");
    let (runtime, authority) = installation.into_parts();
    let schema = runtime
        .installed_packages()
        .bind_application_schema(declaration)
        .expect("test schema must bind");
    let binding = schema
        .principal_binding(SessionIdentityBinding::reference())
        .expect("test principal binding must install");
    let mut graph = authority
        .prepare_primary_graph(
            &runtime,
            &schema,
            worth_query_host::facade::runtime::WorthQueryProductWorldResources::install(
                worth_query_host::facade::runtime::RuntimeWorldBudgetInstallation {
                    branches: worth_query_host::facade::runtime::RuntimeWorldBranchBudgetInstallation { live_product_branches: 128 },
                    history: worth_query_host::facade::runtime::RuntimeWorldHistoryBudgetInstallation { retained_composite_commits: 1_024, history_metadata_bytes: 16 * 1024 * 1024 },
                    observations: worth_query_host::facade::runtime::RuntimeWorldObservationBudgetInstallation { active_observations: 512 },
                    publication: worth_query_host::facade::runtime::RuntimeWorldPublicationBudgetInstallation { active_publication_attempts: 128 },
                    recovery: worth_query_host::facade::runtime::RuntimeWorldRecoveryBudgetInstallation { retained_product_unpublished_records: 128, retained_partial_metadata_bytes: 16 * 1024 * 1024 },
                    retention: worth_query_host::facade::runtime::RuntimeWorldRetentionBudgetInstallation { unique_exact_component_pins: 1_024, in_flight_pin_acquisition_reservations: 256 },
                    custody: worth_query_host::facade::runtime::RuntimeWorldCustodyBudgetInstallation { owner_created_component_custody_records: 256 },
                },
                worth_query_host::facade::runtime::WorthQueryProductWorldClock::start(),
            )
            .expect("the server test Product World resources are valid"),
        )
        .expect("test graph must prepare");
    graph
        .bind_principal(
            &binding,
            primary_graph::WorthQueryApplicationPrincipalKey::new("session-principal")
                .expect("principal key must be valid"),
            1_u64,
            worth_query_host::facade::declaration::authentication::WorthQueryExternalPrincipalIdentity::new(
                "https://issuer.example",
                "session-principal",
            )
            .expect("external identity must be valid"),
            worth_query_host::facade::declaration::authentication::WorthQueryPrincipalMappingStatus::Enabled,
        )
        .expect("test principal must bind");
    graph
        .publish_application_runtime(
            runtime,
            authority,
            schema,
            primary_graph::SignalConditionalEvaluationBudget::development(),
        )
        .expect("test application must publish")
}

fn build_broken_registration_server(
    declaration: WorthServerProductOperationDeclaration,
) -> Result<WorthServer, WorthServerBuildError> {
    WorthServer::builder()
        .with_config(base_config())
        .register_operations(worth_server::WorthServerOperationRegistration::phase_two_defaults())
        .register_surface(WorthNativeSurface::enabled())
        .register_surface(CompatHttpSurface::phase_one_enabled())
        .register_product_adapter(
            WorthServerProductApplicationAdapterRegistration::new(
                "broken-editor",
                Arc::new(EditorAdapter::default()),
            )
            .with_operation(declaration),
        )
        .build()
}

fn assert_registration_code(
    result: Result<WorthServer, WorthServerBuildError>,
    expected_code: WorthServerProductAdapterCertificationCode,
) {
    match result {
        Err(WorthServerBuildError::InvalidProductAdapterRegistry(error)) => {
            assert_eq!(error.certification_code(), Some(expected_code));
        }
        other => panic!("expected product adapter registration failure, got {other:?}"),
    }
}
