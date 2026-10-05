use std::collections::{BTreeMap, BTreeSet};

use worth_foundational::PartitionIdentity;

use super::super::{PartitionItemId, PartitionWork};
use super::{BisectionDenial, BisectionQuality, WeightedItem};

pub(super) struct CutTree {
    pub(super) path: u64,
    pub(super) members: BTreeSet<PartitionItemId>,
    pub(super) weight: u64,
    pub(super) children: Option<(Box<CutTree>, Box<CutTree>)>,
}

pub(super) fn ordered_pair(
    a: PartitionItemId,
    b: PartitionItemId,
) -> (PartitionItemId, PartitionItemId) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

pub(super) fn lightest_leaf(tree: &CutTree) -> (PartitionIdentity, u64) {
    let Some((left, right)) = &tree.children else {
        return (PartitionIdentity::new(tree.path), tree.weight);
    };
    let left = lightest_leaf(left);
    let right = lightest_leaf(right);
    if left.1 <= right.1 {
        left
    } else {
        right
    }
}

pub(super) fn update_path(
    tree: &mut CutTree,
    target: u64,
    item: PartitionItemId,
    old_weight: u64,
    new_weight: u64,
    items: &BTreeMap<PartitionItemId, WeightedItem>,
    edges: &BTreeMap<(PartitionItemId, PartitionItemId), u64>,
    incident: &BTreeMap<PartitionItemId, BTreeSet<(PartitionItemId, PartitionItemId)>>,
    max_leaf_weight: u64,
    tolerance: u64,
    routes: &mut BTreeMap<PartitionItemId, PartitionIdentity>,
    work: &mut PartitionWork,
) -> Result<(), BisectionDenial> {
    let recut_path = planned_recut(
        tree,
        target,
        old_weight,
        new_weight,
        max_leaf_weight,
        tolerance,
    );
    let mut replacement = if let Some(path) = recut_path {
        let selected = subtree(tree, path);
        let mut members = selected.members.clone();
        work.members_visited += members.len() as u64;
        if new_weight == 0 {
            members.remove(&item);
        } else {
            members.insert(item);
        }
        let mut local_edges = BTreeMap::new();
        for member in &members {
            if let Some(pairs) = incident.get(member) {
                for pair in pairs {
                    work.edges_visited += 1;
                    if pair.0 == *member && members.contains(&pair.1) {
                        local_edges.insert(*pair, edges[pair]);
                    }
                }
            }
        }
        let mut unmetered = |_| Ok(());
        Some(
            build_tree(
                path,
                members,
                items,
                &local_edges,
                max_leaf_weight,
                work,
                &mut unmetered,
            )
            .map_err(|denial| match denial {
                crate::partition::PartitionUpdateDenial::Bisection(reason) => reason,
                _ => unreachable!("unmetered construction cannot stop"),
            })?,
        )
    } else {
        None
    };
    apply_path(
        tree,
        target,
        item,
        old_weight,
        new_weight,
        recut_path,
        &mut replacement,
        routes,
        work,
    );
    Ok(())
}

fn belongs_to(mut target: u64, ancestor: u64) -> bool {
    while target > ancestor {
        target >>= 1;
    }
    target == ancestor
}

pub(super) fn planned_recut(
    tree: &CutTree,
    target: u64,
    old_weight: u64,
    new_weight: u64,
    max_leaf_weight: u64,
    tolerance: u64,
) -> Option<u64> {
    if let Some((left, right)) = &tree.children {
        let left_changed = belongs_to(target, left.path);
        let changed = if left_changed { left } else { right };
        let changed_weight = changed.weight - old_weight + new_weight;
        let other_weight = if left_changed {
            right.weight
        } else {
            left.weight
        };
        if changed_weight == 0 || changed_weight.abs_diff(other_weight) > tolerance {
            return Some(tree.path);
        }
        planned_recut(
            changed,
            target,
            old_weight,
            new_weight,
            max_leaf_weight,
            tolerance,
        )
    } else {
        let next_weight = tree.weight - old_weight + new_weight;
        let next_count = tree.members.len() + usize::from(old_weight == 0 && new_weight != 0)
            - usize::from(old_weight != 0 && new_weight == 0);
        (next_weight > max_leaf_weight && next_count > 1).then_some(tree.path)
    }
}

