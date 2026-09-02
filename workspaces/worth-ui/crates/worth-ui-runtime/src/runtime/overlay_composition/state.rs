use std::collections::{BTreeMap, BTreeSet};

use worth_ui_dsl::UiBackdropDeclaration;

use super::dependency_index::{
    UiOverlayAffectedScope, UiOverlayChangeSet, UiOverlayDependencyIndex,
};
use super::extent::UiOverlayMotionSnapshot;
use super::full_rebuild::compile_full;
use super::materialization::{current_portals, ensure_backdrop_capacity, materialize_one, reserve};
use super::order::compile_order;
use super::planner::{
    UiOverlayCompositionDenial, UiOverlayCompositionInput, UiOverlayCompositionState,
    UiOverlayPlanCounters, UiOverlayReservation, UiPreparedOverlayComposition,
};
use super::relation_cache::{self, UiOverlayRelationCache};
use super::snapshot::{
    UiBackdropInstanceIdentity, UiOverlayBackdropRow, UiOverlayStackParticipant,
    UiOverlayStackSnapshot,
};

impl UiOverlayCompositionState {
    pub(crate) fn admit(
        declarations: impl IntoIterator<Item = UiBackdropDeclaration>,
        declaration_revision: u64,
        capacity: super::planner::UiOverlayCapacityProfile,
    ) -> Result<Self, UiOverlayCompositionDenial> {
        if declaration_revision == 0 {
            return Err(UiOverlayCompositionDenial::InvalidDeclarationRevision);
        }
        let declarations = declarations.into_iter().collect::<Vec<_>>();
        if declarations.len() > capacity.max_backdrop_declarations {
            return Err(
                UiOverlayCompositionDenial::BackdropDeclarationCapacityExceeded {
                    observed: declarations.len(),
                    maximum: capacity.max_backdrop_declarations,
                },
            );
        }
        let index = UiOverlayDependencyIndex::rebuild(&declarations)
            .map_err(|()| UiOverlayCompositionDenial::DuplicateBackdropIdentity)?;
        let relations =
            relation_cache::build(&declarations).map_err(UiOverlayCompositionDenial::Relation)?;
        Ok(Self {
            declarations: declarations.into_boxed_slice(),
            declaration_revision,
            capacity,
            index: Some(index),
            relations: Some(relations),
            current: None,
        })
    }

    pub(crate) fn current(&self) -> Option<&UiOverlayStackSnapshot> {
        self.current.as_ref()
    }

    pub(crate) fn dependency_index(&self) -> Option<&UiOverlayDependencyIndex> {
        self.index.as_ref()
    }

    #[cfg(test)]
    pub(crate) fn discard_index_for_test(&mut self) {
        self.index = None;
    }

    #[cfg(test)]
    pub(crate) fn discard_relation_cache_for_test(&mut self) {
        self.relations = None;
    }

    pub(crate) fn prepare_initial(
        &self,
        input: UiOverlayCompositionInput<'_>,
    ) -> Result<UiPreparedOverlayComposition, UiOverlayCompositionDenial> {
        let index = self
            .index
            .clone()
            .ok_or(UiOverlayCompositionDenial::ReconstructionRequired)?;
        let relations = self
            .relations
            .clone()
            .ok_or(UiOverlayCompositionDenial::ReconstructionRequired)?;
        let (snapshot, reservation, counters) = compile_full(self, &input, &relations)?;
        Ok(UiPreparedOverlayComposition {
            predecessor: None,
            snapshot,
            reservation,
            counters,
            index,
            relations,
        })
    }

    pub(crate) fn prepare_successor(
        &self,
        input: UiOverlayCompositionInput<'_>,
        changes: &UiOverlayChangeSet,
    ) -> Result<UiPreparedOverlayComposition, UiOverlayCompositionDenial> {
        let predecessor = self
            .current
            .clone()
            .ok_or(UiOverlayCompositionDenial::NoCurrentSnapshot)?;
        if !same_world(&predecessor, &input) {
            return Err(UiOverlayCompositionDenial::ReconstructionRequired);
        }
        if changes.has_declaration_change() {
            return Err(UiOverlayCompositionDenial::ReconstructionRequired);
        }
        let index = self
            .index
            .as_ref()
            .ok_or(UiOverlayCompositionDenial::ReconstructionRequired)?;
        let relations = self
            .relations
            .clone()
            .ok_or(UiOverlayCompositionDenial::ReconstructionRequired)?;
        let scope = index.affected_scope(changes);
        let (snapshot, reservation, counters) = if changes.has_structural_change() {
            self.compile_successor(&input, &predecessor, &scope, &relations)?
        } else {
            self.update_nonstructural(&input, &predecessor, &scope, changes)?
        };
        Ok(UiPreparedOverlayComposition {
            predecessor: Some(predecessor),
            snapshot,
            reservation,
            counters,
            index: index.clone(),
            relations,
        })
    }

