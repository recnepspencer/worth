use super::*;
use crate::data::aspect::{Aspect, PartitionVersionMap};
use crate::data::output::{PartitionSubscription, ScopePath};
use crate::data::retained_storage::RetainedStoragePreparation as Work;

fn path(depth: usize, leaf: &str) -> ScopePath {
    ScopePath::new((0..depth).map(|n| {
        if n + 1 == depth {
            leaf.to_owned()
        } else {
            format!("p{n}")
        }
    }))
    .unwrap()
}

fn stamp(n: u64) -> AspectVersion {
    AspectVersion::zero().with(Aspect::new(0), n)
}

#[test]
fn exact_leaf_writes_preserve_siblings_and_advance_subtree_ancestors() {
    for depth in [1, 2, 4, 8] {
        let leaf = path(depth, "left");
        let sibling = path(depth, "right");
        let mut map = PartitionVersionOverrides::default();
        let region = ChangedRegion::exact(leaf.clone());
        map.apply_evaluation(stamp(7), &[region]);
        assert_eq!(
            map.scoped_or_global(&PartitionSubscription::exact(leaf.clone()), stamp(7))
                .get(Aspect::new(0)),
            7
        );
        assert_eq!(
            map.scoped_or_global(&PartitionSubscription::exact(sibling), stamp(7))
                .get(Aspect::new(0)),
            0
        );
        assert_eq!(
            map.scoped_or_global(&PartitionSubscription::subtree(leaf.clone()), stamp(7))
                .get(Aspect::new(0)),
            7
        );
        if depth > 1 {
            let ancestor = leaf.prefix(depth - 1).unwrap();
            assert_eq!(
                map.scoped_or_global(&PartitionSubscription::subtree(ancestor.clone()), stamp(7))
                    .get(Aspect::new(0)),
                7
            );
            assert_eq!(
                map.scoped_or_global(&PartitionSubscription::exact(ancestor), stamp(7))
                    .get(Aspect::new(0)),
                0
            );
        }
    }
}

#[test]
fn subtree_write_reaches_unseen_deep_leaf_and_survives_fork_and_json() {
    let ancestor = path(2, "branch");
    let descendant = ancestor
        .clone()
        .with_segment("three")
        .unwrap()
        .with_segment("four")
        .unwrap()
        .with_segment("five")
        .unwrap()
        .with_segment("six")
        .unwrap()
        .with_segment("seven")
        .unwrap()
        .with_segment("eight")
        .unwrap();
    let mut source = PartitionVersionMap::zero();
    source.apply_evaluation(stamp(11), &[ChangedRegion::subtree(ancestor.clone())]);
    let fork = source.clone();
    let restored: PartitionVersionMap =
        serde_json::from_str(&serde_json::to_string(&fork).unwrap()).unwrap();
    assert_eq!(
        restored.version_for_scope(
            Aspect::new(0),
            Some(&PartitionSubscription::exact(descendant))
        ),
        11
    );
    assert_eq!(
        restored.version_for_scope(
            Aspect::new(0),
            Some(&PartitionSubscription::exact(ancestor))
        ),
        11
    );
    assert_eq!(
        restored.version_for_scope(
            Aspect::new(0),
            Some(&PartitionSubscription::exact(path(2, "sibling")))
        ),
        0
    );
    assert_eq!(source, fork);
    let mut obsolete = serde_json::to_value(&source).unwrap();
    obsolete.as_object_mut().unwrap().remove("overrides");
    obsolete["partitions"] = serde_json::json!({});
    assert!(serde_json::from_value::<PartitionVersionMap>(obsolete).is_err());
}

#[test]
fn admission_counts_each_prefix_and_rejects_one_visit_short() {
    let regions = [ChangedRegion::exact(path(8, "leaf"))];
    let map = PartitionVersionOverrides::default();
    let mut measured = Work::new(usize::MAX);
    map.admit_evaluation_work(&regions, &mut EvaluationWork::Conditional(&mut measured))
        .unwrap();
    let cost = measured.visits();
    let mut short = Work::new(cost - 1);
    assert!(map
        .admit_evaluation_work(&regions, &mut EvaluationWork::Conditional(&mut short))
        .is_err());
    assert!(map.paths.is_empty());
}