pub(super) fn subtree(tree: &CutTree, path: u64) -> &CutTree {
    if tree.path == path {
        return tree;
    }
    let (left, right) = tree.children.as_ref().unwrap();
    subtree(
        if belongs_to(path, left.path) {
            left
        } else {
            right
        },
        path,
    )
}

pub(super) fn apply_path(
    tree: &mut CutTree,
    target: u64,
    item: PartitionItemId,
    old_weight: u64,
    new_weight: u64,
    recut_path: Option<u64>,
    replacement: &mut Option<CutTree>,
    routes: &mut BTreeMap<PartitionItemId, PartitionIdentity>,
    work: &mut PartitionWork,
) {
    if new_weight == 0 {
        tree.members.remove(&item);
    } else {
        tree.members.insert(item);
    }
    tree.weight = tree.weight - old_weight + new_weight;
    if recut_path == Some(tree.path) {
        let rebuilt = replacement.take().unwrap();
        publish_routes(&rebuilt, routes, work);
        *tree = rebuilt;
        work.subtrees_recut += 1;
    } else if let Some((left, right)) = &mut tree.children {
        let child = if belongs_to(target, left.path) {
            left
        } else {
            right
        };
        apply_path(
            child,
            target,
            item,
            old_weight,
            new_weight,
            recut_path,
            replacement,
            routes,
            work,
        );
    } else if new_weight != 0
        && routes.insert(item, PartitionIdentity::new(tree.path))
            != Some(PartitionIdentity::new(tree.path))
    {
        work.items_rerouted += 1;
    }
}
mod construction;
pub(super) use construction::build_tree;

// Walk each cut node once and write leaf routes directly into the retained map.
// A binary cut with m nonempty leaves has fewer than 2m nodes and at most m
// route writes, so the checked caller can precharge 3m commit units.
fn publish_routes(
    tree: &CutTree,
    routes: &mut BTreeMap<PartitionItemId, PartitionIdentity>,
    work: &mut PartitionWork,
) {
    if let Some((left, right)) = &tree.children {
        publish_routes(left, routes, work);
        publish_routes(right, routes, work);
    } else {
        for item in &tree.members {
            let partition = PartitionIdentity::new(tree.path);
            if routes.insert(*item, partition) != Some(partition) {
                work.items_rerouted += 1;
            }
        }
    }
}

pub(super) fn collect_leaves(
    tree: &CutTree,
    result: &mut BTreeMap<PartitionIdentity, BTreeSet<PartitionItemId>>,
) {
    if let Some((left, right)) = &tree.children {
        collect_leaves(left, result);
        collect_leaves(right, result);
    } else {
        result.insert(PartitionIdentity::new(tree.path), tree.members.clone());
    }
}

pub(super) fn collect_interfaces(
    tree: &CutTree,
    edges: &BTreeMap<(PartitionItemId, PartitionItemId), u64>,
    result: &mut BTreeMap<PartitionIdentity, BTreeSet<(PartitionItemId, PartitionItemId)>>,
) {
    if let Some((left, right)) = &tree.children {
        let crossing = edges
            .keys()
            .filter(|(a, b)| {
                left.members.contains(a) != left.members.contains(b)
                    && tree.members.contains(a)
                    && tree.members.contains(b)
            })
            .copied()
            .collect();
        result.insert(PartitionIdentity::new(tree.path), crossing);
        collect_interfaces(left, edges, result);
        collect_interfaces(right, edges, result);
    }
}

pub(super) fn summarize_quality(
    tree: &CutTree,
    edges: &BTreeMap<(PartitionItemId, PartitionItemId), u64>,
    quality: &mut BisectionQuality,
) {
    if let Some((left, right)) = &tree.children {
        quality.cut_count += 1;
        quality.maximum_cut_imbalance = quality
            .maximum_cut_imbalance
            .max(left.weight.abs_diff(right.weight));
        for ((a, b), weight) in edges {
            if tree.members.contains(a)
                && tree.members.contains(b)
                && left.members.contains(a) != left.members.contains(b)
            {
                quality.cut_weight = quality.cut_weight.saturating_add(*weight);
            }
        }
        summarize_quality(left, edges, quality);
        summarize_quality(right, edges, quality);
    } else {
        quality.maximum_leaf_weight = quality.maximum_leaf_weight.max(tree.weight);
    }
}