    pub(crate) fn reconstruct(
        &self,
        input: UiOverlayCompositionInput<'_>,
    ) -> Result<UiPreparedOverlayComposition, UiOverlayCompositionDenial> {
        let index = UiOverlayDependencyIndex::rebuild(&self.declarations)
            .map_err(|()| UiOverlayCompositionDenial::DuplicateBackdropIdentity)?;
        let relations = relation_cache::build(&self.declarations)
            .map_err(UiOverlayCompositionDenial::Relation)?;
        let (snapshot, reservation, counters) = compile_full(self, &input, &relations)?;
        Ok(UiPreparedOverlayComposition {
            predecessor: self.current.clone(),
            snapshot,
            reservation,
            counters,
            index,
            relations,
        })
    }

    pub(crate) fn publish(
        &mut self,
        prepared: UiPreparedOverlayComposition,
    ) -> Result<(), super::planner::UiOverlayCommitDenial> {
        if self.current != prepared.predecessor {
            return Err(super::planner::UiOverlayCommitDenial::StalePredecessor);
        }
        self.current = Some(prepared.snapshot);
        self.index = Some(prepared.index);
        self.relations = Some(prepared.relations);
        Ok(())
    }

    fn compile_successor(
        &self,
        input: &UiOverlayCompositionInput<'_>,
        predecessor: &UiOverlayStackSnapshot,
        scope: &UiOverlayAffectedScope,
        relation_cache: &UiOverlayRelationCache,
    ) -> Result<
        (
            UiOverlayStackSnapshot,
            UiOverlayReservation,
            UiOverlayPlanCounters,
        ),
        UiOverlayCompositionDenial,
    > {
        let portal_rows = current_portals(input, self.capacity)?;
        let portal_stack_rows_read = portal_rows.source_rows_read();
        let portal_binding_entries_read = portal_rows.binding_entries_read();
        let portals = portal_rows.rows();
        let relations =
            relation_cache::for_surface(relation_cache, input.extent.declaration_surface());
        let affected = scope
            .backdrops()
            .iter()
            .map(|backdrop| backdrop.identity())
            .collect::<BTreeSet<_>>();
        let mut previous_affected = BTreeMap::new();
        let mut backdrops = Vec::new();
        for participant in predecessor.participants() {
            let UiOverlayStackParticipant::Backdrop(row) = participant else {
                continue;
            };
            if affected.contains(&row.declaration()) {
                previous_affected.insert(row.identity(), row.clone());
            } else {
                backdrops.push(row.clone());
            }
        }
        let mut changed = 0;
        for identity in &affected {
            let declaration = self
                .index
                .as_ref()
                .and_then(|index| index.declaration_index(*identity))
                .and_then(|index| self.declarations.get(index))
                .ok_or(UiOverlayCompositionDenial::ReconstructionRequired)?;
            if declaration.surface() == input.extent.declaration_surface() {
                for row in materialize_one(declaration, &portals, input.extent, input.motion)? {
                    if previous_affected
                        .remove(&row.identity())
                        .is_none_or(|previous| previous != row)
                    {
                        changed += 1;
                    }
                    backdrops.push(row);
                }
                ensure_backdrop_capacity(backdrops.len(), self.capacity)?;
            }
        }
        changed += previous_affected.len();
        let (participants, relation_edges) = compile_order(&portals, &backdrops, &relations)?;
        let reservation = reserve(
            portals.len(),
            backdrops.len(),
            participants.len(),
            relation_edges,
            self.capacity,
        )?;
        let snapshot = UiOverlayStackSnapshot::seal(
            input.generation.clone(),
            input.extent.declaration_surface(),
            input.extent.runtime_surface(),
            input.presentation,
            input.portal_snapshot.owner_revision(),
            self.declaration_revision,
            input.extent.revision(),
            input.motion.map(UiOverlayMotionSnapshot::owner_revision),
            participants.clone(),
        );
        Ok((
            snapshot,
            reservation,
            UiOverlayPlanCounters {
                portal_stack_rows_read,
                portal_binding_entries_read,
                backdrop_declarations_selected: affected.len(),
                overlay_relation_edges_visited: relation_edges,
                backdrop_mechanics_changed: changed,
                ..UiOverlayPlanCounters::default()
            },
        ))
    }

