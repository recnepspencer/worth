//! Native nested Sources, ordinary Parents and proposed rows face an authored graph oracle.
use super::super::super::super::authentication;
use super::{authoring::Edit, host, mutation, prefix, query, rule};
use std::sync::Arc;
use worth_query_host::facade::{
    application_entry::*, primary_graph::*, runtime::ExecutionAllocationPolicy,
};
mod fixture;
mod model;
use fixture::{find, link, publish, setup};
use model::{exact, values, Model};
#[test]
fn empty_nested_source_then_convergent_paths_match_ordinary_and_proposed_reads() {
    let (world, calls, records, mut model, mut key) = setup();
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
    let initial = request.query(find("A", "match")).execute().unwrap();
    assert_eq!(values(initial.rows()), model.expected("A", "match", true));
    assert!(
        initial.rows()[0].children.is_empty(),
        "the declared deep profile initially has no child occurrence"
    );
    for (from, to) in [
        ("x", "a"),
        ("x", "b"),
        ("a", "z"),
        ("b", "z"),
        ("z", "c"),
        ("q", "x"),
        ("q", "p"),
    ] {
        link(&world, &mut model, &mut key, from, to, 1);
    }
    for directory in ["A", "B", "D"] {
        let source = request
            .query(query::ReadSource {
                directory: directory.into(),
            })
            .execute()
            .unwrap();
        let parent = request.query(find(directory, "")).execute().unwrap();
        assert_eq!(values(source.rows()), model.expected(directory, "", false));
        assert_eq!(values(parent.rows()), model.expected(directory, "", true));
        assert_eq!(
            exact(source.rows()),
            exact(parent.rows()),
            "actual native coordinates and child order survive the public handoff"
        );
    }
    records.records.lock().unwrap().clear();
    let before = model.clone();
    let cold = request
        .query(find("A", "match"))
        .open_index::<prefix::PrefixIndex>()
        .unwrap();
    let builds = calls.builds.load(std::sync::atomic::Ordering::SeqCst);
    publish(
        &world,
        &mut model,
        &mut key,
        Edit::Name {
            item: "c".into(),
            value: "changed-tail".into(),
        },
    );
    let observed = records.records.lock().unwrap();
    assert_eq!(
        observed.len(),
        2,
        "only the two actual shared-parent scopes are affected"
    );
    let scopes: std::collections::BTreeSet<_> =
        observed.iter().map(|record| record.scope).collect();
    assert_eq!(
        scopes,
        [model.scope("A"), model.scope("B")].into_iter().collect()
    );
    for record in observed.iter() {
        let directory = model.directory(record.scope);
        assert_eq!(
            values(record.before.as_ref().expect("both scopes already exist")),
            before.expected(directory, "", false)
        );
        assert_eq!(values(&record.after), model.expected(directory, "", false));
        let ordinary = request
            .query(query::ReadSource {
                directory: directory.into(),
            })
            .execute()
            .unwrap();
        assert_eq!(
            exact(&record.after),
            exact(ordinary.rows()),
            "actual proposed Native tree equals the selected committed tree"
        );
    }
    assert_eq!(cold.lifecycle(), WorthQueryManagedIndexLifecycle::Ready);
    assert_eq!(
        calls.builds.load(std::sync::atomic::Ordering::SeqCst),
        builds,
        "descendant updates maintain warm roots"
    );
}

mod controls;
mod influence;
mod retirement;

#[test]
fn reverse_root_branch_and_its_nested_siblings_use_actual_parent_routes_and_inverse_scope_routing()
{
    let (world, _, records, mut model, mut key) = setup();
    link(&world, &mut model, &mut key, "q", "x", 1);
    link(&world, &mut model, &mut key, "q", "p", 1);
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
    let before = request.query(find("A", "match")).execute().unwrap();
    assert_eq!(values(before.rows()), model.expected("A", "match", true));
    assert_eq!(before.rows()[0].parents.len(), 1);
    assert_eq!(before.rows()[0].parents[0].children.len(), 2);
    records.records.lock().unwrap().clear();
    let old = model.clone();
    publish(
        &world,
        &mut model,
        &mut key,
        Edit::Name {
            item: "p".into(),
            value: "changed-peer".into(),
        },
    );
    let observed = records.records.lock().unwrap();
    let scopes: std::collections::BTreeSet<_> =
        observed.iter().map(|record| record.scope).collect();
    assert_eq!(
        scopes,
        [model.scope("A"), model.scope("B")].into_iter().collect()
    );
    for record in observed.iter() {
        let directory = model.directory(record.scope);
        assert_eq!(
            values(record.before.as_ref().unwrap()),
            old.expected(directory, "", false)
        );
        assert_eq!(values(&record.after), model.expected(directory, "", false));
        let parent = request.query(find(directory, "")).execute().unwrap();
        assert_eq!(exact(&record.after), exact(parent.rows()));
    }
}
