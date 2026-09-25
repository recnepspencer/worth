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
    /// Record lifecycle can change, so every dependency on the entity is affected.
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

    pub(super) fn insert_membership(&mut self, dependency: &ViewDependency) {
        self.insert(dependency, Target::Membership);
    }

    pub(super) fn remove_entry(&mut self, key: &Key, dependencies: &BTreeSet<ViewDependency>) {
        self.remove_target(dependencies, Target::Entry(key.clone()));
    }

    pub(super) fn remove_membership(&mut self, dependencies: &BTreeSet<ViewDependency>) {
        self.remove_target(dependencies, Target::Membership);
    }

    fn remove_target(&mut self, dependencies: &BTreeSet<ViewDependency>, target: Target<Key>) {
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
        let axes = if matches!(
            dependency,
            ViewDependency::Aspect(..) | ViewDependency::Field(..)
        ) {
            4
        } else {
            3
        };
        dependency
            .retained_bytes()
            .saturating_mul(axes)
            .saturating_add(std::mem::size_of::<Target<Key>>().saturating_mul(axes))
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
mod tests;