    fn update_nonstructural(
        &self,
        input: &UiOverlayCompositionInput<'_>,
        predecessor: &UiOverlayStackSnapshot,
        scope: &UiOverlayAffectedScope,
        changes: &UiOverlayChangeSet,
    ) -> Result<
        (
            UiOverlayStackSnapshot,
            UiOverlayReservation,
            UiOverlayPlanCounters,
        ),
        UiOverlayCompositionDenial,
    > {
        let portal_rows = current_portals(input, self.capacity)?;
        let portal_stack_rows_read = portal_rows.source_rows_read();
        let portal_binding_entries_read = portal_rows.binding_entries_read();
        let portals = portal_rows.rows();
        let old_portals = predecessor
            .participants()
            .iter()
            .filter_map(|participant| match participant {
                UiOverlayStackParticipant::Portal(row) => Some(row),
                UiOverlayStackParticipant::Backdrop(_) => None,
            })
            .collect::<Vec<_>>();
        if old_portals.len() != portals.len()
            || old_portals.iter().zip(&portals).any(|(old, new)| {
                old.portal() != new.portal()
                    || old.declaration() != new.declaration()
                    || old.parent() != new.parent()
                    || old.ordinal() != new.ordinal()
                    || old.lifecycle() != new.lifecycle()
            })
            || predecessor.portal_revision() != input.portal_snapshot.owner_revision()
            || (predecessor.motion_revision()
                != input.motion.map(UiOverlayMotionSnapshot::owner_revision)
                && !changes.has_motion_change())
            || (predecessor.extent_revision() != input.extent.revision()
                && !changes.has_extent_change())
        {
            return Err(UiOverlayCompositionDenial::ReconstructionRequired);
        }
        let mut replacements = BTreeMap::<UiBackdropInstanceIdentity, UiOverlayBackdropRow>::new();
        for affected in scope.backdrops() {
            let declaration = self
                .index
                .as_ref()
                .and_then(|index| index.declaration_index(affected.identity()))
                .and_then(|index| self.declarations.get(index))
                .ok_or(UiOverlayCompositionDenial::ReconstructionRequired)?;
            if declaration.surface() != input.extent.declaration_surface() {
                continue;
            }
            for row in materialize_one(declaration, &portals, input.extent, input.motion)? {
                replacements.insert(row.identity(), row);
                ensure_backdrop_capacity(replacements.len(), self.capacity)?;
            }
        }
        let mut changed = 0;
        let participants = predecessor
            .participants()
            .iter()
            .map(|participant| match participant {
                UiOverlayStackParticipant::Portal(row) => {
                    UiOverlayStackParticipant::Portal(row.clone())
                }
                UiOverlayStackParticipant::Backdrop(row) => {
                    replacements.get(&row.identity()).map_or_else(
                        || UiOverlayStackParticipant::Backdrop(row.clone()),
                        |replacement| {
                            if replacement != row {
                                changed += 1;
                            }
                            UiOverlayStackParticipant::Backdrop(replacement.clone())
                        },
                    )
                }
            })
            .collect::<Vec<_>>();
        let backdrop_count = participants
            .iter()
            .filter(|participant| matches!(participant, UiOverlayStackParticipant::Backdrop(_)))
            .count();
        let reservation = reserve(
            portals.len(),
            backdrop_count,
            participants.len(),
            0,
            self.capacity,
        )?;
        let snapshot = UiOverlayStackSnapshot::seal(
            input.generation.clone(),
            input.extent.declaration_surface(),
            input.extent.runtime_surface(),
            input.presentation,
            input.portal_snapshot.owner_revision(),
            self.declaration_revision,
            input.extent.revision(),
            input.motion.map(UiOverlayMotionSnapshot::owner_revision),
            participants,
        );
        Ok((
            snapshot,
            reservation,
            UiOverlayPlanCounters {
                portal_stack_rows_read,
                portal_binding_entries_read,
                backdrop_declarations_selected: scope.backdrops().len(),
                backdrop_mechanics_changed: changed,
                ..UiOverlayPlanCounters::default()
            },
        ))
    }
}

fn same_world(predecessor: &UiOverlayStackSnapshot, input: &UiOverlayCompositionInput<'_>) -> bool {
    predecessor.generation() == &input.generation
        && predecessor.declaration_surface() == input.extent.declaration_surface()
        && predecessor.runtime_surface() == input.extent.runtime_surface()
}
