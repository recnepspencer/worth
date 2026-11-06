//! Corrupt only original selection evidence; the fresh request remains lawful.
use super::*;
use crate::application_invariant_acceptance::managed_index::prefix_observation;

#[test]
fn retained_watch_coordinates_refuse_refresh_and_keep_original_custody() {
    lifecycle::run(journey);
}

fn journey() {
    let calls = Arc::new(prefix_observation::Calls::default());
    let world = host::install_watch(calls.clone(), Arc::new(rule::Calls::new(calls)));
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
    let (read, mut watch) = request
        .query(find("A", "later"))
        .watch_index::<prefix::PrefixIndex>()
        .unwrap();
    let (_, mut other_scope) = request
        .query(find("B", "later"))
        .watch_index::<prefix::PrefixIndex>()
        .unwrap();
    let calls = Arc::new(prefix_observation::Calls::default());
    let foreign_world = host::install_watch(calls.clone(), Arc::new(rule::Calls::new(calls)));
    let foreign_principal = authentication::block_on(
        authentication::admit(foreign_world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ),
    )
    .unwrap();
    let (_, mut other_parent) = foreign_world
        .runtime()
        .request(&foreign_principal, &scope)
        .on_branch(foreign_world.current_world())
        .query(find("A", "later"))
        .watch_index::<prefix::PrefixIndex>()
        .unwrap();
    assert!(matches!(
        request
            .query(find("A", "later"))
            .next_watch(&mut watch)
            .unwrap(),
        WorthQueryManagedIndexWatchOutcome::Pending
    ));
    watch.exchange_selection_scope_for_test(&mut other_scope);
    refused(
        request
            .query(find("A", "later"))
            .next_watch(&mut watch)
            .err()
            .expect("foreign original scope"),
    );
    watch.exchange_selection_scope_for_test(&mut other_scope);
    assert!(matches!(
        request
            .query(find("A", "later"))
            .next_watch(&mut watch)
            .unwrap(),
        WorthQueryManagedIndexWatchOutcome::Pending
    ));
    watch.exchange_selection_association_for_test(&mut other_parent);
    refused(
        request
            .query(find("A", "later"))
            .next_watch(&mut watch)
            .err()
            .expect("foreign original association"),
    );
    watch.exchange_selection_association_for_test(&mut other_parent);
    assert!(matches!(
        request
            .query(find("A", "later"))
            .next_watch(&mut watch)
            .unwrap(),
        WorthQueryManagedIndexWatchOutcome::Pending
    ));
    let held = watch.selection_custody_observer_for_test();
    assert_eq!(held(), [true; 4]);
    other_scope.close().unwrap();
    other_parent.close().unwrap();
    drop(read);
    drop(world);
    assert_eq!(
        held(),
        [true; 4],
        "the watch retains original immutable backings after runtime custody ends"
    );
    watch.close().unwrap();
    assert_eq!(
        held(),
        [false; 4],
        "closing the last watch releases every original custody role"
    );
}

fn refused(error: WorthQueryApplicationRequestQueryDenial) {
    let WorthQueryApplicationRequestQueryDenial::ManagedIndex(cause) = error else {
        panic!("{error:?}")
    };
    assert!(
        matches!(
            cause.cause(),
            WorthQueryManagedIndexDenialCause::Refused(
                WorthQueryManagedIndexRefusal::ForeignContext
            )
        ),
        "{cause:?}"
    );
}
