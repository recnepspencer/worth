//! Any mix of envelope steps, navigation and paging keeps every step's snapshot.
use super::*;
use proptest::prelude::*;
use worth_query_host::facade::application_entry::WorthQueryEditEnvelopeOutcome as Bare;

/// One caller key for each envelope a sequence can record.
const KEYS: [&str; 8] = ["p0", "p1", "p2", "p3", "p4", "p5", "p6", "p7"];

#[derive(Clone, Debug)]
enum Action {
    /// Record a new step under an unused key.
    Envelope { model: u64, member: u64 },
    /// Repeat the key and intent of an envelope this sequence recorded.
    Replay(usize),
    /// An envelope whose member stage the domain denies.
    Refused,
    Undo,
    /// Redo to one child of the current step.
    RedoTo(usize),
    /// Check out any step this sequence recorded, or the root.
    Checkout(usize),
    /// Read the whole tree in pages of this width.
    Page(usize),
}

fn action() -> impl Strategy<Value = Action> {
    prop_oneof![
        3 => (2u64..=9, 1u64..=99)
            .prop_map(|(model, member)| Action::Envelope { model, member }),
        1 => (0usize..8).prop_map(Action::Replay),
        1 => Just(Action::Refused),
        2 => Just(Action::Undo),
        1 => (0usize..8).prop_map(Action::RedoTo),
        2 => (0usize..8).prop_map(Action::Checkout),
        1 => (1usize..=5).prop_map(Action::Page),
    ]
}

/// What the sequence expects of one step.
struct Node {
    id: WorthQueryHistoryNodeId,
    parent: Option<WorthQueryHistoryNodeId>,
    at: Snapshot,
}

/// The expected tree in recorded order, and the step the document selects.
struct Expected {
    nodes: Vec<Node>,
    current: WorthQueryHistoryNodeId,
    recorded: Vec<(Stages, WorthQueryHistoryNodeId)>,
}

impl Expected {
    fn ids(&self) -> Vec<WorthQueryHistoryNodeId> {
        self.nodes.iter().map(|node| node.id).collect()
    }

    fn node(&self, id: WorthQueryHistoryNodeId) -> &Node {
        self.nodes.iter().find(|node| node.id == id).unwrap()
    }
}

/// Every step of the tree, read in pages of `width`.
fn pages(host: &world::Host, document: &Forest<'_>, width: usize) -> Vec<WorthQueryHistoryNodeId> {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let width = NonZeroUsize::new(width).unwrap();
    let mut after = None;
    let mut all = Vec::new();
    for _ in 0..64 {
        let limits = WorthQueryHistoryPageLimits {
            maximum_entries: width,
            maximum_examined_entries: width,
            maximum_owned_bytes: NonZeroUsize::new(16 * 1024).unwrap(),
        };
        let page = document
            .history_page(&request, limits, after.as_ref())
            .unwrap();
        all.extend(page.rows().iter().map(|row| row.step().id()));
        after = page.continuation();
        if after.is_none() {
            return all;
        }
    }
    panic!("bounded paging must end");
}

/// The document selects the expected step and reads that step's snapshot.
fn verify(host: &world::Host, document: &Forest<'_>, expected: &Expected, context: &str) {
    assert_eq!(
        observe(host, document).current,
        expected.current,
        "{context}"
    );
    assert_eq!(
        snapshot(host, document),
        expected.node(expected.current).at,
        "{context}"
    );
    assert_eq!(pages(host, document, 5), expected.ids(), "{context}");
}

fn holds(actions: &[Action]) {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = world::install();
    let mut document = attach_with_both_trees(&host);
    let root = observe(&host, &document).current;
    let mut expected = Expected {
        nodes: vec![Node {
            id: root,
            parent: None,
            at: snapshot(&host, &document),
        }],
        current: root,
        recorded: Vec::new(),
    };
    assert_eq!(pages(&host, &document, 5), [root]);
    for (ordinal, action) in actions.iter().enumerate() {
        let context = format!("{actions:?} at {ordinal}");
        let idempotency = 0x57e0_1000 + ordinal as u128;
        match *action {
            Action::Envelope { model, member } => {
                let stages = Stages {
                    key: KEYS[expected.recorded.len()],
                    model,
                    member,
                };
                let (next, step) =
                    recorded(record(&host, document, &request_scope(), stages, metadata(b"")));
                document = next;
                assert_eq!(step.parent(), Some(expected.current), "{context}");
                let at = snapshot(&host, &document);
                assert_eq!(
                    (at.model, at.member, at.outputs.0.value, at.outputs.1.value),
                    (model, member, 10 * model, 1000 + model),
                    "{context}"
                );
                expected.nodes.push(Node {
                    id: step.id(),
                    parent: step.parent(),
                    at,
                });
                expected.recorded.push((stages, step.id()));
                expected.current = step.id();
            }
            Action::Replay(pick) if !expected.recorded.is_empty() => {
                let (stages, step) = expected.recorded[pick % expected.recorded.len()];
                document = match record(&host, document, &request_scope(), stages, metadata(b"")) {
                    Ok(Outcome::AlreadyRecorded { document, step: named }) => {
                        assert_eq!(named.id(), step, "{context}");
                        document
                    }
                    other => panic!("{context}: a replay must name its step: {other:?}"),
                };
            }
            Action::Refused => {
                let stages = Stages {
                    key: "denied",
                    model: 2,
                    member: 2000,
                };
                let (next, denial) =
                    refused(record(&host, document, &request_scope(), stages, metadata(b"")));
                document = next;
                assert!(
                    matches!(denial, Denial::Envelope(Bare::DomainDenied(_))),
                    "{context}: {denial:?}"
                );
            }
            Action::Undo => {
                if let Some(parent) = expected.node(expected.current).parent {
                    navigate(&host, &document, Move::Undo, idempotency);
                    expected.current = parent;
                }
            }
            Action::RedoTo(pick) => {
                let children: Vec<_> = expected
                    .nodes
                    .iter()
                    .filter(|node| node.parent == Some(expected.current))
                    .map(|node| node.id)
                    .collect();
                if !children.is_empty() {
                    let child = children[pick % children.len()];
                    navigate(&host, &document, Move::RedoTo(child), idempotency);
                    expected.current = child;
                }
            }
            Action::Checkout(pick) => {
                let target = expected.nodes[pick % expected.nodes.len()].id;
                if target != expected.current {
                    navigate(&host, &document, Move::Checkout(target), idempotency);
                    expected.current = target;
                }
            }
            Action::Page(width) => {
                assert_eq!(pages(&host, &document, width), expected.ids(), "{context}");
            }
            // Nothing is recorded yet, so there is no key to repeat.
            Action::Replay(_) => {}
        }
        verify(&host, &document, &expected, &context);
    }
    close(&host, document);
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 12, max_shrink_iters: 32, ..ProptestConfig::default() })]
    #[test]
    fn any_mix_of_envelopes_navigation_and_paging_keeps_every_step(
        actions in proptest::collection::vec(action(), 3..=8)
    ) {
        holds(&actions);
    }
}

/// The property is not vacuous: this sequence records, branches, replays,
/// refuses, pages and visits every step it recorded.
#[test]
fn a_branched_tree_survives_replay_refusal_and_paging() {
    holds(&[
        Action::Envelope { model: 2, member: 6 },
        Action::Envelope { model: 3, member: 7 },
        Action::Undo,
        Action::Envelope { model: 4, member: 8 },
        Action::Replay(1),
        Action::Refused,
        Action::Undo,
        Action::RedoTo(0),
        Action::Checkout(3),
        Action::Page(1),
    ]);
}
