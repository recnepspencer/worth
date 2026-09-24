use std::collections::{BTreeMap, BTreeSet};

use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, CanonicalFieldPath, LocatorAuthority,
};
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::{EntityId, KindId};
use worth_runtime_world::facade::{
    CompositeCommitIdentity, ProductBranchIdentity, ProductBranchIncarnation,
};

use super::dependency::ViewDependency;

/// Canonical changes are translated to these dependency-level events before
/// publication. A field event also changes aspect readers; a whole-aspect
/// event changes every field reader of that aspect.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum ViewChange {
    Entity(EntityId),
    Aspect(EntityId, AspectKey),
    Field(EntityId, AspectKey, CanonicalFieldPath),
    Adjacency(EntityId, KindId, u8),
}

pub(in crate::domain_computation::primary_graph) struct ViewPublicationBasis<'a> {
    pub before: &'a CompositeCommitIdentity,
    pub relational_branch: &'a BranchId,
    pub product_branch: &'a ProductBranchIdentity,
    pub incarnation: ProductBranchIncarnation,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Target<Key> {
    Membership,
    Entry(Key),
}

pub(super) struct DependencyIndex<Key> {
    exact: BTreeMap<ViewDependency, BTreeSet<Target<Key>>>,
    entity: BTreeMap<EntityId, BTreeSet<Target<Key>>>,
    aspect: BTreeMap<(EntityId, AspectKey), BTreeSet<Target<Key>>>,
}

impl<Key> Default for DependencyIndex<Key> {
    fn default() -> Self {
        Self {
            exact: BTreeMap::new(),
            entity: BTreeMap::new(),
            aspect: BTreeMap::new(),
        }
    }
}

