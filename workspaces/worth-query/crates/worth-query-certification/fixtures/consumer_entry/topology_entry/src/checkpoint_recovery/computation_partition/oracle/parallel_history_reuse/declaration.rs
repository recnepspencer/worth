//! Independently declared full-run compute and tree work. Input/item identity
//! encoding belongs to preparation and is not separately observable here.
use super::*;
pub(super) fn computed_work(model: &ReuseFacts) -> u64 {
    let identities = model
        .members()
        .map(|item| super::super::tree_work::identity(item.key))
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let tree = super::super::tree_work::build_charge(&identities);
    model.expected_compute_charge(tree)
}
