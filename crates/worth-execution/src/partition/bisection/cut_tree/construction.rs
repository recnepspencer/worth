use std::collections::{BTreeMap, BTreeSet};

use super::super::{BisectionDenial, WeightedItem};
use super::CutTree;
use crate::{
    backend::KernelStop,
    partition::{PartitionItemId, PartitionUpdateDenial, PartitionWork},
};

pub(crate) fn build_tree(
    path: u64,
    members: BTreeSet<PartitionItemId>,
    items: &BTreeMap<PartitionItemId, WeightedItem>,
    edges: &BTreeMap<(PartitionItemId, PartitionItemId), u64>,
    max_leaf_weight: u64,
    work: &mut PartitionWork,
    checkpoint: &mut dyn FnMut(u64) -> Result<(), KernelStop>,
) -> Result<CutTree, PartitionUpdateDenial> {
    checkpoint(members.len() as u64).map_err(PartitionUpdateDenial::Stop)?;
    let weight = members
        .iter()
        .try_fold(0_u64, |sum, id| sum.checked_add(items[id].weight))
        .ok_or(PartitionUpdateDenial::Bisection(
            BisectionDenial::WeightOverflow,
        ))?;
    work.members_visited += members.len() as u64;
    let mut tree = CutTree {
        path,
        members,
        weight,
        children: None,
    };
    if tree.weight <= max_leaf_weight || tree.members.len() <= 1 {
        return Ok(tree);
    }
    let left_path = path.checked_mul(2).ok_or(PartitionUpdateDenial::Bisection(
        BisectionDenial::PathExhausted,
    ))?;
    let right_path = left_path
        .checked_add(1)
        .ok_or(PartitionUpdateDenial::Bisection(
            BisectionDenial::PathExhausted,
        ))?;
    let ordered: Vec<_> = tree.members.iter().copied().collect();
    let mut best = (u64::MAX, u64::MAX, 0_usize);
    let mut left_weight = 0_u64;
    for split in 1..ordered.len() {
        checkpoint(split as u64).map_err(PartitionUpdateDenial::Stop)?;
        work.members_visited += split as u64;
        left_weight += items[&ordered[split - 1]].weight;
        let left: BTreeSet<_> = ordered[..split].iter().copied().collect();
        let mut cut = 0_u64;
        for ((a, b), weight) in edges {
            checkpoint(1).map_err(PartitionUpdateDenial::Stop)?;
            work.edges_visited += 1;
            if tree.members.contains(a)
                && tree.members.contains(b)
                && left.contains(a) != left.contains(b)
            {
                cut = cut
                    .checked_add(*weight)
                    .ok_or(PartitionUpdateDenial::Bisection(
                        BisectionDenial::WeightOverflow,
                    ))?;
            }
        }
        let candidate = (left_weight.abs_diff(tree.weight - left_weight), cut, split);
        if candidate < best {
            best = candidate;
        }
    }
    checkpoint(ordered.len() as u64).map_err(PartitionUpdateDenial::Stop)?;
    work.members_visited += ordered.len() as u64;
    let left: BTreeSet<_> = ordered[..best.2].iter().copied().collect();
    let right: BTreeSet<_> = ordered[best.2..].iter().copied().collect();
    tree.children = Some((
        Box::new(build_tree(
            left_path,
            left,
            items,
            edges,
            max_leaf_weight,
            work,
            checkpoint,
        )?),
        Box::new(build_tree(
            right_path,
            right,
            items,
            edges,
            max_leaf_weight,
            work,
            checkpoint,
        )?),
    ));
    Ok(tree)
}
