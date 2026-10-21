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
    ) -> Result<Self, CompanionPreflightStop> {
        let mut by_entity = OrdSet::<RoleKey>::new();
        for (position, entity) in entities.enumerate() {
            by_entity.insert((entity, position));
        }
        Ok(Self { by_entity })
    }

    /// The roles naming `entity`. Both range ends descend the tree; each
    /// yielded role is a visit its caller charges.
    fn roles_of<'index>(
        &'index self,
        entity: EntityId,
    ) -> Result<impl Iterator<Item = usize> + use<'index>, CompanionPreflightStop> {
        Ok(self
            .by_entity
            .range((entity, 0)..=(entity, usize::MAX))
            .map(|(_, position)| *position))
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
    ) -> Result<bool, CompanionPreflightStop> {
        let mut kind_seen = vec![false; roles.len()];
        for fact in facts.iter() {
            match fact {
                Fact::Entity { entity_id, kind } => {
                    for position in self.roles_of(*entity_id)? {
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
                    for position in self.roles_of(*entity_id)? {
                        let role = &roles[position];
                        for probe in &mut aspects[role.first_aspect..role.end_aspect] {
                            if probe.aspect != *aspect {
                                continue;
                            }

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
        Ok(kind_seen.iter().all(|seen| *seen)
            && aspects.iter().all(|probe| probe.revision.is_some()))
    }
}
