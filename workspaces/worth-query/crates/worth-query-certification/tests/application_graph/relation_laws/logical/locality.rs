//! Existing rule read observations distinguish local acquisition from a census.
use super::super::{installation::install, outcome, read};
use super::*;
use std::collections::BTreeSet;
use worth_query_host::facade::application_invariants::WorthQueryInvariantProjectionWork as Work;
#[derive(Clone, Debug)]
struct LocalLogical;
impl LawPolicy for LocalLogical {
    const INTEGRITY: ApplicationRelationIntegrity =
        ApplicationRelationIntegrity::same_context_unbounded_retain_dangling().orbit();
    const LOGICAL_RELATIONS: bool = true;
    const LOGICAL_CENSUS: bool = false;
    fn logical_read_observer() -> Option<observation::Observer> {
        observation::current()
    }
}
#[test]
fn logical_local_rule_business_acquisitions_ignore_ten_and_ten_thousand_unrelated_cohorts() {
    assert_eq!(
        run(10),
        run(10_000),
        "actual local preparation/evaluation business counts, not charged work"
    );
}
fn run(unrelated: u64) -> (Work, Work) {
    let capture = observation::Capture::place();
    let mut edges = vec![(0, 1), (1, 0)];
    edges.extend((2..unrelated + 2).map(|node| (node, node)));
    let world = install::<LocalLogical>(unrelated + 2, &edges);
    let before = read::committed(&world);
    assert_eq!(before.edges.len(), unrelated as usize + 2);
    let root = if before.identities[&0] < before.identities[&1] {
        0
    } else {
        1
    };
    let original = super::court::checked(&world, 699, root);
    assert_eq!(
        original.members.iter().cloned().collect::<BTreeSet<_>>(),
        BTreeSet::from([
            (before.identities[&root], "Negative".into()),
            (before.identities[&(1 - root)], "Positive".into()),
        ])
    );
    capture.take();
    outcome::committed(world.replace_edges(700, &[(0, 0), (1, 1)], &[(0, 1), (1, 0)]));
    let samples = capture.take();
    let prepared = samples
        .iter()
        .filter(|(evaluated, _)| !evaluated)
        .map(|(_, work)| *work)
        .collect::<Vec<_>>();
    let evaluated = samples
        .iter()
        .filter(|(evaluated, _)| *evaluated)
        .map(|(_, work)| *work)
        .collect::<Vec<_>>();
    assert_eq!(prepared.len(), 1);
    assert_eq!(evaluated.len(), 1);
    assert!(prepared[0].adjacency_lists_read() > 0);
    assert!(prepared[0].adjacency_edges_inspected() > 0);
    assert!(prepared[0].endpoint_records_read() > 0);
    // The privileged literal oracle runs after the measured original invocation.
    let after = read::committed(&world);
    let expected = BTreeSet::from([(0, 0), (1, 1)])
        .into_iter()
        .chain((2..unrelated + 2).map(|node| (node, node)))
        .collect::<BTreeSet<_>>();
    assert_eq!(after.edges, expected);
    assert_eq!(after.identities, before.identities);
    let logical = super::court::checked(&world, 701, root);
    assert_eq!(
        logical.members.iter().cloned().collect::<BTreeSet<_>>(),
        BTreeSet::from([
            (after.identities[&root], "Negative".into()),
            (after.identities[&root], "Positive".into()),
        ])
    );
    assert_eq!(
        logical.identity, original.identity,
        "the actual surviving Negative heir retains its Native name"
    );
    world.close();
    (prepared[0], evaluated[0])
}
