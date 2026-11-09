//! Real indirect endpoint edits cross both maintained Source readers and scopes.
use super::super::super::super::authentication;
use super::super::super::lifecycle;
use super::super::{
    bounds::Coordinates, court::accounted_request, host, prefix, rule as prefix_rule,
    schema::Catalog,
};
use super::{authoring::Edit, provider, query, rule, Configuration};
use stats_alloc::Region;
use std::sync::{atomic::Ordering, Arc, Mutex};
use worth_query_host::facade::{
    admission::authenticated_principal::*, application_entry::*, primary_graph::*,
    runtime::ExecutionAllocationPolicy,
};

mod affinity;
mod cancellation;
mod close;
mod cold;
mod exact_refinement;
mod growth;
mod owned_summary;
mod realized_footprint;
mod rebuild;
mod rewiring;
mod summary;
mod summary_locality;
mod watch;
type Principal = WorthQueryAuthenticatedExternalPrincipal<Catalog>;
fn find(directory: &str, x: i64) -> query::FindBoxes {
    query::FindBoxes {
        directory: directory.into(),
        probe: Coordinates::new(x, 0, x, 0),
    }
}
fn all(directory: &str) -> query::FindBoxes {
    query::FindBoxes {
        directory: directory.into(),
        probe: Coordinates::new(-1, -1, 100, 1),
    }
}
pub(super) fn values(rows: &[query::Row]) -> Vec<(String, Coordinates, Coordinates)> {
    let mut values: Vec<_> = rows
        .iter()
        .map(|row| (row.identity.clone(), row.start, row.end))
        .collect();
    values.sort_by(|a, b| a.0.cmp(&b.0));
    values
}
/// The independent universe comes from the authored coordinates and edge roster.
/// This interpreter uses closed coordinate comparisons, never provider nodes,
/// Box2, Source descriptors, or another managed result as its expected answer.
pub(super) fn expected(
    directory: &str,
    end: i64,
    probe: Coordinates,
) -> Vec<(String, Coordinates, Coordinates)> {
    let point = |x| Coordinates::new(x, 0, x, 0);
    [("e1", 0, end), ("e2", 50, 60), ("e3", 50, 70)]
        .into_iter()
        .filter(|(id, _, _)| directory == "B" || *id == "e1")
        .filter(|(_, a, b)| {
            a.min(b) <= &probe.upper_x
                && &probe.lower_x <= a.max(b)
                && probe.lower_y <= 0
                && 0 <= probe.upper_y
        })
        .map(|(id, a, b)| (id.to_owned(), point(a), point(b)))
        .collect()
}
pub(super) fn commit(
    world: &host::Host,
    principal: &Principal,
    scope: &WorthQueryRequestScope,
    edit: Edit,
    key: u64,
) {
    let result = world
        .runtime()
        .request(principal, scope)
        .on_branch(world.current_world())
        .advanced()
        .mutate(edit)
        .idempotency(&key)
        .execute_in_program(world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            result,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{result:?}"
    );
}
pub(super) fn setup(world: &host::Host, principal: &Principal, scope: &WorthQueryRequestScope) {
    let vertices = [0, 10, 50, 60, 70, 30]
        .into_iter()
        .enumerate()
        .map(|(index, x)| Edit::Vertex {
            directory: "catalog".into(),
            identity: format!("v{index}"),
            x,
            y: 0,
        });
    let edges = [
        ("A", "e1", "v0", "v1"),
        ("B", "e2", "v2", "v3"),
        ("B", "e3", "v2", "v4"),
    ]
    .into_iter()
    .map(|(directory, identity, start, end)| Edit::Edge {
        directory: directory.into(),
        identity: identity.into(),
        start: start.into(),
        end: end.into(),
    });
    for (index, edit) in vertices
        .chain(edges)
        .chain([Edit::Share {
            directory: "B".into(),
            edge: "e1".into(),
        }])
        .enumerate()
    {
        commit(world, principal, scope, edit, 0x919400 + index as u64);
    }
}
#[test]
fn indirect_vertex_motion_prepares_unopened_scope_and_keeps_atomic_geometry_roots() {
    lifecycle::isolated_allocation_case(
        run,
        concat!(
            module_path!(),
            "::indirect_vertex_motion_prepares_unopened_scope_and_keeps_atomic_geometry_roots"
        ),
    );
}
fn run() {
    journey();
    let region = Region::new(lifecycle::ALLOCATOR);
    journey();
    let released = region.change();
    assert_eq!(
        released.bytes_allocated, released.bytes_deallocated,
        "all geometry runtime, roots, snapshots, calls and request owners close"
    );
}
fn journey() {
    let calls = Arc::new(provider::Calls::default());
    let configuration = Configuration::new(Arc::clone(&calls));
    let rules = Arc::clone(&configuration.rules);
    let prefix_calls = Arc::new(prefix::Calls::default());
    let prefix_rules = Arc::new(prefix_rule::Calls::new(Arc::clone(&prefix_calls)));
    let world = host::install_geometry(prefix_calls, prefix_rules, configuration);
    let scope = authentication::request_scope();
    let principal = authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ),
    )
    .unwrap();
    setup(&world, &principal, &scope);
    rules.records.lock().unwrap().clear();
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    let a = request
        .query(all("A"))
        .open_index::<provider::BoxIndex>()
        .unwrap();
    let original = request.query(all("A")).execute().unwrap();
    assert_eq!(values(original.rows()), expected("A", 10, all("A").probe));
    assert!(request
        .query(find("A", 15))
        .execute()
        .unwrap()
        .rows()
        .is_empty());
    let head = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let builds = calls.builds.load(Ordering::SeqCst);
    let updates = calls.updates.load(Ordering::SeqCst);
    let memory = accounted_request();
    let observed_memory = Arc::clone(&memory);
    let reached = Arc::new(Mutex::new(None));
    let observed = Arc::clone(&reached);
    let baseline = memory.charged_memory_bytes();
    *calls.after_node_write.lock().unwrap() = Some(Box::new(move || {
        *observed.lock().unwrap() = Some(observed_memory.charged_memory_bytes())
    }));
    let result = request
        .advanced()
        .mutate(Edit::Move {
            directory: "A".into(),
            vertex: "v1".into(),
            x: 50,
            y: 0,
        })
        .idempotency(&0x919410u64)
        .execute_in_program(&world, ExecutionAllocationPolicy::Execution(&memory))
        .unwrap();
    require_violation(result);
    assert!(
        reached
            .lock()
            .unwrap()
            .is_some_and(|bytes| bytes > baseline),
        "real staged geometry backing used the original payer"
    );
    assert!(
        calls
            .last_decision
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .is_empty(),
        "the ordinary handler read the real committed A Parent"
    );
    {
        let records = rules.records.lock().unwrap();
        assert!((1..=2).contains(&records.len()));
        assert!(
            records
                .iter()
                .all(|record| record.updates_before_callback >= updates + 2),
            "B's root was ready before the FIRST invariant"
        );
        let b = records
            .iter()
            .find(|record| record.after.len() == 3)
            .expect("unopened B actually observes the proposed enclosure violation");
        assert_eq!(
            values(b.before.as_ref().unwrap()),
            expected("B", 10, rule::probe())
        );
        assert_eq!(values(&b.after), expected("B", 50, rule::probe()));
    }
    rules.records.lock().unwrap().clear();
    assert_eq!(
        memory.charged_memory_bytes(),
        baseline,
        "refused geometry staging releases its original grants"
    );
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &head
    );
    assert!(request
        .query(find("A", 15))
        .execute()
        .unwrap()
        .rows()
        .is_empty());
    commit(
        &world,
        &principal,
        &scope,
        Edit::Move {
            directory: "A".into(),
            vertex: "v1".into(),
            x: 20,
            y: 0,
        },
        0x919411,
    );
    assert_eq!(
        values(original.rows()),
        expected("A", 10, all("A").probe),
        "retained output keeps the actual old endpoint values"
    );
    assert_eq!(
        values(request.query(find("A", 15)).execute().unwrap().rows()),
        expected("A", 20, find("A", 15).probe)
    );
    let b = request
        .query(all("B"))
        .open_index::<provider::BoxIndex>()
        .unwrap();
    assert_eq!(b.lifecycle(), WorthQueryManagedIndexLifecycle::Ready);
    assert_eq!(
        values(request.query(find("B", 15)).execute().unwrap().rows()),
        expected("B", 20, find("B", 15).probe)
    );
    let source = request
        .query(query::ReadSource {
            directory: "B".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(values(source.rows()), expected("B", 20, all("B").probe));
    assert_eq!(
        calls.builds.load(Ordering::SeqCst),
        builds,
        "public B attaches the atomically published root without reconstruction"
    );
    drop(source);
    rewiring::run(&world, &principal, &scope, &calls, &rules);
    cancellation::run(&world, &principal, &calls, &rules);
    a.close();
    b.close();
}
fn require_violation<D: std::fmt::Debug, R: std::fmt::Debug>(
    outcome: WorthQueryApplicationMutationOutcome<D, R>,
) {
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = outcome
    else {
        panic!("actual B geometry rule must refuse: {outcome:?}")
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::CustomInvariantDenied
    );
    let Some(WorthQueryCustomInvariantDenial::Violation { identity, .. }) =
        denial.custom_invariant_denial()
    else {
        panic!("Source or setup failure cannot counterfeit the geometry domain rule: {denial:?}")
    };
    assert_eq!(identity.rule_id.as_str(), "AtMostTwoProbeEnclosures");
}
