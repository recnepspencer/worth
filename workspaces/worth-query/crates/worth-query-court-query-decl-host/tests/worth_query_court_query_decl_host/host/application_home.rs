//! Real directory startup/read/owning close, extended by the ordinary commit seam.
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_contribution::{
        with_bootstrap_advancement_from_request, WorthQueryAdvancementDenial,
        WorthQueryManagedComputationResourceDenial,
    },
    application_entry::{
        WorthQueryApplicationRequestExt, WorthQueryApplicationRequestQueryDenialKind,
    },
    application_installation::ApplicationBuildIdentity,
    primary_graph::WorthQueryConditionalRuntimeInstallationDenialKind,
    WorldCompositionRetained,
};
use worth_query_testkit_host::facade::host::{
    application_home as fixture, application_runtime_surface as identity, checkpoint_capture,
};

#[test]
fn fresh_directory_home_journey() {
    let location = fixture::location();
    let initialized = Arc::new(AtomicUsize::new(0));
    let request = fixture::request(
        location.clone(),
        ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
        Arc::clone(&initialized),
    );
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let home = with_bootstrap_advancement_from_request(execution, |phase| {
        request.open_in_advancement(&phase)
    })
    .expect("the original bootstrap execution opens")
    .unwrap_or_else(|refused| panic!("fresh directory {}: {refused:?}", location.display()));
    assert!(home.is_directory());
    assert_eq!(initialized.load(Ordering::SeqCst), 1);
    {
        let cancellation = WorthQueryCancellationSource::new();
        let scope = WorthQueryRequestScope::new(
            Some(Instant::now() + Duration::from_secs(60)),
            cancellation.token(),
        );
        let runtime = home.runtime();
        let principal = identity::inspection_authentication::authenticate(
            runtime.runtime().installed_schema(),
            &scope,
        );
        let result = runtime
            .request(&principal, &scope)
            .query(identity::inspection_model::AccountRead(
                "account-001".to_owned(),
            ))
            .execute()
            .expect("the actual Home World selects the seeded Native row");
        assert_eq!(result.rows(), &["account-001".to_owned()]);
    }
    let mut retained_runtime = home.runtime().clone();
    let refused = home
        .close(&execution)
        .err()
        .expect("the actual retained Query runtime keeps Store's composition alive");
    assert!(matches!(
        refused.directory_denial(),
        Some(WorldCompositionRetained)
    ));
    {
        let cancellation = WorthQueryCancellationSource::new();
        let scope = WorthQueryRequestScope::new(
            Some(Instant::now() + Duration::from_secs(60)),
            cancellation.token(),
        );
        let principal = identity::inspection_authentication::authenticate(
            retained_runtime.runtime().installed_schema(),
            &scope,
        );
        let denial = retained_runtime
            .request(&principal, &scope)
            .query(identity::inspection_model::AccountRead(
                "account-001".to_owned(),
            ))
            .execute()
            .err()
            .expect("ordinary weak World execution stays inert after sealing");
        assert!(
            matches!(
                denial.kind(),
                WorthQueryApplicationRequestQueryDenialKind::ExecutionRequest(
                    WorthQueryAdvancementDenial::Resource(
                        WorthQueryManagedComputationResourceDenial::NoActiveExecutionScope
                    )
                )
            ),
            "{denial:?}"
        );
    }
    let legacy_denial = retained_runtime
        .close_conditional_runtime()
        .err()
        .expect("the retained Query clone never acquires Store lifecycle custody");
    assert_eq!(
        legacy_denial.kind(),
        WorthQueryConditionalRuntimeInstallationDenialKind::ApplicationHomeOwnsLifecycle,
    );
    drop(retained_runtime);
    let closed = refused
        .try_close(&execution)
        .unwrap_or_else(|refused| panic!("same owning recovery closes: {refused:?}"));
    assert!(closed.is_closed());
    assert_eq!(initialized.load(Ordering::SeqCst), 1);
}
