//! Real registrations grow beside a fixed public query and fixed World/Native data.
use super::*;
use crate::application_invariant_acceptance::managed_index::prefix_alias_input;
use crate::application_invariant_acceptance::managed_index::prefix_observation;
use worth_execution::{
    CancellationSource, ExecutionWorkCeiling, SerialMemoryBudget, SerialRequest,
};

fn invoice<T>(operation: impl FnOnce() -> T) -> (T, u64) {
    let original = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let (result, report) = ExecutionWorkCeiling::new(None)
        .run_with_checkpoint(None, Some(&original), |_| operation())
        .unwrap();
    (result, report.charged_work())
}

#[test]
fn retained_registrations_have_indexed_query_and_repeated_open_work() {
    run(indexed_lookup);
}
fn indexed_lookup() {
    let calls = Arc::new(prefix_observation::Calls::default());
    let world = host::install(Arc::clone(&calls));
    // Build the entire finite World topology before measuring anything. Growing
    // registrations below changes neither scope-resolution data nor branch lookup.
    let mut branches = vec![world.current_world()];
    for _ in 0..6 {
        branches.push(
            world
                .runtime()
                .with_host_advancement(|phase| {
                    world
                        .runtime()
                        .branches()
                        .fork(world.current_world())
                        .components(|components| {
                            components.fork_relational().reuse_exact_signal_basis()
                        })
                        .create(&phase)
                        .unwrap()
                })
                .unwrap(),
        );
    }
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
    let expected = journey::expected(source.rows(), "al");
    for _ in 0..3 {
        assert_eq!(
            invoice(|| request.query(find("al")).execute().unwrap())
                .0
                .rows(),
            expected
        );
    }
    let locations = branches
        .iter()
        .copied()
        .flat_map(|branch| {
            [
                "catalog",
                "empty",
                "boxes",
                "empty-boxes",
                "invalid-boxes",
                "other-boxes",
            ]
            .into_iter()
            .map(move |directory| (branch, directory))
        })
        .skip(1)
        .collect::<Vec<_>>();
    let mut retained = Vec::new();
    let mut queries = Vec::new();
    let mut opens = Vec::new();
    for count in [8, 16, 32] {
        while retained.len() + 1 < count {
            let (branch, directory) = locations[retained.len()];
            retained.push(
                world
                    .runtime()
                    .request(&principal, &scope)
                    .on_branch(branch)
                    .query(query::FindPrefix {
                        directory: directory.into(),
                        prefix: "".into(),
                    })
                    .open_index::<prefix::PrefixIndex>()
                    .unwrap(),
            );
        }
        let (result, work) = invoice(|| request.query(find("al")).execute().unwrap());
        assert_eq!(result.rows(), expected);
        queries.push(work);
        let before = calls.totals();
        let (sibling, work) = invoice(|| {
            request
                .query(find("another probe"))
                .open_index::<prefix::PrefixIndex>()
                .unwrap()
        });
        assert_eq!(sibling.lifecycle(), WorthQueryManagedIndexLifecycle::Ready);
        assert_eq!(
            calls.totals(),
            before,
            "repeated open does not rebuild or select"
        );
        opens.push(work);
        drop(sibling);
    }
    eprintln!("fixed-data SDK invoices: query={queries:?}, repeated-open={opens:?}");
    for (name, costs) in [("public query", queries), ("repeated open", opens)] {
        let first = costs[1]
            .checked_sub(costs[0])
            .expect("more routing depth never loses charged work");
        let second = costs[2].checked_sub(costs[1]).unwrap();
        assert!(
            first > 0,
            "the original SDK invoice includes indexed routing"
        );
        assert!(second * 2 <= first * 3,
            "{name}: fixed-data doubling increments must remain indexed with 50% headroom, not double a population scan: {costs:?}");
    }
    let mut aliases = Vec::new();
    for &(branch, directory) in locations.iter().take(32) {
        aliases.push(
            world
                .runtime()
                .request(&principal, &scope)
                .on_branch(branch)
                .query(alias(directory))
                .open_index::<prefix::PrefixIndex>()
                .unwrap(),
        );
    }
    let (branch, directory) = locations[retained.len()];
    let extra = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(branch);
    refusal(
        extra
            .query(query::FindPrefix {
                directory: directory.into(),
                prefix: "".into(),
            })
            .open_index::<prefix::PrefixIndex>()
            .err()
            .expect("shared registration capacity refuses the 65th owner"),
        WorthQueryManagedIndexRefusal::RegistrationCapacityExceeded,
    );
    assert_eq!(
        request.query(find("al")).execute().unwrap().rows(),
        expected,
        "capacity refusal preserves the genuine warm receiver"
    );
    let sibling = request
        .query(find("al"))
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    assert_eq!(
        sibling.lifecycle(),
        WorthQueryManagedIndexLifecycle::Ready,
        "a full registry still reuses its exact existing association"
    );
    aliases.pop().unwrap().close();
    let admitted = extra
        .query(query::FindPrefix {
            directory: directory.into(),
            prefix: "".into(),
        })
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    admitted.close();
    for alias in aliases {
        alias.close();
    }
    index.close();
    for handle in retained {
        handle.close();
    }
    // A stale sibling's issued close id cannot unregister its reopened successor.
    let reopened = request
        .query(find("al"))
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    index.close();
    assert_eq!(
        reopened.lifecycle(),
        WorthQueryManagedIndexLifecycle::Created
    );
    request.query(find("al")).rebuild_index(&reopened).unwrap();
    assert_eq!(
        request.query(find("al")).execute().unwrap().rows(),
        expected
    );
    reopened.close();
}

fn alias(directory: &str) -> prefix_alias_input::FindPrefixAlias {
    prefix_alias_input::FindPrefixAlias {
        directory: directory.into(),
        prefix: "al".into(),
    }
}

#[test]
fn distinct_parent_bindings_to_one_query_keep_independent_registrations() {
    run(parent_binding_twins);
}
fn parent_binding_twins() {
    let calls = Arc::new(prefix_observation::Calls::default());
    let world = host::install(Arc::clone(&calls));
    let scope = authentication::request_scope();
    let principal = authenticate(&world, &scope);
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let source = request
        .query(query::ReadSource {
            directory: "catalog".into(),
        })
        .execute()
        .unwrap();
    let expected = journey::expected(source.rows(), "al");
    let original = request
        .query(find("al"))
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    request.query(find("al")).rebuild_index(&original).unwrap();
    let second = request
        .query(alias("catalog"))
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    assert_eq!(second.lifecycle(), WorthQueryManagedIndexLifecycle::Created,
        "matching Query obligation, scope and Index do not reuse a different nominal Parent binding");
    not_ready(
        request
            .query(alias("catalog"))
            .execute()
            .err()
            .expect("the independent Parent is cold"),
    );
    request
        .query(alias("catalog"))
        .rebuild_index(&second)
        .unwrap();
    assert_eq!(
        request.query(alias("catalog")).execute().unwrap().rows(),
        expected
    );
    original.close();
    assert_eq!(second.lifecycle(), WorthQueryManagedIndexLifecycle::Ready);
    assert_eq!(
        request.query(alias("catalog")).execute().unwrap().rows(),
        expected
    );
    not_ready(
        request
            .query(find("al"))
            .execute()
            .err()
            .expect("only the original registration closed"),
    );
    second.close();
}
