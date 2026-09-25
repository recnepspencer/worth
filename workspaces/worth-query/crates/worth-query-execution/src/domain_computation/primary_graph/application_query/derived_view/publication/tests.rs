use std::collections::BTreeSet;

use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, CanonicalFieldPath, FieldKey, LocatorAuthority,
};
use worth_relational::facade::identity::{EntityId, KindId, PartitionId};

use super::{DependencyIndex, ViewChange, ViewDependency};

fn entity(slot: u64) -> EntityId {
    EntityId::new(PartitionId::main(), slot, 1)
}

fn field(aspect: &AspectKey, name: &str) -> AspectFieldLocator {
    AspectFieldLocator::new(
        LocatorAuthority::Authoritative,
        aspect.clone(),
        CanonicalFieldPath::single(FieldKey::new(name).unwrap()),
    )
}

#[test]
fn exact_field_change_preserves_sibling_and_other_occurrence() {
    let aspect = AspectKey::new("frame").unwrap();
    let width = field(&aspect, "width");
    let height = field(&aspect, "height");
    let entries = [
        (
            1,
            BTreeSet::from([ViewDependency::Field(entity(2), width.clone())]),
        ),
        (
            2,
            BTreeSet::from([ViewDependency::Field(entity(2), height.clone())]),
        ),
        (
            3,
            BTreeSet::from([ViewDependency::Field(entity(3), width.clone())]),
        ),
        (
            4,
            BTreeSet::from([ViewDependency::Aspect(entity(2), aspect.clone())]),
        ),
    ];
    let index = DependencyIndex::build(
        &BTreeSet::from([ViewDependency::Entity(entity(1))]),
        entries
            .iter()
            .map(|(key, dependencies)| (key, dependencies)),
    );
    let affected = index
        .affected(
            &[ViewChange::Field(
                entity(2),
                aspect.clone(),
                width.field_path().clone(),
            )],
            16,
        )
        .unwrap();
    assert_eq!(affected.entries, BTreeSet::from([1, 4]));
    assert!(!affected.membership);
    assert!(index
        .affected(
            &[ViewChange::Field(
                entity(2),
                aspect,
                height.field_path().clone()
            )],
            1
        )
        .is_none());
}

#[test]
fn structural_changes_include_all_fields_membership_and_both_endpoint_directions() {
    let aspect = AspectKey::new("frame").unwrap();
    let kind = KindId::new(7);
    let entries = [
        (
            1,
            BTreeSet::from([ViewDependency::Field(entity(2), field(&aspect, "width"))]),
        ),
        (
            2,
            BTreeSet::from([ViewDependency::Adjacency(entity(3), kind, 1)]),
        ),
    ];
    let index = DependencyIndex::build(
        &BTreeSet::from([ViewDependency::Adjacency(entity(1), kind, 0)]),
        entries
            .iter()
            .map(|(key, dependencies)| (key, dependencies)),
    );
    assert_eq!(
        index
            .affected(&[ViewChange::Aspect(entity(2), aspect)], 16)
            .unwrap()
            .entries,
        BTreeSet::from([1])
    );
    let affected = index
        .affected(
            &[
                ViewChange::Adjacency(entity(1), kind, 0),
                ViewChange::Adjacency(entity(3), kind, 1),
            ],
            16,
        )
        .unwrap();
    assert!(affected.membership);
    assert_eq!(affected.entries, BTreeSet::from([2]));
    assert_eq!(
        index
            .affected(&[ViewChange::Entity(entity(2))], 16)
            .unwrap()
            .entries,
        BTreeSet::from([1])
    );
}

#[test]
fn replacing_an_entry_dependency_removes_the_old_body_set_edge() {
    let old = ViewDependency::Entity(entity(2));
    let new = ViewDependency::Entity(entity(3));
    let membership = BTreeSet::from([ViewDependency::Entity(entity(1))]);
    let mut index = DependencyIndex::build(&membership, [(&7, &BTreeSet::from([old.clone()]))]);
    index.remove_entry(&7, &BTreeSet::from([old]));
    index.insert_entry(&new, &7);
    assert!(index
        .affected(&[ViewChange::Entity(entity(2))], 16)
        .unwrap()
        .entries
        .is_empty());
    assert_eq!(
        index
            .affected(&[ViewChange::Entity(entity(3))], 16)
            .unwrap()
            .entries,
        BTreeSet::from([7])
    );
}

#[test]
fn replacing_aspect_dependency_removes_old_aspect_edge() {
    let aspect = AspectKey::new("body").unwrap();
    let old = ViewDependency::Aspect(entity(2), aspect.clone());
    let new = ViewDependency::Aspect(entity(3), aspect.clone());
    let membership = BTreeSet::from([ViewDependency::Entity(entity(1))]);
    let mut index = DependencyIndex::build(&membership, [(&7, &BTreeSet::from([old.clone()]))]);
    index.remove_entry(&7, &BTreeSet::from([old]));
    index.insert_entry(&new, &7);
    assert!(index
        .affected(&[ViewChange::Aspect(entity(2), aspect.clone())], 16)
        .unwrap()
        .entries
        .is_empty());
    assert_eq!(
        index
            .affected(&[ViewChange::Aspect(entity(3), aspect)], 16)
            .unwrap()
            .entries,
        BTreeSet::from([7])
    );
}
