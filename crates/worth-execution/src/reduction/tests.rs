use worth_foundational::PartitionIdentity;

use crate::{oracle::CanonicalBits, report::ChargedBytes};

use super::{plan::priority, ReductionDenial, ReductionPlan, ReductionTree};

fn id(value: u64) -> PartitionIdentity {
    PartitionIdentity::new(value)
}

fn plan(identities: &[u64]) -> ReductionPlan {
    ReductionPlan::try_from_sorted_unique(identities.iter().copied().map(id).collect()).unwrap()
}

fn from_plan<T, F>(
    plan: ReductionPlan,
    values: Vec<T>,
    identity: T,
    combine: F,
) -> Result<(ReductionTree<T, F>, super::ReductionMetrics), ReductionDenial>
where
    T: Clone + ChargedBytes + CanonicalBits,
    F: Fn(&T, &T) -> T,
{
    let entries = plan.identities().iter().copied().zip(values).collect();
    ReductionTree::try_from_declared(plan, entries, identity, combine)
}

fn sum(left: &u64, right: &u64) -> u64 {
    left + right
}

#[test]
fn concatenation_preserves_partition_identity_order_after_build_and_update() {
    let (mut tree, _) = from_plan(
        plan(&[1, 2, 3, 4, 5, 6, 7, 8]),
        (1..=8).map(|value| vec![value]).collect(),
        Vec::<u64>::new(),
        |left, right| left.iter().chain(right).copied().collect(),
    )
    .unwrap();
    assert_eq!(
        tree.result(),
        &vec![1, 2, 3, 4, 5, 6, 7, 8],
        "reduction must combine in partition identity order after build"
    );
    tree.update(id(4), vec![40]).unwrap();
    assert_eq!(
        tree.result(),
        &vec![1, 2, 3, 40, 5, 6, 7, 8],
        "reduction must combine in partition identity order after update"
    );
}

#[test]
fn reduction_shape_uses_identity_digest_for_a_cancelling_float_pair() {
    // The fixed digest priorities of identities 1, 2, 3 put 3 at the root.
    // That groups the cancelling pair before adding 1, whose exact result is 1.
    let (tree, _) = from_plan(
        plan(&[1, 2, 3]),
        vec![1.0e16_f64, -1.0e16, 1.0],
        0.0,
        |left, right| left + right,
    )
    .unwrap();
    assert_eq!(
        tree.result().to_bits(),
        1.0_f64.to_bits(),
        "reduction shape must follow partition identity digest"
    );
}

#[test]
fn a_leaf_reads_the_value_its_partition_holds() {
    let (mut tree, _) = from_plan(plan(&[3, 5, 9]), vec![30_u64, 50, 90], 0, sum).unwrap();

    assert_eq!(tree.leaf(id(3)), Some(&30));
    assert_eq!(tree.leaf(id(5)), Some(&50));
    assert_eq!(tree.leaf(id(9)), Some(&90));
    assert_eq!(tree.leaf(id(4)), None, "no partition, no leaf");

    tree.update(id(5), 7).unwrap();
    assert_eq!(tree.leaf(id(5)), Some(&7), "an edit replaces the leaf");
    assert_eq!(tree.leaf(id(3)), Some(&30));
    assert_eq!(*tree.result(), 127, "the result reduces the leaves read");
}

#[test]
fn a_plans_build_work_is_the_work_its_checked_build_charges() {
    let mut state = 0x2545_f491_4f6c_dd1d_u64;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    for case in 0..200 {
        let count = usize::try_from(case % 40).unwrap();
        let mut identities = (0..count)
            .map(|_| if case % 2 == 0 { next() % 64 } else { next() })
            .collect::<Vec<_>>();
        identities.sort_unstable();
        identities.dedup();
        let values = identities.iter().map(|value| value % 7).collect();
        let (_, metrics) = from_plan(plan(&identities), values, 0, sum).unwrap();
        assert_eq!(
            plan(&identities).checked_build_work(),
            Some(metrics.charged_work),
            "{identities:?}"
        );
    }
}