impl<Key: Clone + Ord> DependencyIndex<Key> {
    pub(super) fn build<'a>(
        membership: &BTreeSet<ViewDependency>,
        entries: impl IntoIterator<Item = (&'a Key, &'a BTreeSet<ViewDependency>)>,
    ) -> Self
    where
        Key: 'a,
    {
        let mut index = Self::default();
        for dependency in membership {
            index.insert(dependency, Target::Membership);
        }
        for (key, dependencies) in entries {
            for dependency in dependencies {
                index.insert(dependency, Target::Entry(key.clone()));
            }
        }
        index
    }

    fn insert(&mut self, dependency: &ViewDependency, target: Target<Key>) {
        self.exact
            .entry(dependency.clone())
            .or_default()
            .insert(target.clone());
        let entity = match dependency {
            ViewDependency::Entity(entity)
            | ViewDependency::Aspect(entity, _)
            | ViewDependency::Field(entity, _)
            | ViewDependency::Adjacency(entity, _, _) => *entity,
        };
        self.entity
            .entry(entity)
            .or_default()
            .insert(target.clone());
        match dependency {
            ViewDependency::Aspect(_, aspect) => {
                self.aspect
                    .entry((entity, aspect.clone()))
                    .or_default()
                    .insert(target);
            }
            ViewDependency::Field(_, locator) => {
                self.aspect
                    .entry((entity, locator.aspect().aspect_key().clone()))
                    .or_default()
                    .insert(target);
            }
            _ => {}
        }
    }

    pub(super) fn insert_entry(&mut self, dependency: &ViewDependency, key: &Key) {
        self.insert(dependency, Target::Entry(key.clone()));
    }

    pub(super) fn remove_entry(&mut self, key: &Key, dependencies: &BTreeSet<ViewDependency>) {
        let target = Target::Entry(key.clone());
        for dependency in dependencies {
            if let Some(targets) = self.exact.get_mut(dependency) {
                targets.remove(&target);
                if targets.is_empty() {
                    self.exact.remove(dependency);
                }
            }
            let entity = match dependency {
                ViewDependency::Entity(entity)
                | ViewDependency::Aspect(entity, _)
                | ViewDependency::Field(entity, _)
                | ViewDependency::Adjacency(entity, _, _) => *entity,
            };
            if let Some(targets) = self.entity.get_mut(&entity) {
                targets.remove(&target);
                if targets.is_empty() {
                    self.entity.remove(&entity);
                }
            }
            let aspect = match dependency {
                ViewDependency::Aspect(_, aspect) => Some(aspect.clone()),
                ViewDependency::Field(_, locator) => Some(locator.aspect().aspect_key().clone()),
                _ => None,
            };
            if let Some(aspect) = aspect {
                let axis = (entity, aspect);
                if let Some(targets) = self.aspect.get_mut(&axis) {
                    targets.remove(&target);
                    if targets.is_empty() {
                        self.aspect.remove(&axis);
                    }
                }
            }
        }
    }

    pub(super) fn entry_insertion_bound(dependency: &ViewDependency) -> usize {
        dependency
            .retained_bytes()
            .saturating_mul(3)
            .saturating_add(std::mem::size_of::<Target<Key>>().saturating_mul(3))
            .saturating_add(128)
    }

    pub(super) fn affected(
        &self,
        changes: &[ViewChange],
        maximum_work_units: usize,
    ) -> Option<AffectedTargets<Key>> {
        let mut affected = AffectedTargets::default();
        let mut work = 0usize;
        for change in changes {
            work = work.checked_add(1)?;
            if work > maximum_work_units {
                return None;
            }
            match change {
                ViewChange::Entity(entity) => {
                    affected.extend(self.entity.get(entity), &mut work, maximum_work_units)?;
                }
                ViewChange::Aspect(entity, aspect) => {
                    affected.extend(
                        self.aspect.get(&(*entity, aspect.clone())),
                        &mut work,
                        maximum_work_units,
                    )?;
                }
                ViewChange::Field(entity, aspect, path) => {
                    affected.extend(
                        self.exact.get(&ViewDependency::Field(
                            *entity,
                            AspectFieldLocator::new(
                                LocatorAuthority::Authoritative,
                                aspect.clone(),
                                path.clone(),
                            ),
                        )),
                        &mut work,
                        maximum_work_units,
                    )?;
                    affected.extend(
                        self.exact
                            .get(&ViewDependency::Aspect(*entity, aspect.clone())),
                        &mut work,
                        maximum_work_units,
                    )?;
                }
                ViewChange::Adjacency(entity, kind, direction) => {
                    affected.extend(
                        self.exact
                            .get(&ViewDependency::Adjacency(*entity, *kind, *direction)),
                        &mut work,
                        maximum_work_units,
                    )?;
                }
            }
        }
        affected.work_units = work;
        Some(affected)
    }
}

pub(super) struct AffectedTargets<Key> {
    pub(super) membership: bool,
    pub(super) entries: BTreeSet<Key>,
    pub(super) work_units: usize,
}

impl<Key> Default for AffectedTargets<Key> {
    fn default() -> Self {
        Self {
            membership: false,
            entries: BTreeSet::new(),
            work_units: 0,
        }
    }
}

impl<Key: Clone + Ord> AffectedTargets<Key> {
    fn extend(
        &mut self,
        targets: Option<&BTreeSet<Target<Key>>>,
        work: &mut usize,
        maximum_work_units: usize,
    ) -> Option<()> {
        if let Some(targets) = targets {
            for target in targets {
                *work = work.checked_add(1)?;
                if *work > maximum_work_units {
                    return None;
                }
                match target {
                    Target::Membership => self.membership = true,
                    Target::Entry(key) => {
                        self.entries.insert(key.clone());
                    }
                }
            }
        }
        Some(())
    }
}

pub(super) struct PreparedViewTransition<Key> {
    pub(super) before: CompositeCommitIdentity,
    pub(super) revision: u64,
    pub(super) affected: AffectedTargets<Key>,
    pub(super) work_units: usize,
}

#[cfg(test)]
mod tests {
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
                    aspect.clone(),
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
        let affected = index
            .affected(&[ViewChange::Aspect(entity(2), aspect)], 16)
            .unwrap();
        assert_eq!(affected.entries, BTreeSet::from([1]));
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
        let affected = index
            .affected(&[ViewChange::Entity(entity(2))], 16)
            .unwrap();
        assert_eq!(affected.entries, BTreeSet::from([1]));
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
}
