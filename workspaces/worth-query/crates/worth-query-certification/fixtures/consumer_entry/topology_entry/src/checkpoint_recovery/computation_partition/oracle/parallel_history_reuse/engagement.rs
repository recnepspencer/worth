//! Engagement is judged from declared partition changes, not reported run names.
use super::*;
pub(super) fn initialize(change: Change, model: &mut ReuseFacts) {
    match change {
        Change::EmptyBefore | Change::EmptyStable => {
            for item in &mut model.items {
                item.member = false;
            }
        }
        Change::EmptyAfter => {
            for (index, item) in model.items.iter_mut().enumerate() {
                item.member = index == 0;
            }
        }
        _ => {}
    }
}
type PartitionFacts = Vec<(u64, u64, u64, bool, Option<u64>)>;
fn partitions(model: &ReuseFacts) -> std::collections::BTreeMap<u32, PartitionFacts> {
    let mut result = std::collections::BTreeMap::<u32, PartitionFacts>::new();
    for item in model.members() {
        result.entry(item.key).or_default().push((
            item.number,
            item.value,
            item.work,
            item.fault,
            item.key
                .is_multiple_of(2)
                .then_some(model.weights[usize::from(model.odd)]),
        ));
    }
    for facts in result.values_mut() {
        facts.sort_unstable();
    }
    result
}
pub(super) fn assert_kernels(
    change: Change,
    before: &ReuseFacts,
    after: &ReuseFacts,
    reuse: bool,
    actual: &OracleRun,
) {
    let prior = partitions(before);
    let current = partitions(after);
    let full = !reuse
        || matches!(
            change,
            Change::Input | Change::EmptyStable | Change::CodeIdentity
        );
    let computed = if full {
        current.len()
    } else {
        current
            .iter()
            .filter(|(key, facts)| prior.get(key) != Some(facts))
            .count()
    };
    if matches!(change, Change::CodeIdentity) {
        assert_eq!(
            actual.calls.kernels,
            current.len(),
            "code identity alone forbids carrying any partition"
        );
    }
    assert_eq!(
        actual.calls.kernels, computed,
        "every changed or unretained partition computes exactly once; carried partitions never compute"
    );
    let carried = current
        .len()
        .checked_sub(actual.calls.kernels)
        .expect("no extra partition kernels");
    let expected_carried = current.len() - computed;
    assert_eq!(
        carried, expected_carried,
        "every reuse cell carries exactly the partitions admitted by its independent facts model"
    );
    let declared_carried = if !reuse {
        0
    } else {
        match change {
            Change::Gather
            | Change::Fault
            | Change::Join
            | Change::Leave
            | Change::Replacement
            | Change::CarriedCeiling => 3,
            Change::Shared | Change::Key | Change::DigestSwap => 2,
            Change::Input
            | Change::CodeIdentity
            | Change::EmptyBefore
            | Change::EmptyAfter
            | Change::EmptyStable => 0,
        }
    };
    assert_eq!(
        carried, declared_carried,
        "each named edit keeps its declared carried-partition witness"
    );
    println!("REUSE_ENGAGEMENT:computed={computed}:carried={carried}");
    if matches!(change, Change::Fault) {
        assert_eq!(
            computed,
            if reuse { 1 } else { 4 },
            "the fault changes one of four partition inputs"
        );
        assert_eq!(
            current.len() - computed,
            if reuse { 3 } else { 0 },
            "the other three fault-cell partitions are carried iff reuse is on"
        );
    }
}

pub(super) fn assert_empty_history(
    change: Change,
    before: &ReuseFacts,
    after: &ReuseFacts,
    first: &OracleRun,
    next: &OracleRun,
) {
    if !matches!(
        change,
        Change::EmptyBefore | Change::EmptyAfter | Change::EmptyStable
    ) {
        return;
    }
    let expected = (partitions(before).len(), partitions(after).len());
    assert_eq!(
        (first.calls.kernels, next.calls.kernels),
        expected,
        "empty histories have independently declared before/after kernel counts"
    );
    let [prior] = first.published.as_slice() else {
        panic!("empty history publishes one prior slot")
    };
    let [current] = next.published.as_slice() else {
        panic!("empty history publishes one successor slot")
    };
    // Empty trees still carry the input/edition proof; reuse-off ignores rather
    // than discards that publication. A reported run name cannot prove this.
    let retained = prior.tree_node_sharing_with_for_test::<RegionKey, Entry, f64>(current);
    assert_eq!(
        retained.len(),
        expected.0,
        "the real retained prior keeps its original partitions"
    );
    let declared = match change {
        Change::EmptyBefore => (0, 1),
        Change::EmptyAfter => (1, 0),
        Change::EmptyStable => (0, 0),
        _ => unreachable!(),
    };
    assert_eq!(
        expected, declared,
        "the three empty input histories are different model observations"
    );
}