fn oracle(values: &[(PartitionIdentity, f64)]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let root = values
        .iter()
        .enumerate()
        .min_by_key(|(_, (identity, _))| priority(*identity))
        .unwrap()
        .0;
    (oracle(&values[..root]) + values[root].1) + oracle(&values[root + 1..])
}

fn canonical_path_len(identities: &[PartitionIdentity], target: PartitionIdentity) -> usize {
    let root = identities
        .iter()
        .enumerate()
        .min_by_key(|(_, identity)| priority(**identity))
        .unwrap()
        .0;
    if identities[root] == target {
        1
    } else if target < identities[root] {
        1 + canonical_path_len(&identities[..root], target)
    } else {
        1 + canonical_path_len(&identities[root + 1..], target)
    }
}

#[test]
fn checked_plan_rejects_unsorted_and_duplicate_identities() {
    for identities in [[2, 1], [1, 1]] {
        assert!(matches!(
            ReductionPlan::try_from_sorted_unique(identities.into_iter().map(id).collect()),
            Err(ReductionDenial::IdentitiesNotCanonical)
        ));
    }
}

#[test]
fn declared_values_must_cover_the_checked_identities_exactly() {
    let result = ReductionTree::try_from_declared(
        plan(&[1, 2]),
        vec![(id(1), 7_u64), (id(3), 9_u64)],
        0_u64,
        sum,
    );
    assert!(matches!(result, Err(ReductionDenial::CoverageMismatch)));
}

#[test]
fn incremental_edits_match_fresh_tree_and_charge_only_affected_paths() {
    let original: Vec<u64> = (0..128).collect();
    let mut values = original.iter().map(|v| v + 1).collect::<Vec<_>>();
    let (mut tree, fresh) = from_plan(plan(&original), values.clone(), 0, sum).unwrap();
    assert!(fresh.combine_calls >= 2 * original.len() as u64);
    values[31] = 900;
    let changed = tree.update(id(31), 900).unwrap();
    assert!(changed.recombined_nodes > 0);
    assert!(changed.recombined_nodes < original.len() as u64);
    assert_eq!(changed.combine_calls, changed.charged_work);
    assert_eq!(changed.charged_work, changed.charged_span);
    let (rebuilt, _) = from_plan(plan(&original), values.clone(), 0, sum).unwrap();
    assert_eq!(tree.result(), rebuilt.result());
    assert_eq!(tree.update(id(31), 900).unwrap().combine_calls, 0);

    let inserted = tree.insert(id(128), 777).unwrap();
    assert!(inserted.recombined_nodes < 128);
    let mut grown = original.clone();
    grown.push(128);
    values.push(777);
    let (rebuilt, _) = from_plan(plan(&grown), values.clone(), 0, sum).unwrap();
    assert_eq!(tree.result(), rebuilt.result());
    assert!(matches!(
        tree.insert(id(128), 7),
        Err(ReductionDenial::IdentityAlreadyPresent(_))
    ));

    let removed = tree.delete(id(31)).unwrap();
    assert!(removed.recombined_nodes < 128);
    grown.remove(31);
    values.remove(31);
    let (rebuilt, _) = from_plan(plan(&grown), values, 0, sum).unwrap();
    assert_eq!(tree.result(), rebuilt.result());
    assert_eq!(tree.partition_count(), 128);
    assert!(matches!(
        tree.delete(id(31)),
        Err(ReductionDenial::UnknownIdentity(_))
    ));
}

#[test]
fn signed_zero_update_is_not_suppressed_by_numeric_equality() {
    let (mut tree, _) = from_plan(plan(&[1]), vec![0.0_f64], 1.0, |a, b| a * b).unwrap();
    assert_eq!(tree.result().to_bits(), 0.0_f64.to_bits());
    let changed = tree.update(id(1), -0.0).unwrap();
    assert_eq!(changed.recombined_nodes, 1);
    assert_eq!(tree.result().to_bits(), (-0.0_f64).to_bits());
}

