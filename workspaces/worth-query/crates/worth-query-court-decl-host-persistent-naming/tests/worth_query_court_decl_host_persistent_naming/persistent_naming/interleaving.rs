//! Hostile interleavings of authored deletion, recorded probes, `undo_step`,
//! `redo_to` and arbitrary `checkout`: after every step each held name means
//! exactly what the checked-out node holds, every live member keeps its exact
//! original identity, and independent installations mint byte-identical names.
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;

use proptest::prelude::*;
use proptest::test_runner::{Config, FileFailurePersistence, RngAlgorithm, TestRng};

use super::history::{
    named, navigate, probe, probe_refused, record_edit, resolution, DeleteAction, Navigate,
};
use super::*;
use probe::Observation;
use worth_query_decl::facade::application_naming::WorthQueryName;

const KEYS: [&str; 2] = ["direct", "nested"];

#[derive(Clone, Debug)]
enum Step {
    Probe,
    Delete(usize),
    Undo,
    Redo(usize),
    Checkout(usize),
}

fn step() -> impl Strategy<Value = Step> {
    prop_oneof![
        2 => Just(Step::Probe),
        3 => (0..KEYS.len()).prop_map(Step::Delete),
        3 => Just(Step::Undo),
        2 => (0..8usize).prop_map(Step::Redo),
        4 => (0..16usize).prop_map(Step::Checkout),
    ]
}

/// The independently owned oracle: each recorded node's parent and the
/// members live there.
struct Node {
    id: WorthQueryHistoryNodeId,
    parent: Option<usize>,
    live: [bool; 2],
}

/// Names minted by the first installation; every later one must agree.
static FIRST_NAMES: OnceLock<[WorthQueryName<Member>; 2]> = OnceLock::new();
/// Navigations that restored, and that retired, a named member.
static RESTORED: AtomicUsize = AtomicUsize::new(0);
static RETIRED: AtomicUsize = AtomicUsize::new(0);

