//! A checkpoint's expected roles indexed once by entity.
//!
//! Each original fact is located by its entity with one ordered lookup, then
//! compared only against that entity's roles and their installed aspects. F
//! facts against R roles cost O((F + R) log R) navigation plus each matched
//! role's own aspect comparisons, never a rescan of every fact per role and
//! per aspect.

use crate::domain_computation::primary_graph::application_checkpoint::facts::FactSequence;
use im::OrdSet;

use super::*;
use crate::domain_computation::primary_graph::output_lineage::invalidation::{
    insertion_sequence_bytes, ordered_navigation_work,
};

/// A role's entity beside its position. Roles naming one entity stay distinct.
type RoleKey = (EntityId, usize);

pub(super) struct CheckpointRoleIndex {
    by_entity: OrdSet<RoleKey>,
}

impl CheckpointRoleIndex {
    /// The tree is created empty and never shared while it grows, so its
    /// complete build is admitted before the first insertion.
    pub(super) fn build(
        entities: impl ExactSizeIterator<Item = EntityId>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Self, CompanionPreflightStop> {
        admission.admit_read_scratch(
            insertion_sequence_bytes::<RoleKey, ()>(entities.len()).ok_or_else(overflow)?,
        )?;
        let mut by_entity = OrdSet::<RoleKey>::new();
        for (position, entity) in entities.enumerate() {
            admission.charge_external_work(navigation(by_entity.len())?)?;
            by_entity.insert((entity, position));
        }
        Ok(Self { by_entity })
    }

    /// The roles naming `entity`. Both range ends descend the tree; each
    /// yielded role is a visit its caller charges.
    fn roles_of<'index>(
        &'index self,
        entity: EntityId,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<impl Iterator<Item = usize> + use<'index>, CompanionPreflightStop> {
        let descent = navigation(self.by_entity.len())?;
        admission.charge_external_work(
            descent
                .checked_mul(2)
                .ok_or(CompanionPreflightStop::WorkCounterOverflow)?,
        )?;
        Ok(self
            .by_entity
            .range((entity, 0)..=(entity, usize::MAX))
            .map(|(_, position)| *position))
    }

    /// The most roles naming one entity: the per-fact fan-out a ceiling
    /// prices. Ordered keys keep each entity's roles adjacent.
    pub(super) fn widest_entity(
        &self,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<usize, CompanionPreflightStop> {
        let len = u64::try_from(self.by_entity.len())
            .map_err(|_| CompanionPreflightStop::WorkCounterOverflow)?;
        admission.charge_external_work(len)?;
        let (mut widest, mut run, mut previous) = (0, 0, None);
        for (entity, _) in self.by_entity.iter() {
            run = if previous == Some(*entity) {
                run + 1
            } else {
                1
            };
            widest = usize::max(widest, run);
            previous = Some(*entity);
        }
        Ok(widest)
    }

    /// Fill each aspect probe's original revision and confirm each role's
    /// entity kind in one pass. `false` when an entity fact names another
    /// kind, two revisions of one entity aspect disagree, or any role's
    /// entity or aspect has no fact.
    pub(super) fn record_facts(
        &self,
        facts: &(impl FactSequence + ?Sized),
        roles: &[RoleProbe],
        aspects: &mut [AspectProbe],
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, CompanionPreflightStop> {
        admission.admit_read_scratch(
            u64::try_from(roles.len() * size_of::<bool>()).map_err(|_| overflow())?,
        )?;
        let mut kind_seen = vec![false; roles.len()];
        for fact in facts.iter() {
            admission.charge_external_work(1)?;
            match fact {
                Fact::Entity { entity_id, kind } => {
                    for position in self.roles_of(*entity_id, admission)? {
                        admission.charge_external_work(1 + (size_of::<KindId>() * 2) as u64)?;
                        if *kind != roles[position].kind {
                            return Ok(false);
                        }
                        kind_seen[position] = true;
                    }
                }
                Fact::SourceAspectRevision {
                    entity_id,
                    aspect,
                    native_revision,
                } => {
                    for position in self.roles_of(*entity_id, admission)? {
                        admission.charge_external_work(1)?;
                        let role = &roles[position];
                        for probe in &mut aspects[role.first_aspect..role.end_aspect] {
                            let comparison = aspect
                                .as_str()
                                .len()
                                .checked_add(probe.aspect.as_str().len())
                                .and_then(|n| n.checked_add(1))
                                .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
                            admission.charge_external_work(comparison as u64)?;
                            if probe.aspect != *aspect {
                                continue;
                            }
                            admission
                                .charge_external_work((size_of::<Option<u64>>() * 2 + 1) as u64)?;
                            if probe
                                .revision
                                .is_some_and(|revision| revision != *native_revision)
                            {
                                return Ok(false);
                            }
                            probe.revision = Some(*native_revision);
                        }
                    }
                }
                _ => {}
            }
        }
        let expectations = roles
            .len()
            .checked_add(aspects.len())
            .ok_or(CompanionPreflightStop::WorkCounterOverflow)?;
        admission.charge_external_work(expectations as u64)?;
        Ok(kind_seen.iter().all(|seen| *seen)
            && aspects.iter().all(|probe| probe.revision.is_some()))
    }
}

/// One fixed-width key comparison per step of a pinned im descent.
pub(super) fn navigation(entries: usize) -> Result<u64, CompanionPreflightStop> {
    ordered_navigation_work(entries).ok_or(CompanionPreflightStop::WorkCounterOverflow)
}
