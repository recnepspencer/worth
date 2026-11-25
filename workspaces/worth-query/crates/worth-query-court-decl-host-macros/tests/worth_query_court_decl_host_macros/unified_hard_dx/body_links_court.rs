//! Typed bodies replace Daymark's NewProject and SetDependency handlers: a created
//! project links to its scope and itself; a dependency toggles one exact edge.
use worth_query_decl::facade::application_schema::*;
use worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock;
use worth_query_host::facade::WorthQuerySessionRunDenial;

use super::body_links::*;
use super::courts::{open_daymark, temp_home_path};
use super::program::{owner, DaymarkHome};
use super::queries::BlockedTasks;
use super::schema::*;

fn home(label: &str) -> (DaymarkHome, WorthQueryAdmissionClock) {
    let clock = WorthQueryAdmissionClock::system();
    (open_daymark(&temp_home_path(label), &clock), clock)
}

fn keys(rows: impl IntoIterator<Item = u64>) -> Vec<u64> {
    rows.into_iter().collect()
}

#[test]
fn created_project_links_from_its_scope_and_to_itself_in_one_commit() {
    let (home, clock) = home("body-self-edge");
    let me = owner(&home, &clock);
    let session = home.session(&me);
    session
        .run(NewProject {
            parent: 1,
            id: 7,
            label: "Ops".into(),
        })
        .expect("scope link and self edge commit together");
    let nested = session.query(NestedProjects { project: 1 }).unwrap();
    assert_eq!(
        nested.iter().cloned().collect::<Vec<_>>(),
        vec![ProjectRow {
            project_key: 7,
            label: "Ops".into(),
        }]
    );
    let watched = |project| {
        keys(
            session
                .query(WatchedProjects { project })
                .unwrap()
                .iter()
                .map(|row| row.project_key),
        )
    };
    assert_eq!(watched(7), vec![7], "the created project watches itself");
    assert!(watched(1).is_empty(), "the scope gains no watch edge");
    assert!(
        session
            .query(NestedProjects { project: 7 })
            .unwrap()
            .is_empty(),
        "the self edge is not written through the scope relation"
    );
}

#[test]
fn self_edge_through_a_relation_forbidding_self_edges_is_refused_at_authoring() {
    let refused = NewProjectOperation::reference()
        .typed_body()
        .in_scope(Project::reference())
        .create_in_scope(Project::reference(), "project")
        .link_to_self(NestsProject::reference(), "nested-self")
        .finish();
    assert_eq!(
        refused.err(),
        Some(ApplicationOperationBodyAuthoringDenial::ForbiddenSelfEdge)
    );
}

#[test]
fn dependency_toggle_links_and_unlinks_one_exact_edge_and_repeats_are_noops() {
    let (home, clock) = home("body-toggle");
    let me = owner(&home, &clock);
    let session = home.session(&me);
    let blocked = || {
        keys(
            session
                .query(BlockedTasks { project: 1 })
                .unwrap()
                .iter()
                .map(|row| row.task_key),
        )
    };
    let set = |task, blocker, linked| {
        session
            .run(SetDependency {
                task,
                blocker,
                linked,
            })
            .map(|_| ())
    };
    assert_eq!(blocked(), vec![2], "seed: task 2 is blocked by task 1");
    // A repeated link would duplicate a DirectedSemanticEdge-unique edge if not a no-op.
    for _ in 0..2 {
        set(3, 1, true).expect("absent edge links; present edge is unchanged");
        assert_eq!(blocked(), vec![2, 3]);
    }
    set(3, 2, true).expect("a second outgoing edge from the same task links");
    // Unlinking 3 -> 1 must leave 3 -> 2 and the seeded 2 -> 1 in place.
    set(3, 1, false).expect("present edge unlinks");
    assert_eq!(
        blocked(),
        vec![2, 3],
        "task 3 is still blocked by open task 2"
    );
    for _ in 0..2 {
        set(3, 2, false).expect("present edge unlinks; absent edge is unchanged");
        assert_eq!(blocked(), vec![2]);
    }
}

#[test]
fn dependency_toggle_with_an_unknown_blocker_refuses_before_any_effect() {
    let (home, clock) = home("body-toggle-unknown");
    let me = owner(&home, &clock);
    let session = home.session(&me);
    let refused = session.run(SetDependency {
        task: 3,
        blocker: 99,
        linked: true,
    });
    assert!(
        matches!(refused, Err(WorthQuerySessionRunDenial::Request(_))),
        "{refused:?}"
    );
    let blocked = session.query(BlockedTasks { project: 1 }).unwrap();
    assert_eq!(keys(blocked.iter().map(|row| row.task_key)), vec![2]);
}
