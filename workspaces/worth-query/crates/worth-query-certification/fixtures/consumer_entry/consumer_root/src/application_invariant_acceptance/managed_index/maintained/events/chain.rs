//! Authored chain histories keep the roster width fixed while scopes grow.
use super::super::super::super::authentication;
use super::super::host;
use super::court::fixture::*;
use super::{history, prefix, query};
use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use worth_query_host::facade::admission::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal;
use worth_query_host::facade::{
    application_entry::*, primary_graph::*, runtime::ExecutionAllocationPolicy,
};

fn append(
    event: &str,
    previous: &str,
    introduced: Option<(String, String)>,
    retired: Option<String>,
) -> history::Edit {
    history::Edit::Append {
        rename: None,
        event: event.into(),
        previous: previous.into(),
        introduced,
        retired,
    }
}
fn principal(
    world: &host::EventHost,
    scope: &worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope,
) -> WorthQueryAuthenticatedExternalPrincipal<super::schema::Catalog> {
    authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            scope,
        ),
    )
    .unwrap()
}

pub(in super::super) fn growth(count: usize) -> u64 {
    let (world, calls, _, mut key) = setup_growth();
    let mut active = VecDeque::from(["x".to_owned()]);
    for i in 0..15 {
        let id = format!("seed-{i:04}");
        publish(
            &world,
            &mut key,
            crate::application_invariant_acceptance::managed_index::maintained::nested::edit::Edit::Item {
                directory: "A".into(),
                identity: id.clone(),
                name: format!("match-{id}"),
                root: true,
                incarnation: 1,
            },
        );
        active.push_back(id);
    }
    let builds = calls.builds.load(Ordering::SeqCst);
    let mut previous = "A".to_owned();
    let mut measured = 0;
    for i in 0..count {
        let event = format!("scope-{i:04}");
        let row = format!("row-{i:04}");
        let retired = active.pop_front().unwrap();
        let ((), work) = super::super::court::prefix_growth::invoice(|| {
            edit_history(
                &world,
                &mut key,
                append(
                    &event,
                    &previous,
                    Some((row.clone(), format!("match-{row}"))),
                    Some(retired),
                ),
            )
        });
        measured = work;
        active.push_back(row);
        previous = event;
    }
    assert_eq!(
        calls.builds.load(Ordering::SeqCst),
        builds,
        "every new scope must inherit; no prefix may rebuild"
    );
    let scope = authentication::request_scope();
    let principal = principal(&world, &scope);
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let expected = active.into_iter().collect();
    assert_eq!(
        names(request.query(find(&previous, "")).execute().unwrap().rows()),
        expected
    );
    assert_eq!(
        names(
            request
                .query(query::ReadSource {
                    directory: previous
                })
                .execute()
                .unwrap()
                .rows()
        ),
        expected
    );
    measured
}

