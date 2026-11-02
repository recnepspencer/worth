//! Real Native-ahead/World-predecessor geometry and owned recovery in both directions.
use super::*;
use std::{num::NonZeroUsize, time::Duration};
use worth_query_host::facade::application_contribution::{
    WorthQueryAdvancementDenial, WorthQueryManagedComputationInterruption,
};
mod restoration;
mod suspension;

fn read_source(directory: &str) -> query::ReadSource {
    query::ReadSource {
        directory: directory.into(),
    }
}
fn authenticate(
    world: &host::Host,
    scope: &WorthQueryRequestScope,
) -> WorthQueryAuthenticatedExternalPrincipal<Catalog> {
    authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            scope,
        ),
    )
    .unwrap()
}
fn stopped_scope() -> WorthQueryRequestScope {
    let cancellation = WorthQueryCancellationSource::new();
    cancellation.cancel();
    WorthQueryRequestScope::new(None, cancellation.token())
}
fn is_cancelled(cause: &WorthQueryAdvancementDenial) -> bool {
    matches!(
        cause,
        WorthQueryAdvancementDenial::Interrupted(
            WorthQueryManagedComputationInterruption::Cancelled
        )
    )
}

fn assert_activation_unavailable<T>(result: Result<T, WorthQueryApplicationRequestQueryDenial>) {
    match result {
        Err(WorthQueryApplicationRequestQueryDenial::ProductSelection(
            WorthQueryProductBranchAdmissionDenial::Refused(
                WorthQueryProductBranchAdmissionRefusalKind::ProductActivationUnavailable,
            ),
        )) => (),
        Err(cause) => panic!(
            "live publication preserves its exact pre-disclosure activation refusal: {cause:?}"
        ),
        Ok(_) => panic!("live publication admission excludes fresh selection"),
    }
}

#[test]
fn pending_generated_materialization_keeps_world_selected_images_and_recovers_both_directions() {
    lifecycle::isolated_allocation_case(run,concat!(module_path!(),"::pending_generated_materialization_keeps_world_selected_images_and_recovers_both_directions"));
}
fn run() {
    journey();
    let region = stats_alloc::Region::new(lifecycle::ALLOCATOR);
    journey();
    let released = region.change();
    assert_eq!(released.bytes_allocated,released.bytes_deallocated,"all actual pending Native/World recovery, predecessor/next images and final reader owners close");
}
fn journey() {
    let calls = Arc::new(provider::Calls::default());
    let configuration = Configuration::new(calls.clone());
    let rules = configuration.rules.clone();
    let prefix = Arc::new(prefix::Calls::default());
    let prefix_rules = Arc::new(prefix_rule::Calls::new(prefix.clone()));
    let world = host::install_geometry(prefix, prefix_rules, configuration);
    let scope = authentication::request_scope();
    let principal = authenticate(&world, &scope);
    population::native_b(&world, &principal, &scope);
    population::produce(&world, &principal, &scope);
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let a = request
        .query(all("A"))
        .open_index::<provider::BoxIndex>()
        .unwrap();
    let b = request
        .query(all("B"))
        .open_index::<provider::BoxIndex>()
        .unwrap();
    let original = request.query(all("A")).execute().unwrap();
    let observation = request.retain_current_read().unwrap();
    let pinned = request.at(&observation);
    let a_source = request
        .query(source::ReadDirectory {
            directory: "A".into(),
        })
        .execute()
        .unwrap();
    let generated_source = a_source.observed_sources()[0].clone();
    let a_scope = a_source.rows()[0].coordinate;
    let b_scope = request
        .query(source::ReadDirectory {
            directory: "B".into(),
        })
        .execute()
        .unwrap()
        .rows()[0]
        .coordinate;
    let scopes = [(a_scope, "A"), (b_scope, "B")];
    rules.capture_complete_source.store(true, Ordering::SeqCst);
    assert_eq!(world.world_active_publication_attempts_for_test(), 0);
    let suspended = suspension::run(&world, generated_source, &rules, scopes);
    assert_suspended(request.query(all("A")).execute());
    let completed = reconstruction::complete(&world, suspended, 20);
    restoration::run(&world, completed, &rules, scopes);
    for directory in ["A", "B"] {
        assert_eq!(
            values(request.query(all(directory)).execute().unwrap().rows()),
            expected(directory, 20)
        );
        assert_eq!(
            values(
                request
                    .query(read_source(directory))
                    .execute()
                    .unwrap()
                    .rows()
            ),
            expected(directory, 20)
        );
        assert_eq!(
            values(
                pinned
                    .query(read_source(directory))
                    .execute()
                    .unwrap()
                    .rows()
            ),
            expected(directory, 10)
        );
    }
    assert_eq!(values(original.rows()), expected("A", 10));
    assert_eq!(
        world.world_active_publication_attempts_for_test(),
        0,
        "settlement releases actual attempt obligations"
    );
    a.close();
    b.close();
}
