//! Authored independent row rosters; graph identities come only from publication.
use super::*;
use crate::application_invariant_acceptance::managed_index::maintained::nested::edit::Edit;
use crate::application_invariant_acceptance::managed_index::prefix_observation;
pub(super) fn set(names: &[&str]) -> BTreeSet<String> {
    names.iter().map(|s| (*s).to_owned()).collect()
}
pub(super) fn names(rows: &[query::ItemRow]) -> BTreeSet<String> {
    rows.iter().map(|r| r.identity.clone()).collect()
}
pub(super) fn find(event: &str, prefix: &str) -> query::FindPrefix {
    query::FindPrefix {
        directory: event.into(),
        prefix: prefix.into(),
    }
}
pub(super) fn publish(world: &host::EventHost, key: &mut u64, input: Edit) {
    let scope = authentication::request_scope();
    let principal = authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ),
    )
    .unwrap();
    *key += 1;
    let result = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world())
        .advanced()
        .mutate(input)
        .without_source()
        .idempotency(key)
        .execute_in_program(world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(result.result().is_some(), "{result:?}");
}
pub(super) fn edit_history(world: &host::EventHost, key: &mut u64, input: history::Edit) {
    let scope = authentication::request_scope();
    let principal = authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ),
    )
    .unwrap();
    *key += 1;
    let result = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world())
        .advanced()
        .mutate(input)
        .without_source()
        .idempotency(key)
        .execute_in_program(world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(result.result().is_some(), "{result:?}");
}
pub(super) fn setup() -> (
    host::EventHost,
    Arc<prefix_observation::Calls>,
    Arc<rule::Calls>,
    u64,
) {
    let provider = Arc::new(prefix_observation::Calls::default());
    let records = Arc::new(rule::Calls::default());
    let world = host::install_events(Arc::clone(&provider), Arc::clone(&records));
    let mut key = 3000;
    for event in ["A", "B", "C", "D", "U"] {
        publish(
            &world,
            &mut key,
            Edit::Directory {
                identity: event.into(),
            },
        );
    }
    for (event, item) in [("A", "x"), ("C", "y"), ("U", "z")] {
        publish(
            &world,
            &mut key,
            Edit::Item {
                directory: event.into(),
                identity: item.into(),
                name: format!("match-{item}"),
                root: true,
                incarnation: 1,
            },
        );
    }
    for (event, previous) in [("B", "A"), ("C", "B"), ("D", "C")] {
        edit_history(
            &world,
            &mut key,
            history::Edit::Previous {
                event: event.into(),
                previous: Some(previous.into()),
                edition: 1,
            },
        );
    }
    edit_history(
        &world,
        &mut key,
        history::Edit::Retirement {
            event: "B".into(),
            item: "x".into(),
            retired: true,
            edition: 1,
        },
    );
    edit_history(
        &world,
        &mut key,
        history::Edit::Introduction {
            event: "C".into(),
            item: "x".into(),
            introduced: true,
            edition: 1,
        },
    );
    (world, provider, records, key)
}