pub(in super::super) fn forks() {
    let (world, calls, _, mut key) = setup();
    edit_history(
        &world,
        &mut key,
        append(
            "E",
            "A",
            Some(("e".into(), "match-e".into())),
            Some("x".into()),
        ),
    );
    let scope = authentication::request_scope();
    let principal = principal(&world, &scope);
    let builds = calls.builds.load(Ordering::SeqCst);
    let reused = world
        .runtime()
        .with_host_advancement(|phase| {
            world
                .runtime()
                .branches()
                .fork(world.current_world())
                .components(|c| c.reuse_exact_relational_basis().reuse_exact_signal_basis())
                .create(&phase)
                .unwrap()
        })
        .unwrap();
    let forked = world
        .runtime()
        .with_host_advancement(|phase| {
            world
                .runtime()
                .branches()
                .fork(world.current_world())
                .components(|c| c.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .unwrap()
        })
        .unwrap();
    for branch in [world.current_world(), reused, forked] {
        let request = world
            .runtime()
            .request(&principal, &scope)
            .on_branch(branch);
        for (event, expected) in [("A", set(&["x"])), ("E", set(&["e"]))] {
            request
                .query(find(event, ""))
                .open_index::<prefix::PrefixIndex>()
                .unwrap();
            assert_eq!(
                names(request.query(find(event, "")).execute().unwrap().rows()),
                expected
            );
        }
    }
    assert_eq!(calls.builds.load(Ordering::SeqCst), builds);
    edit_history(
        &world,
        &mut key,
        append(
            "F",
            "E",
            Some(("f".into(), "match-f".into())),
            Some("e".into()),
        ),
    );
    for branch in [reused, forked] {
        let request = world
            .runtime()
            .request(&principal, &scope)
            .on_branch(branch);
        assert_eq!(
            names(request.query(find("A", "")).execute().unwrap().rows()),
            set(&["x"])
        );
        assert_eq!(
            names(request.query(find("E", "")).execute().unwrap().rows()),
            set(&["e"])
        );
    }
}

pub(in super::super) fn eviction() {
    let (world, calls, _, mut key) = setup();
    let scope = authentication::request_scope();
    let principal = principal(&world, &scope);
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let parent = request
        .query(find("A", ""))
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    request.query(find("A", "")).evict_index(&parent).unwrap();
    let builds = calls.builds.load(Ordering::SeqCst);
    edit_history(
        &world,
        &mut key,
        append("E", "A", Some(("e".into(), "match-e".into())), None),
    );
    assert_eq!(
        calls.builds.load(Ordering::SeqCst),
        builds + 1,
        "only the successor may cold-build"
    );
    assert_eq!(
        names(request.query(find("E", "")).execute().unwrap().rows()),
        set(&["x", "e"])
    );
    let successor = request
        .query(find("E", ""))
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    let before = request.query(find("E", "")).execute().unwrap();
    publish(
        &world,
        &mut key,
        crate::application_invariant_acceptance::managed_index::maintained::nested::edit::Edit::Name {
            item: "x".into(),
            value: "changed-prefix".into(),
        },
    );
    let current = request.query(find("E", "")).execute().unwrap();
    assert!(current
        .rows()
        .iter()
        .any(|row| row.identity == "x" && row.name == "changed-prefix"));
    assert!(before
        .rows()
        .iter()
        .any(|row| row.identity == "x" && row.name == "match-x"));
    request
        .query(find("E", ""))
        .evict_index(&successor)
        .unwrap();
    request
        .query(find("E", ""))
        .rebuild_index(&successor)
        .unwrap();
    assert_eq!(
        current.rows(),
        request.query(find("E", "")).execute().unwrap().rows()
    );
}

pub(in super::super) fn cancellation() {
    use worth_query_host::facade::admission::authenticated_principal::*;
    let (world, calls, _, mut key) = setup();
    let scope = authentication::request_scope();
    let principal = principal(&world, &scope);
    let cancelled = WorthQueryCancellationSource::new();
    let stopped =
        WorthQueryRequestScope::new(None, cancelled.token(), WorthQueryAdmissionClock::system());
    let request = world
        .runtime()
        .request(&principal, &stopped)
        .on_branch(world.current_world());
    let head = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let reached = Arc::new(AtomicBool::new(false));
    let observed = Arc::clone(&reached);
    *calls.after_first_node.lock().unwrap() = Some(Box::new(move || {
        observed.store(true, Ordering::SeqCst);
        cancelled.cancel();
    }));
    key += 1;
    let result = request
        .advanced()
        .mutate(append(
            "E",
            "A",
            Some(("e".into(), "match-e".into())),
            Some("x".into()),
        ))
        .without_source()
        .idempotency(&key)
        .execute_in_program(&world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        result.result().is_none(),
        "cancelled birth must refuse: {result:?}"
    );
    assert!(reached.load(Ordering::SeqCst));
    let fresh = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    assert_eq!(
        fresh.retain_current_read().unwrap().selected_commit(),
        &head
    );
    assert_eq!(
        names(fresh.query(find("A", "")).execute().unwrap().rows()),
        set(&["x"])
    );
    edit_history(
        &world,
        &mut key,
        append(
            "E",
            "A",
            Some(("e".into(), "match-e".into())),
            Some("x".into()),
        ),
    );
    assert_eq!(
        names(fresh.query(find("E", "")).execute().unwrap().rows()),
        set(&["e"])
    );
}

pub(in super::super) fn foreign() {
    let (world, calls, _, mut key) = setup();
    edit_history(&world, &mut key, append("E", "A", None, None));
    let scope = authentication::request_scope();
    let principal = principal(&world, &scope);
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let unrelated = request
        .query(find("U", ""))
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    request
        .query(find("U", ""))
        .rebuild_index(&unrelated)
        .unwrap();
    calls.query_read_first.store(true, Ordering::SeqCst);
    let denial = request
        .query(find("E", ""))
        .execute()
        .err()
        .expect("a foreign Source slot must refuse");
    let WorthQueryApplicationRequestQueryDenial::ManagedIndex(denial) = denial else {
        panic!("{denial:?}")
    };
    assert!(
        matches!(
            denial.cause(),
            WorthQueryManagedIndexDenialCause::Work(WorthQueryManagedIndexWorkDenial::Controlled(
                WorthQueryManagedIndexStop::ForeignSourceSlot
                    | WorthQueryManagedIndexStop::ForeignSummarySource
            ))
        ),
        "{denial:?}"
    );
    calls.query_read_first.store(false, Ordering::SeqCst);
    assert_eq!(
        names(request.query(find("E", "")).execute().unwrap().rows()),
        set(&["x"])
    );
}

pub(in super::super) mod evidence;
pub(in super::super) mod foreign_node;

pub(in super::super) mod retained_queries;
