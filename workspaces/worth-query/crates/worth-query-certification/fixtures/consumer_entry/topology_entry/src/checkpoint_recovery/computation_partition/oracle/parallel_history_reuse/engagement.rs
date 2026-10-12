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
    assert_eq!(
        actual.calls.kernels, computed,
        "every changed or unretained partition computes exactly once; carried partitions never compute"
    );
    println!(
        "REUSE_ENGAGEMENT:computed={computed}:carried={}",
        current.len() - computed
    );
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
