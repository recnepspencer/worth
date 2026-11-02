//! A real filesystem Home is the sole mutation and retry owner.
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_contribution::with_bootstrap_advancement_from_request,
    application_entry::WorthQueryApplicationRequestExt,
    application_installation::{ApplicationBuildIdentity, WorthQueryProgramApplicationRuntime},
    WorthQueryApplicationHome,
};
use worth_query_testkit_host::facade::host::{
    application_home as home, application_runtime_surface as identity, checkpoint_capture,
};
use worth_server::*;

pub(super) type Runtime =
    WorthQueryProgramApplicationRuntime<home::schema::HomeIdentitySchema, home::HomeProgram>;
pub(super) fn open(
    contacts: Arc<AtomicUsize>,
) -> WorthQueryApplicationHome<home::schema::HomeIdentitySchema, home::HomeProgram> {
    open_at(home::location(), contacts, true)
}
pub(super) fn open_at(
    location: std::path::PathBuf,
    contacts: Arc<AtomicUsize>,
    fresh: bool,
) -> WorthQueryApplicationHome<home::schema::HomeIdentitySchema, home::HomeProgram> {
    let lease = checkpoint_capture::lease(128 << 20);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let build = ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap();
    let request = if fresh {
        home::request(location, build, Arc::new(AtomicUsize::new(0)), contacts)
    } else {
        home::request_with_initializer(location, build, contacts, |_, _, _| {
            panic!("Server reopen cannot replay initialization")
        })
    };
    with_bootstrap_advancement_from_request(execution, |phase| request.open_in_advancement(&phase))
        .unwrap()
        .unwrap_or_else(|denial| panic!("real Server Home opens: {denial:?}"))
}
pub(super) fn scope() -> WorthQueryRequestScope {
    WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(60)),
        WorthQueryCancellationSource::new().token(),
    )
}
pub(super) fn server(runtime: &Runtime) -> WorthServer {
    let application = runtime.clone();
    let binding = WorthServerQueryApplicationMutationBinding::for_program(
        runtime.clone(),
        0_u64,
        move |admitted| {
            let context = admitted.prepared_request().admission().request_context();
            assert_eq!(
                context.authenticated_principal().principal_id(),
                "principal-7"
            );
            assert_eq!(context.branch_target(), &WorthServerBranchTarget::Main);
            assert_eq!(admitted.operation_name(), "accounts.set");
            let [WorthServerCompatibilityMutationCommand::Update {
                entity_identity,
                aspects,
                ..
            }] = admitted.mutation().commands()
            else {
                panic!("the declared adapter accepts one account update");
            };
            let value = aspects
                .get("balance.value")
                .and_then(serde_json::Value::as_u64)
                .unwrap();
            let scope = scope();
            let principal = identity::inspection_authentication::authenticate(
                application.runtime().installed_schema(),
                &scope,
            );
            Ok(WorthServerQueryMutationSelection::new(
                principal,
                scope,
                application.runtime().current_world(),
                home::native_mutation::SetAccountBalance {
                    account: entity_identity.clone(),
                    balance: value,
                },
            ))
        },
    );
    WorthServer::builder()
        .with_config(
            WorthServerConfig::builder()
                .with_bind_address(([127, 0, 0, 1], 8080).into())
                .with_middleware_config(
                    WorthServerMiddlewareConfig::builder()
                        .with_query_mutation_enabled(true)
                        .build()
                        .unwrap(),
                )
                .with_query_handoff_config(
                    WorthServerQueryHandoffConfig::builder()
                        .with_application_mutation(binding)
                        .build()
                        .unwrap(),
                )
                .build()
                .unwrap(),
        )
        .register_operations(WorthServerOperationRegistration::phase_two_defaults())
        .register_surface(CompatHttpSurface::phase_one_enabled())
        .build()
        .unwrap()
}
pub(super) fn execute(
    server: &WorthServer,
    key: &str,
    value: u64,
    request_id: &str,
) -> WorthServerQueryMutationAnswer {
    let request = WorthServerCompatibilityRequestInput::builder()
        .with_authenticated_principal_id("principal-7")
        .with_tenant_id("tenant-a")
        .with_workspace_id("workspace-42")
        .with_main_branch()
        .with_route_family(WorthServerCompatHttpRouteFamily::Mutation)
        .with_method("POST")
        .with_path("/compat/mutations/accounts.set")
        .with_header("accept", "application/json")
        .with_header("idempotency-key", key)
        .with_header("x-request-id", request_id)
        .with_body_content_type("application/json")
        .with_body_present(true)
        .build()
        .unwrap();
    let prepared = server.compat_http().prepare_request(request).unwrap();
    let input = WorthServerCompatibilityMutationExecutionInput::new(
        prepared,
        "accounts.set",
        serde_json::json!({
            "command": { "family": "update", "entity_identity": "account-001", "aspects": {"balance.value": value} }
        }),
    );
    match server.compat_http().mutate(input).unwrap() {
        WorthServerCompatibilityMutationExecution::Application(answer) => answer,
        WorthServerCompatibilityMutationExecution::Direct(_) => {
            panic!("a keyed request cannot enter direct submission")
        }
    }
}
pub(super) fn balance(runtime: &Runtime) -> u64 {
    let scope = scope();
    let principal = identity::inspection_authentication::authenticate(
        runtime.runtime().installed_schema(),
        &scope,
    );
    let result = runtime
        .request(&principal, &scope)
        .query(home::balance_query::AccountRead("account-001".to_owned()))
        .execute()
        .unwrap();
    result.rows()[0]
}
pub(super) fn assert_contacts(contacts: &AtomicUsize, expected: usize) {
    assert_eq!(contacts.load(Ordering::SeqCst), expected);
}
