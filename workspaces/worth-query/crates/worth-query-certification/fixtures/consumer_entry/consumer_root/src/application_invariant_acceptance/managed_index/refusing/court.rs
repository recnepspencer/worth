//! Refuse remains fixed through construction, warm use, eviction and public refill.
use super::super::super::authentication;
use super::super::lifecycle;
use super::{host, mutation, prefix, query, rule};
use crate::application_invariant_acceptance::managed_index::directory_edit;
use crate::application_invariant_acceptance::managed_index::prefix_observation;
use stats_alloc::Region;
use std::sync::{atomic::Ordering, Arc};
use worth_query_host::facade::{
    application_entry::*, application_installation::*, primary_graph::*,
    runtime::ExecutionAllocationPolicy,
};
#[test]
fn explicit_initialization_is_confined_to_one_unpublished_seed_attempt() {
    lifecycle::isolated_allocation_case(
        run,
        concat!(
            module_path!(),
            "::explicit_initialization_is_confined_to_one_unpublished_seed_attempt"
        ),
    );
}
fn run() {
    journey();
    let region = Region::new(lifecycle::ALLOCATOR);
    journey();
    let balance = region.change();
    assert_eq!(
        balance.bytes_allocated, balance.bytes_deallocated,
        "failed and successful initialization, proof/session/basis backing and all roots release"
    );
}
fn find(directory: &str) -> query::FindPrefix {
    query::FindPrefix {
        directory: directory.into(),
        prefix: "".into(),
    }
}
fn values(rows: &[query::ItemRow]) -> Vec<(String, String)> {
    let mut rows: Vec<_> = rows
        .iter()
        .map(|row| (row.identity.clone(), row.name.clone()))
        .collect();
    rows.sort();
    rows
}
fn expected(directory: &str, shared: &str) -> Vec<(String, String)> {
    // Authored roster/value oracle; no provider node or another managed query.
    let mut rows = if directory == "A" {
        vec![("a-only", "amber")]
    } else {
        vec![("b-only", "beta")]
    };
    rows.push(("shared", shared));
    rows.sort();
    rows.into_iter()
        .map(|(id, name)| (id.into(), name.into()))
        .collect()
}
fn configured(calls: &Arc<prefix_observation::Calls>) -> (host::Configuration, Arc<rule::Calls>) {
    let rules = Arc::new(rule::Calls::new(Arc::clone(calls)));
    (
        host::Configuration {
            provider: Arc::clone(calls),
            rules: Arc::clone(&rules),
        },
        rules,
    )
}
fn publication_denial(
    error: WorthQueryInMemoryApplicationDenial,
) -> WorthQueryPrimaryGraphInstallationDenial {
    match error {
        WorthQueryInMemoryApplicationDenial::Publication(denial) => denial,
        other => panic!("expected the actual unpublished graph denial: {other:?}"),
    }
}
fn initial_state_denial(
    error: WorthQueryInMemoryApplicationDenial,
) -> WorthQueryPrimaryGraphInstallationDenial {
    match error {
        WorthQueryInMemoryApplicationDenial::InitialState(denial) => denial,
        other => panic!("expected the actual initial-state API denial: {other:?}"),
    }
}
fn require_cold_refusal(
    result: WorthQueryApplicationMutationOutcome<mutation::Denial, mutation::Renamed>,
) {
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = result
    else {
        panic!("Refuse must stop preparation: {result:?}")
    };
    assert!(
        denial
            .invariant_execution_failure()
            .unwrap()
            .managed_source_denial()
            .unwrap()
            .source_unavailable(),
        "{denial:?}"
    );
}
fn journey() {
    let calls = Arc::new(prefix_observation::Calls::default());
    let (configuration, rules) = configured(&calls);
    let error = host::install(configuration, host::omit, ["alpha", "amber", "beta"])
        .err()
        .expect("Refuse does not silently construct new scopes");
    let denial = publication_denial(error);
    assert_eq!(
        denial.kind(),
        WorthQueryPrimaryGraphInstallationDenialKind::RelationalCommitRejected
    );
    assert!(denial.managed_source_denial().unwrap().source_unavailable());
    assert_eq!(calls.builds.load(Ordering::SeqCst), 0);
    assert!(rules.records.lock().unwrap().is_empty());
    let (configuration, _) = configured(&calls);
    let error = host::install(
        configuration,
        host::empty_attempt,
        ["alpha", "amber", "beta"],
    )
    .err()
    .expect("empty attempted batch consumes selection and refuses");
    assert_eq!(
        initial_state_denial(error).kind(),
        WorthQueryPrimaryGraphInstallationDenialKind::EmptyBootstrap
    );
    assert_eq!(calls.builds.load(Ordering::SeqCst), 0);
    let (configuration, _) = configured(&calls);
    let error = host::install(
        configuration,
        host::foreign_binding,
        ["alpha", "amber", "beta"],
    )
    .err()
    .expect("equal names/meaning cannot select another installed binding");
    assert_eq!(
        initial_state_denial(error).kind(),
        WorthQueryPrimaryGraphInstallationDenialKind::ManagedIndexProviderMeaningMismatch
    );
    assert_eq!(calls.builds.load(Ordering::SeqCst), 0);

    let (configuration, _) = configured(&calls);
    let error = host::install(
        configuration,
        host::unrelated_attempt,
        ["alpha", "amber", "beta"],
    )
    .err()
    .expect("unused initialization is consumed by its nonempty unrelated seed attempt");
    assert!(publication_denial(error)
        .managed_source_denial()
        .unwrap()
        .source_unavailable());
    assert_eq!(calls.builds.load(Ordering::SeqCst), 0);

    let (configuration, rejected_rules) = configured(&calls);
    let error = host::install(configuration, host::initialize, ["taken", "taken", "beta"])
        .err()
        .expect("normal rules must judge explicitly initialized seeds");
    let denial = publication_denial(error);
    assert!(
        matches!(
            denial.custom_invariant_denial(),
            Some(WorthQueryCustomInvariantDenial::Violation { .. })
        ),
        "{denial:?}"
    );
    assert!(rejected_rules
        .records
        .lock()
        .unwrap()
        .iter()
        .any(|record| record
            .source
            .after
            .iter()
            .filter(|row| row.name == "taken")
            .count()
            == 2));

    let (configuration, rules) = configured(&calls);
    let world = host::install(configuration, host::initialize, ["alpha", "amber", "beta"]).unwrap();
    assert_eq!(
        rules.records.lock().unwrap().len(),
        3,
        "every seeded Source scope is judged before visibility"
    );
    assert!(rules
        .records
        .lock()
        .unwrap()
        .iter()
        .all(|record| record.source.before.is_none()));
    let scope = authentication::request_scope();
    let principal = authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ),
    )
    .unwrap();
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let index = request
        .query(find("B"))
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    let original = request.query(find("A")).execute().unwrap();
    let builds = calls.builds.load(Ordering::SeqCst);
    assert_eq!(values(original.rows()), expected("A", "alpha"));
    let edit = |replacement: &str| mutation::Rename {
        directory: "A".into(),
        prefix: "".into(),
        item: "shared".into(),
        replacement: replacement.into(),
    };
    let warm = request
        .advanced()
        .mutate(edit("alpine"))
        .expect_result_set(original.result_set_observation().unwrap().clone())
        .idempotency(&0x919480u64)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(warm, WorthQueryApplicationMutationOutcome::Committed { .. }),
        "{warm:?}"
    );
    assert_eq!(
        calls.builds.load(Ordering::SeqCst),
        builds,
        "both shared scopes remain warm under Refuse"
    );
    assert_eq!(
        values(request.query(find("B")).execute().unwrap().rows()),
        expected("B", "alpine")
    );
    let current_a = request.query(find("A")).execute().unwrap();
    request.query(find("B")).evict_index(&index).unwrap();
    rules.records.lock().unwrap().clear();
    let head = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let refused = request
        .advanced()
        .mutate(edit("alto"))
        .expect_result_set(current_a.result_set_observation().unwrap().clone())
        .idempotency(&0x919481u64)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    require_cold_refusal(refused);
    assert!(
        rules.records.lock().unwrap().is_empty(),
        "all Source preparation precedes every rule"
    );
    assert_eq!(
        calls.builds.load(Ordering::SeqCst),
        builds,
        "Refuse never borrows vanished initialization permission"
    );
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &head
    );
    assert_eq!(
        values(request.query(find("A")).execute().unwrap().rows()),
        expected("A", "alpine")
    );
    let cold = request
        .query(find("B"))
        .execute()
        .err()
        .expect("cold source must refuse");
    let WorthQueryApplicationRequestQueryDenial::ManagedIndex(denial) = cold else {
        panic!("{cold:?}")
    };
    assert!(matches!(
        denial.cause(),
        WorthQueryManagedIndexDenialCause::Refused(WorthQueryManagedIndexRefusal::RootUnavailable)
    ));
    drop(denial);
    request.query(find("B")).rebuild_index(&index).unwrap();
    assert_eq!(calls.builds.load(Ordering::SeqCst), builds + 1);
    let retry = request
        .advanced()
        .mutate(edit("alto"))
        .expect_result_set(current_a.result_set_observation().unwrap().clone())
        .idempotency(&0x919481u64)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            retry,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{retry:?}"
    );
    assert_eq!(
        calls.builds.load(Ordering::SeqCst),
        builds + 1,
        "ordinary retry uses the canonical refilled material"
    );
    assert_eq!(
        values(request.query(find("A")).execute().unwrap().rows()),
        expected("A", "alto")
    );
    assert_eq!(
        values(request.query(find("B")).execute().unwrap().rows()),
        expected("B", "alto")
    );
    assert_eq!(values(original.rows()), expected("A", "alpha"));
    let head = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let created = request
        .advanced()
        .mutate(directory_edit::Replace {
            edit: directory_edit::Edit::Create,
        })
        .idempotency(&0x919482u64)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = created
    else {
        panic!("ordinary new scope cannot inherit initialization: {created:?}")
    };
    assert!(denial
        .invariant_execution_failure()
        .unwrap()
        .managed_source_denial()
        .unwrap()
        .source_unavailable());
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &head
    );
    assert_eq!(calls.builds.load(Ordering::SeqCst), builds + 1);
    index.close();
}
