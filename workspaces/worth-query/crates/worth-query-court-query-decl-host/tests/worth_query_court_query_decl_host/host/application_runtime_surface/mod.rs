pub(crate) use worth_query_testkit_host::facade::host::application_runtime_surface::*;
use std::time::{Duration, Instant};
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    primary_graph::WorthQueryPrincipalResolutionMode,
    application_installation::ApplicationBuildIdentity,
};
fn installed_application() -> worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<HostIdentitySchema> {
    worth_query_testkit_host::facade::host::application_runtime_surface::installed_application(
        ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
    )
}
#[test]
fn host_facade_publishes_a_narrow_primary_graph_application_runtime() {
    let application = installed_application();
    assert_eq!(application.publication().principal_binding_count(), 1);
    assert_eq!(application.publication().identity_index_count(), 1);
    assert_eq!(
        application.publication().application_equality_index_count(),
        5
    );
    assert_eq!(
        application.installed_schema().schema_name(),
        "HostIdentitySchema"
    );
    let cancellation = WorthQueryCancellationSource::new();
    let request_scope = WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(60)),
        cancellation.token(),
    );
    let account = application
        .with_host_advancement(|phase| {
            application
                .on_branch(application.current_world())
                .select(&phase)
                .expect("the selected product branch remains admitted")
                .resolve_entity(
                    &phase,
                    AccountNumber::reference(),
                    "account-001".to_string(),
                    &request_scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("installed host call opens");
    assert_eq!(account.examined_candidate_count(), 1);
    assert_eq!(
        account.binding_identity(),
        application.publication().binding_identity()
    );
}
mod inspection;
mod comparison;