fn run(steps: &[Step], case: u64) {
    let token = format!("interleaving-{case}");
    let host = world::install();
    let mut document = world::attach(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let branch = |document: &world::Document<'_>| {
        document
            .selection(&request)
            .unwrap()
            .physical_endpoint()
            .product_branch()
    };
    let root = document.current_step(&request).unwrap();
    let mut nodes = vec![Node {
        id: root.id(),
        parent: None,
        live: [true, true],
    }];
    let (next, observed) = probe(&host, document, &request, &token, &KEYS, 0);
    document = next;
    let names = [named(&observed, KEYS[0]), named(&observed, KEYS[1])];
    assert_eq!(
        FIRST_NAMES.get_or_init(|| names),
        &names,
        "fresh installations mint identical names"
    );
    for (key, name) in KEYS.iter().zip(names) {
        probe::hold(&token, key, name);
    }
    let originals = KEYS.map(|key| world::member_identity(&host, branch(&document), key).unwrap());
    nodes.push(Node {
        id: document.current_step(&request).unwrap().id(),
        parent: Some(0),
        live: [true, true],
    });
    let mut current = 1;
    for (ordinal, step) in steps.iter().enumerate() {
        let key = 0x5a71_1000 + u128::from(case) * 64 + ordinal as u128;
        let before = nodes[current].live;
        let children: Vec<usize> = (0..nodes.len())
            .filter(|&node| nodes[node].parent == Some(current))
            .collect();
        match *step {
            Step::Probe => {
                let live = nodes[current].live;
                if live.iter().all(|live| *live) {
                    let (next, observed) =
                        probe(&host, document, &request, &token, &KEYS, ordinal as u64 + 1);
                    document = next;
                    verify(&observed, live, &names);
                    nodes.push(Node {
                        id: document.current_step(&request).unwrap().id(),
                        parent: Some(current),
                        live,
                    });
                    current = nodes.len() - 1;
                } else {
                    let (next, observed, denial) =
                        probe_refused(&host, document, &request, &token, &KEYS, ordinal as u64 + 1);
                    document = next;
                    assert!(
                        denial.contains("PersistentName(UntrackedDependency)"),
                        "{denial}"
                    );
                    verify(&observed, live, &names);
                }
            }
            Step::Delete(index) => {
                // `nested` is owned only through `direct`'s `Child` edge.
                // Deleting `direct` first would orphan `nested` out of the
                // model's owned graph, which a later selection of that node
                // retires (existing owned-graph semantics, not naming), so the
                // oracle deletes `direct` only once `nested` is gone.
                if !nodes[current].live[index] || (index == 0 && nodes[current].live[1]) {
                    continue;
                }
                document = record_edit!(
                    &host,
                    document,
                    &request,
                    delete::Intent {
                        key: KEYS[index].into()
                    },
                    DeleteAction,
                    format!("{token}:delete:{ordinal}")
                );
                let mut live = nodes[current].live;
                live[index] = false;
                nodes.push(Node {
                    id: document.current_step(&request).unwrap().id(),
                    parent: Some(current),
                    live,
                });
                current = nodes.len() - 1;
            }
            Step::Undo => {
                let Some(parent) = nodes[current].parent else {
                    continue;
                };
                navigate(&document, &request, Navigate::Undo, key);
                current = parent;
            }
            Step::Redo(choice) => {
                let Some(&child) = children.get(choice % children.len().max(1)) else {
                    continue;
                };
                navigate(&document, &request, Navigate::Redo(nodes[child].id), key);
                current = child;
            }
            Step::Checkout(choice) => {
                let target = choice % nodes.len();
                if target == current {
                    continue;
                }
                navigate(
                    &document,
                    &request,
                    Navigate::Checkout(nodes[target].id),
                    key,
                );
                current = target;
            }
        }
        if matches!(step, Step::Undo | Step::Redo(_) | Step::Checkout(_)) {
            for index in 0..KEYS.len() {
                match (before[index], nodes[current].live[index]) {
                    (false, true) => RESTORED.fetch_add(1, Ordering::Relaxed),
                    (true, false) => RETIRED.fetch_add(1, Ordering::Relaxed),
                    _ => 0,
                };
            }
        }
        assert_eq!(
            document.current_step(&request).unwrap().id(),
            nodes[current].id
        );
        for (index, key) in KEYS.iter().enumerate() {
            let identity = world::member_identity(&host, branch(&document), key);
            let expected = nodes[current].live[index].then_some(originals[index]);
            assert_eq!(identity, expected, "{key} at step {ordinal} ({step:?})");
        }
    }
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}

fn verify(observed: &[Observation], live: [bool; 2], names: &[WorthQueryName<Member>; 2]) {
    for (index, key) in KEYS.iter().enumerate() {
        if live[index] {
            assert_eq!(named(observed, key), names[index], "{observed:?}");
            assert_eq!(
                resolution(observed, key),
                &Observation::ResolvedTo {
                    label: (*key).to_owned(),
                    key: (*key).to_owned()
                }
            );
        } else {
            assert!(observed.contains(&Observation::Absent {
                key: (*key).to_owned()
            }));
            assert_eq!(
                resolution(observed, key),
                &Observation::Retired {
                    label: (*key).to_owned()
                }
            );
        }
    }
}

#[test]
fn hostile_history_interleavings_keep_every_held_name_on_its_node() {
    // A fixed seed makes the case set, and so the hit guards below, exact.
    let rng = TestRng::deterministic_rng(RngAlgorithm::ChaCha);
    let mut runner = proptest::test_runner::TestRunner::new_with_rng(Config {
        cases: 16,
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct(
            "tests/worth_query_court_decl_host_persistent_naming/persistent_naming/proptest-regressions/interleaving.txt",
        ))),
        ..Config::default()
    }, rng);
    let case = std::cell::Cell::new(0_u64);
    runner
        .run(&proptest::collection::vec(step(), 8..16), |steps| {
            case.set(case.get() + 1);
            run(&steps, case.get());
            Ok(())
        })
        .unwrap();
    assert!(
        RESTORED.load(Ordering::Relaxed) >= 4,
        "navigation must restore named members"
    );
    assert!(
        RETIRED.load(Ordering::Relaxed) >= 4,
        "navigation must retire named members"
    );
}
