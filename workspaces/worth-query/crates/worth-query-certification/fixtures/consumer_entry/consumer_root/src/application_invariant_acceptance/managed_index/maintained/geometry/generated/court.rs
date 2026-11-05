//! Actual generated endpoints cross logical Source judgment and physical cutover.
use super::super::super::super::super::authentication;
use super::super::super::super::lifecycle;
use super::super::super::{
    bounds::Coordinates, host, prefix, rule as prefix_rule, schema::Catalog,
};
use super::super::{authoring, provider, query, rule, schema::*, Configuration};
use super::{operation, producer, program, request as action, source};
use crate::application_invariant_acceptance::managed_index::prefix_observation;
use std::sync::{atomic::Ordering, Arc};
use worth_query_host::facade::{
    admission::authenticated_principal::*, application_entry::*, primary_graph::*,
};
mod cancellation;
mod cold_history;
mod interference;
mod late_cancellation;
mod pending;
mod population;
mod reconstruction;

fn all(directory: &str) -> query::FindBoxes {
    query::FindBoxes {
        directory: directory.into(),
        probe: Coordinates::new(-1, -1, 100, 1),
    }
}
fn values(rows: &[query::Row]) -> Vec<(String, Coordinates, Coordinates)> {
    let mut values: Vec<_> = rows
        .iter()
        .map(|row| (row.identity.clone(), row.start, row.end))
        .collect();
    values.sort_by(|a, b| a.0.cmp(&b.0));
    values
}
fn expected(directory: &str, end: i64) -> Vec<(String, Coordinates, Coordinates)> {
    let point = |x| Coordinates::new(x, 0, x, 0);
    let mut rows = vec![("generated-edge".into(), point(0), point(end))];
    if directory == "B" {
        rows.extend([
            ("native-e2".into(), point(50), point(60)),
            ("native-e3".into(), point(50), point(70)),
        ]);
    }
    rows
}
fn assert_suspended<T>(result: Result<T, WorthQueryApplicationRequestQueryDenial>) {
    assert!(
        matches!(result, Err(WorthQueryApplicationRequestQueryDenial::PrincipalResolution(ref cause))
        if cause.kind() == WorthQueryPrincipalResolutionDenialKind::Refused(WorthQueryPrincipalResolutionRefusalKind::BranchMaterializationSuspended)),
        "current physical reads must refuse suspension before provider query"
    );
}
fn assert_rules(
    rules: &rule::Calls,
    scopes: [((u32, u64, u32), &str); 2],
    committed: bool,
    end: i64,
) {
    let rows = rules.records.lock().unwrap();
    assert_eq!(
        rows.len(),
        2,
        "both actual Directory scopes run the ordinary rule"
    );
    for (scope, directory) in scopes {
        let row = rows.iter().find(|row| row.scope == scope).unwrap();
        assert_eq!(
            values(row.source_after.as_ref().unwrap()),
            expected(directory, end)
        );
        if committed {
            assert_eq!(
                values(row.source_before.as_ref().unwrap()),
                expected(directory, 10)
            );
        } else {
            assert!(row.source_before.is_none());
            assert!(row.before.is_none());
        }
    }
}
#[test]
fn generated_geometry_suspension_keeps_endpoint_source_and_judges_reconstruction() {
    lifecycle::isolated_allocation_case(
        run,
        concat!(
            module_path!(),
            "::generated_geometry_suspension_keeps_endpoint_source_and_judges_reconstruction"
        ),
    );
}
fn run() {
    journey();
    let region = stats_alloc::Region::new(lifecycle::ALLOCATOR);
    journey();
    let released = region.change();
    assert_eq!(
        released.bytes_allocated, released.bytes_deallocated,
        "all actual endpoint producer, Source, suspension, reconstruction and read owners close"
    );
}
fn journey() {
    let provider = Arc::new(provider::Calls::default());
    let config = Configuration::new(provider.clone());
    let rules = config.rules.clone();
    let prefix = Arc::new(prefix_observation::Calls::default());
    let prefix_rules = Arc::new(prefix_rule::Calls::new(prefix.clone()));
    let world = host::install_geometry(prefix, prefix_rules, config);
    let scope = authentication::request_scope();
    let principal = authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ),
    )
    .unwrap();
    population::native_b(&world, &principal, &scope);
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let a = request
        .query(all("A"))
        .open_index::<provider::BoxIndex>()
        .unwrap();
    population::produce(&world, &principal, &scope);
    let original = request.query(all("A")).execute().unwrap();
    assert_eq!(values(original.rows()), expected("A", 10));
    let a_source = request
        .query(source::ReadDirectory {
            directory: "A".into(),
        })
        .execute()
        .unwrap();
    let source = a_source.observed_sources()[0].clone();
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
    assert_ne!(a_scope, b_scope);
    rules.capture_complete_source.store(true, Ordering::SeqCst);
    let selected_position = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    cancellation::suspension(&world, &rules, source.clone(), scopes);
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &selected_position
    );
    assert_eq!(
        values(request.query(all("A")).execute().unwrap().rows()),
        expected("A", 10)
    );
    rules.records.lock().unwrap().clear();
    let selected = world
        .with_host_advancement(|phase| {
            world
                .on_branch(world.current_world())
                .select(&phase)
                .unwrap()
        })
        .unwrap();
    let suspended = selected
        .suspend_current_generated_output::<producer::Producer>(&scope, source)
        .unwrap();
    assert_rules(&rules, scopes, true, 10);
    let queries = provider.probe_validations.load(Ordering::SeqCst);
    assert_suspended(request.query(all("A")).execute());
    assert_eq!(provider.probe_validations.load(Ordering::SeqCst), queries);
    assert_eq!(values(original.rows()), expected("A", 10));
    let suspended_position = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    rules.records.lock().unwrap().clear();
    let completed = reconstruction::complete(&world, suspended, 50);
    let failure = match world.restore_generated_output(completed, &scope) {
        Err(failure) => failure,
        Ok(_) => panic!("B's third actual enclosure must refuse"),
    };
    let WorthQueryGeneratedOutputRestorationFailure::Rejected { suspended, cause } = failure else {
        panic!("the real endpoint candidate reaches its rule")
    };
    assert!(
        matches!(cause,WorthQueryGeneratedOutputRestorationFailureCause::PreparationInvariantRejected { identifier,.. } if identifier == "AtMostTwoProbeEnclosures")
    );
    {
        let rows = rules.records.lock().unwrap();
        let b = rows.iter().find(|row| row.scope == b_scope).unwrap();
        assert_eq!(values(b.source_after.as_ref().unwrap()), expected("B", 50));
        assert_eq!(b.after.len(), 3);
        assert!(b.before.is_none());
        assert!(b.source_before.is_none());
    }
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &suspended_position
    );
    assert_suspended(request.query(all("A")).execute());
    rules.records.lock().unwrap().clear();
    let builds = provider.builds.load(Ordering::SeqCst);
    let updates = provider.updates.load(Ordering::SeqCst);
    let completed = reconstruction::complete(&world, suspended, 10);
    world
        .restore_generated_output(completed, &scope)
        .unwrap_or_else(|failure| panic!("exact geometry restores: {:?}", failure.cause()));
    assert_eq!(provider.builds.load(Ordering::SeqCst), builds);
    assert_eq!(provider.updates.load(Ordering::SeqCst), updates);
    assert_rules(&rules, scopes, false, 10);
    for directory in ["A", "B"] {
        assert_eq!(
            values(request.query(all(directory)).execute().unwrap().rows()),
            expected(directory, 10)
        );
    }
    assert_eq!(
        provider.builds.load(Ordering::SeqCst),
        builds,
        "fresh public reads use the same judged canonical backing"
    );
    assert_eq!(values(original.rows()), expected("A", 10));
    a.close();
}
