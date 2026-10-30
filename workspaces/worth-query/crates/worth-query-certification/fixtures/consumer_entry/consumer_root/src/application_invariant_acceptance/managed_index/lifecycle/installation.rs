//! Prefix-specific installed/runtime identity and actual executable version twins.
use super::super::boxes;
use super::*;
use worth_query_host::facade::application_installation::WorthQueryInMemoryApplicationDenial;

#[test]
fn registered_prefix_rejects_foreign_runtime_and_incompatible_executable_before_callbacks() {
    run(installation_twins);
}
fn installation_twins() {
    let calls = Arc::new(prefix::Calls::default());
    let world = host::install(Arc::clone(&calls));
    let scope = authentication::request_scope();
    let principal = authenticate(&world, &scope);
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let index = request
        .query(find("al"))
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    request.query(find("al")).rebuild_index(&index).unwrap();
    let source = request
        .query(query::ReadSource {
            directory: "catalog".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(
        request.query(find("al")).execute().unwrap().rows(),
        journey::expected(source.rows(), "al")
    );
    let before = calls.totals();
    let other_calls = Arc::new(prefix::Calls::default());
    let other = host::install(Arc::clone(&other_calls));
    let other_principal = authenticate(&other, &scope);
    let foreign = other
        .runtime()
        .request(&other_principal, &scope)
        .on_branch(other.current_world());
    refusal(
        foreign.query(find("al")).rebuild_index(&index).unwrap_err(),
        WorthQueryManagedIndexRefusal::ForeignAssociation,
    );
    not_ready(
        foreign
            .query(find("al"))
            .execute()
            .err()
            .expect("managed read must refuse"),
    );
    assert_eq!(other_calls.totals(), (0, 0, 0));
    assert_eq!(calls.totals(), before);
    assert_eq!(index.lifecycle(), WorthQueryManagedIndexLifecycle::Ready);

    let version_calls = Arc::new(prefix::Calls::default());
    let box_calls = Arc::new(boxes::Calls::default());
    let error =
        host::try_install_prefix_version(Arc::clone(&version_calls), Arc::clone(&box_calls))
            .err()
            .expect("actual prefix executable v2 disagrees with installed v1 binding");
    assert!(
        matches!(error, WorthQueryInMemoryApplicationDenial::Contributions(ref cause)
        if cause.kind() == WorthQueryPrimaryGraphInstallationDenialKind::ManagedIndexProviderMeaningMismatch),
        "actual typed installation mismatch: {error:?}"
    );
    assert_eq!(version_calls.totals(), (0, 0, 0));
    assert_eq!(
        box_calls.totals(),
        (0, 0, 0),
        "installation stopped before any provider callback"
    );
    assert_eq!(calls.totals(), before);
    index.close();
}
