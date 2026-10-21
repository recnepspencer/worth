use super::*;
use crate::{CanonicalBits, ReductionRunStop};
use std::sync::Weak;

#[derive(Clone, worth_foundational::facade::ChargedBytes)]
#[charged_bytes(crate = "crate")]
struct Payload {
    count: u64,
    labels: Vec<String>,
}

impl CanonicalBits for Payload {
    fn canonical_len(&self) -> Option<usize> {
        (self.count, &self.labels).canonical_len()
    }
    fn visit_canonical_bits(&self, visitor: &mut dyn FnMut(&[u8]) -> bool) -> bool {
        (self.count, &self.labels).visit_canonical_bits(visitor)
    }
}

fn combine(left: &Payload, right: &Payload) -> Payload {
    let mut labels = left.labels.clone();
    labels.extend(right.labels.iter().cloned());
    Payload {
        count: left.count + right.count,
        labels,
    }
}

fn owned_bytes(value: &Payload) -> u64 {
    (value.labels.capacity() * size_of::<String>()
        + value.labels.iter().map(String::capacity).sum::<usize>()) as u64
}

fn independent_node_charge(node: &Arc<Node<Payload>>) -> u64 {
    (size_of::<Node<Payload>>() + 2 * size_of::<usize>()) as u64
        + owned_bytes(&node.value)
        + owned_bytes(&node.aggregate)
        + node.left.as_ref().map_or(0, independent_node_charge)
        + node.right.as_ref().map_or(0, independent_node_charge)
}

fn weak_nodes(node: &Arc<Node<Payload>>, output: &mut Vec<Weak<Node<Payload>>>) {
    output.push(Arc::downgrade(node));
    for child in [&node.left, &node.right].into_iter().flatten() {
        weak_nodes(child, output);
    }
}

#[test]
fn derived_payloads_keep_full_tree_charges_while_path_copies_share_and_release_nodes() {
    let identities: Vec<_> = (1..=16).map(PartitionIdentity::new).collect();
    let values = (1..=16)
        .map(|count| Payload {
            count,
            labels: vec![format!("item-{count}")],
        })
        .collect();
    let (original, _) = ReductionTree::from_plan_checked(
        crate::ReductionPlan::try_from_sorted_unique(identities).unwrap(),
        values,
        Payload {
            count: 0,
            labels: Vec::new(),
        },
        combine,
        4096,
        || Ok::<_, ()>(()),
    )
    .unwrap();
    let root = original.root.as_ref().unwrap();
    let expected = independent_node_charge(root) + owned_bytes(&original.identity);
    assert_eq!(original.additional_charged_bytes(), expected);
    let mut weak = Vec::new();
    weak_nodes(root, &mut weak);

    let alias = original.clone();
    assert!(Arc::ptr_eq(root, alias.root.as_ref().unwrap()));
    assert_eq!(
        alias.additional_charged_bytes(),
        expected,
        "sharing does not discount the owner's declared tree charge"
    );
    let mut edited = original.clone();
    edited
        .update_checked(
            PartitionIdentity::new(16),
            Payload {
                count: 100,
                labels: vec![String::from("replacement")],
            },
            4096,
            || Ok::<_, ()>(()),
        )
        .unwrap();
    assert_eq!(original.result().count, 136);
    assert_eq!(edited.result().count, 220);
    assert_eq!(
        edited.additional_charged_bytes(),
        independent_node_charge(edited.root.as_ref().unwrap())
    );

    let before = Arc::clone(edited.root.as_ref().unwrap());
    let before_charge = edited.additional_charged_bytes();
    let failure = edited
        .update_checked(
            PartitionIdentity::new(1),
            Payload {
                count: 999,
                labels: vec![String::with_capacity(8192)],
            },
            4096,
            || Ok::<_, ()>(()),
        )
        .unwrap_err();
    assert_eq!(failure.reason, ReductionRunStop::ResultCapacityExceeded);
    assert!(Arc::ptr_eq(&before, edited.root.as_ref().unwrap()));
    assert_eq!(edited.additional_charged_bytes(), before_charge);
    assert_eq!(edited.result().count, 220);
    drop(before);
    drop(original);
    drop(alias);
    assert!(
        weak.iter().any(|node| node.upgrade().is_some()),
        "unchanged subtrees remain shared by the successor"
    );
    assert!(
        weak.iter().any(|node| node.upgrade().is_none()),
        "replaced paths release after their last owner drops"
    );
    drop(edited);
    assert!(weak.iter().all(|node| node.upgrade().is_none()));
}
