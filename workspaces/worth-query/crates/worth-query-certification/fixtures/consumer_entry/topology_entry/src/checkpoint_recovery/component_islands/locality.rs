//! Phase1 business reads for one real split, excluding privileged oracle work.
use super::*;
use worth_query_host::facade::application_invariants::WorthQueryInvariantProjectionWork;

#[test]
fn real_split_component_reads_are_unchanged_with_ten_and_ten_thousand_unrelated_islands() {
    let _guard = checkpoint_recovery_test_guard();
    let small = split(10);
    let large = split(10_000);
    assert_eq!(
        small, large,
        "unrelated islands add no preparation or evaluation business reads"
    );
}
fn split(
    unrelated: usize,
) -> (
    WorthQueryInvariantProjectionWork,
    WorthQueryInvariantProjectionWork,
) {
    let edge = (key(0, "a"), key(1, "a"));
    let mut world = world::World::new(unrelated + 2, std::slice::from_ref(&edge));
    let before = world.groups(&keys(2));
    assert_eq!(before.len(), 1);
    assert_eq!(before[0].members.len(), 6);
    let events = world.couple(edge.0, edge.1, false);
    let mut prepared = Vec::new();
    let mut evaluated = Vec::new();
    let mut selected = Vec::new();
    for event in events {
        match event {
            PlanarInvariantReadEvent::CompleteComponent {
                visited, identity, ..
            } => selected.push((visited, identity)),
            PlanarInvariantReadEvent::Prepared {
                reads,
                succeeded: true,
            } => prepared.push(reads),
            PlanarInvariantReadEvent::Evaluated {
                reads,
                returned_verdict: true,
            } => evaluated.push(reads),
            _ => {}
        }
    }
    assert_eq!(
        selected.len(),
        2,
        "both actual post-split components are read"
    );
    assert!(selected.iter().all(|(members, _)| *members == 3));
    assert_ne!(selected[0].1, selected[1].1);
    assert_eq!(
        prepared.len(),
        1,
        "coupling-only change schedules the ordinary rule"
    );
    assert_eq!(evaluated.len(), 1);
    assert!(prepared[0].adjacency_lists_read() > 0);
    // Oracle inspection begins only after the original measured invocation ends.
    let after = world.groups(&keys(2));
    assert_eq!(after.len(), 2);
    assert!(after.iter().all(|group| group.members.len() == 3));
    assert!(
        after.iter().any(|group| group.name == before[0].name),
        "the heir keeps the prior name"
    );
    (prepared[0], evaluated[0])
}
