//! A checkpoint's expected roles indexed once by entity.
//!
//! Each original fact is located by its entity with two binary bounds, then
//! compared only against that entity's roles and their installed aspects. F
//! facts against R roles cost O((F + R) log R) navigation plus each matched
//! role's own aspect comparisons, never a rescan of every fact per role and
//! per aspect.

use crate::domain_computation::primary_graph::application_checkpoint::facts::FactSequence;

use super::*;

/// A role's entity beside its position. Roles naming one entity stay distinct.
type RoleKey = (EntityId, usize);

pub(super) struct CheckpointRoleIndex {
    by_entity: Vec<RoleKey>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl CheckpointRoleIndex {
    /// The flat index is allocated once and sorted in place, so its
    /// complete build is admitted before the first insertion.
    pub(super) fn build(
        execution: Option<&worth_execution::ExecutionRequest<'_, '_>>,
        visit: &mut dyn FnMut(u64) -> Result<(), CompanionPreflightStop>,
        entities: impl ExactSizeIterator<Item = EntityId>,
    ) -> Result<Self, CompanionPreflightStop> {
        let count = entities.len();
        let depth = count.saturating_add(1).checked_ilog2().unwrap_or(0) as u64 + 1;
        visit((count as u64).checked_mul(depth + 1).ok_or_else(overflow)?)?;
        let bytes = count
            .checked_mul(std::mem::size_of::<RoleKey>() + std::mem::size_of::<bool>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(overflow)?;
        let memory = match execution {
            Some(execution) => {
                crate::domain_computation::primary_graph::structural_walk::memory::reserve(
                    execution,
                    execution.has_accounting(),
                    bytes,
                )
                .map_err(CompanionPreflightStop::Execution)?
            }
            None => None,
        };
        let mut by_entity = Vec::with_capacity(count);
        for (position, entity) in entities.enumerate() {
            by_entity.push((entity, position));
        }
        by_entity.sort_unstable();
        Ok(Self {
            by_entity,
            _memory: memory,
        })
    }

    /// The roles naming `entity`. Both range ends descend the tree; each
    /// yielded role is a visit its caller charges.
    fn roles_of<'index>(
        &'index self,
        entity: EntityId,
        visit: &mut dyn FnMut(u64) -> Result<(), CompanionPreflightStop>,
    ) -> Result<impl Iterator<Item = usize> + use<'index>, CompanionPreflightStop> {
        let depth = self
            .by_entity
            .len()
            .saturating_add(1)
            .checked_ilog2()
            .unwrap_or(0) as u64
            + 1;
        visit(depth * 2)?;
        let start = self.by_entity.partition_point(|(id, _)| *id < entity);
        let end = self.by_entity.partition_point(|(id, _)| *id <= entity);
        Ok(self.by_entity[start..end]
            .iter()
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
        visit: &mut dyn FnMut(u64) -> Result<(), CompanionPreflightStop>,
    ) -> Result<bool, CompanionPreflightStop> {
        let mut kind_seen = vec![false; roles.len()];
        for fact in facts.iter() {
            visit(1)?;
            match fact {
                Fact::Entity { entity_id, kind } => {
                    for position in self.roles_of(*entity_id, visit)? {
                        visit(1)?;
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
                    for position in self.roles_of(*entity_id, visit)? {
                        visit(1)?;
                        let role = &roles[position];
                        for probe in &mut aspects[role.first_aspect..role.end_aspect] {
                            visit(
                                probe.aspect.as_str().len().max(aspect.as_str().len()) as u64 + 1,
                            )?;
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
        visit((kind_seen.len() + aspects.len()) as u64)?;
        Ok(kind_seen.iter().all(|seen| *seen)
            && aspects.iter().all(|probe| probe.revision.is_some()))
    }
}