#[test]
fn non_associative_float_follows_one_canonical_shape_after_edits() {
    let mut entries = vec![(id(1), 1.0e20_f64), (id(2), -1.0e20), (id(3), 3.0)];
    let (mut tree, _) = from_plan(
        plan(&[1, 2, 3]),
        entries.iter().map(|entry| entry.1).collect(),
        0.0,
        |a, b| a + b,
    )
    .unwrap();
    assert_eq!(tree.result().to_bits(), oracle(&entries).to_bits());
    tree.insert(id(4), 0.25).unwrap();
    entries.push((id(4), 0.25));
    assert_eq!(tree.result().to_bits(), oracle(&entries).to_bits());
    tree.update(id(2), -1.0e20 + 16384.0).unwrap();
    entries[1].1 = -1.0e20 + 16384.0;
    assert_eq!(tree.result().to_bits(), oracle(&entries).to_bits());
    tree.delete(id(1)).unwrap();
    entries.remove(0);
    assert_eq!(tree.result().to_bits(), oracle(&entries).to_bits());
}

#[test]
fn edit_history_cannot_change_shape_or_floating_point_bits() {
    let mut entries = vec![(id(0), 0.25_f64)];
    let (mut tree, _) = from_plan(plan(&[0]), vec![0.25_f64], 0.0_f64, |a, b| a + b).unwrap();
    for step in 1..64_u64 {
        let partition = id(step);
        let value = match step % 3 {
            0 => 1.0e20,
            1 => -1.0e20,
            _ => step as f64 + 0.125,
        };
        // Deletions make the retained tree's history differ from fresh build.
        tree.insert(partition, value).unwrap();
        entries.push((partition, value));
        if step % 4 == 0 {
            let removed = step / 2;
            tree.delete(id(removed)).unwrap();
            entries.retain(|entry| entry.0 != id(removed));
        }
        let canonical_ids: Vec<_> = entries.iter().map(|entry| entry.0).collect();
        let fresh_plan = ReductionPlan::try_from_sorted_unique(canonical_ids).unwrap();
        let (fresh, _) = from_plan(
            fresh_plan,
            entries.iter().map(|entry| entry.1).collect(),
            0.0,
            |a, b| a + b,
        )
        .unwrap();
        assert_eq!(
            tree.result().to_bits(),
            fresh.result().to_bits(),
            "step {step}"
        );
        assert_eq!(
            tree.result().to_bits(),
            oracle(&entries).to_bits(),
            "step {step}"
        );
    }
}

#[test]
fn equal_interior_encoding_cuts_off_ancestor_recombination() {
    let identities: Vec<_> = (0..32).collect();
    let mut values = vec![100_u64; 32];
    values[31] = 1;
    let (mut tree, _) = from_plan(plan(&identities), values, 0, |a, b| (*a).max(*b)).unwrap();
    let metrics = tree.update(id(31), 2).unwrap();
    assert_eq!(metrics.recombined_nodes, 2);
    assert!(
        canonical_path_len(
            &identities.iter().copied().map(id).collect::<Vec<_>>(),
            id(31)
        ) > 2
    );
    assert_eq!(*tree.result(), 100);
}

#[test]
fn declared_build_work_has_hand_counted_small_shapes() {
    // 1 alone: both spines contain 1. In [1, 2], root 1 has right child 2:
    // L=1, R=2. In [1, 2, 3], root 3 has left 1, whose right child is 2:
    // L=2, R=1. Count nodes and spine nodes, not the implementation's meter.
    for (ids, left, right, expected) in [
        (&[1][..], 1, 1, 7),
        (&[1, 2][..], 1, 2, 15),
        (&[1, 2, 3][..], 2, 1, 24),
    ] {
        assert_eq!(9 * ids.len() as u64 - left - right, expected);
        assert_eq!(plan(ids).checked_build_work(), Some(expected));
        let (_, metrics) = from_plan(plan(ids), vec![1_u64; ids.len()], 0, sum).unwrap();
        assert_eq!(metrics.charged_work, expected);
    }
}
