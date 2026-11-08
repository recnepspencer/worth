//! One path plan, admission, and atomic replacement for every ordered edit.
use super::{
    funding::FundingSource,
    node::{EntryPlan, Link},
    plan, ExecutionOrderedMapDenial as Denial,
};
use crate::ChargedBytes;

pub(super) fn insert_native<K: Ord + Clone, V: Clone>(
    root: &Link<K, V>,
    key: &K,
    value: &V,
    source: &dyn FundingSource,
) -> Result<Link<K, V>, Denial> {
    insert(
        root,
        key,
        EntryPlan::native(key, value, source.uses_accounting())?,
        source,
    )
}
pub(super) fn insert_measured<K: Ord + Clone + ChargedBytes, V: Clone + ChargedBytes>(
    root: &Link<K, V>,
    key: &K,
    value: &V,
    source: &dyn FundingSource,
) -> Result<Link<K, V>, Denial> {
    insert(
        root,
        key,
        EntryPlan::measured(key, value, source.uses_accounting())?,
        source,
    )
}
fn insert<K: Ord + Clone, V: Clone>(
    root: &Link<K, V>,
    key: &K,
    entry: EntryPlan<'_, K, V>,
    source: &dyn FundingSource,
) -> Result<Link<K, V>, Denial> {
    plan::with_plan(root, |plan| {
        let next = plan.insert(root, key)?;
        let pool = plan.admission(next, entry.bytes(), source)?;
        let entry = entry.seal(&pool)?;
        plan.materialize(next, Some(&entry), &pool)
    })
}
pub(super) fn remove<K: Ord, V>(
    root: &Link<K, V>,
    key: &K,
    source: &dyn FundingSource,
) -> Result<Link<K, V>, Denial> {
    plan::with_plan(root, |plan| {
        let next = plan.remove(root, key)?;
        let pool = plan.admission(next, 0, source)?;
        plan.materialize(next, None, &pool)
    })
}
