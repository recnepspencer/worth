//! Phase 5 persistent-name continuity for owned-graph selection: the Native
//! commit behind `HistoryDocument` `undo_step`, `redo_to` and `checkout`.
//!
//! A selection rematerializes historical records at their exact original
//! full-generation identities and retires the current graph's other members.
//! Relational owns both sides of that transition, so it alone derives the
//! continuity from the canonical selection and the target branch's naming
//! basis; no caller supplies a naming plan:
//!
//! 1. A retired record holding a live name emits `Retire`.
//! 2. A restored record whose name is retired on this branch emits `Restore`
//!    from the name's last retired lineage onto a fresh lineage bound to the
//!    record's exact original identity.
//! 3. A restored record whose name is live on a record this selection retires
//!    continues the name in-commit (`Create` with the kept name + `Replace`),
//!    never a false `Retire`.
//! 4. A restored name still live on a record this selection keeps, or restored
//!    twice, refuses `DuplicateBirth` before any effect.
//!
//! 5. A restored record this branch never named, of a kind with an installed
//!    scheme, refuses `MissingBirth`: it would otherwise go live unnamed.
//!
//! Other records without a name on this branch are untouched, so unnamed and
//! schemaless selections emit exactly the events they emitted before.

use std::collections::{BTreeMap, BTreeSet};

use crate::history::data::CanonicalOwnedGraphSelection;
use crate::identity::data::{EntityId, LineageId, RelationId, VersionId};
use crate::lineage::data::{
    LineageBasisView, LineageNameDenial, LineagePostureKind, LineageRecordTarget,
    PersistentNameHolder, PersistentNameIdentity,
};
use crate::runtime::WorkingState;

use super::posture_validation::{mismatch, ValidatedPosture, ValidatedPostureBatch};
use super::staged_effects::{
    basis_lineage_id, entity_lineage, is_created_in_commit, target_kind_id,
};
use crate::schema::data::RelationalSchemaRegistry;
use crate::transactions::data::TransactionCommitError;

/// A named entity kept live in place whose selected slot carries another
/// lineage; the branch's live lineage is reinstalled so later retirements
/// and continuity read the branch's own lineage, never the source's.
pub(super) struct SelectionLineageRepair {
    pub(super) entity_id: EntityId,
    pub(super) lineage_id: LineageId,
}

/// Appends the selection's derived continuity postures to `validated` and
/// returns the in-place lineage repairs; no selection derives nothing. Pure
/// and fail-closed.
///
/// A selection derives its continuity alone: any caller posture beside it
/// would claim the same records twice (a doubled `Retire` ends a lineage
/// twice and poisons the branch name index), so it refuses before effects.
///
/// Complexity: O((E + R) log (E + R + N)) for E selected/current entities,
/// R relations and N branch names.
pub(super) fn selection_postures(
    staged: &WorkingState,
    basis: &dyn LineageBasisView,
    schema: Option<&RelationalSchemaRegistry>,
    selection: Option<&CanonicalOwnedGraphSelection>,
    validated: &mut ValidatedPostureBatch,
) -> Result<Vec<SelectionLineageRepair>, TransactionCommitError> {
    let Some(selection) = selection else {
        return Ok(Vec::new());
    };
    let transition = SelectionTransition::of(selection);
    if let Some(refusal) = caller_plan_refusal(validated, &transition, selection.model()) {
        return Err(TransactionCommitError::lineage_name_denial(refusal));
    }
    plan(staged, basis, schema, &transition, validated)
        .map_err(TransactionCommitError::lineage_name_denial)
}

fn caller_plan_refusal(
    validated: &ValidatedPostureBatch,
    transition: &SelectionTransition,
    model: EntityId,
) -> Option<LineageNameDenial> {
    if validated.postures.is_empty() && validated.minted_births.is_empty() {
        return None;
    }
    let target = validated
        .claimed_basis_targets
        .iter()
        .chain(&validated.claimed_created_targets)
        .chain(&transition.retired)
        .chain(&transition.restored)
        .next()
        .copied()
        .unwrap_or(LineageRecordTarget::Entity(model));
    Some(mismatch(
        LineagePostureKind::Restore,
        target,
        "an owned-graph selection derives its own continuity and admits no naming plan",
    ))
}

/// The record transitions one selection performs, read from its canonical
/// meaning: records it retires, records it rematerializes, and entities it
/// keeps live in place.
pub(super) struct SelectionTransition {
    pub(super) retired: Vec<LineageRecordTarget>,
    pub(super) restored: Vec<LineageRecordTarget>,
    pub(super) kept_entities: Vec<EntityId>,
}

impl SelectionTransition {
    fn of(selection: &CanonicalOwnedGraphSelection) -> Self {
        let current: BTreeSet<EntityId> = selection.current_entities().iter().copied().collect();
        let current_relations: BTreeSet<RelationId> = selection
            .current_relations()
            .iter()
            .map(|edge| edge.relation_id)
            .collect();
        let (kept_entities, restored_entities): (Vec<EntityId>, Vec<EntityId>) = selection
            .selected_entities()
            .map(|(id, _)| id)
            .partition(|id| current.contains(id));
        Self {
            retired: selection
                .retired_entities()
                .map(LineageRecordTarget::Entity)
                .chain(
                    selection
                        .retired_relations()
                        .map(LineageRecordTarget::Relation),
                )
                .collect(),
            restored: restored_entities
                .into_iter()
                .map(LineageRecordTarget::Entity)
                .chain(
                    selection
                        .selected_relations()
                        .iter()
                        .map(|edge| edge.relation_id)
                        .filter(|id| !current_relations.contains(id))
                        .map(LineageRecordTarget::Relation),
                )
                .collect(),
            kept_entities,
        }
    }
}

pub(super) fn plan(
    staged: &WorkingState,
    basis: &dyn LineageBasisView,
    schema: Option<&RelationalSchemaRegistry>,
    transition: &SelectionTransition,
    validated: &mut ValidatedPostureBatch,
) -> Result<Vec<SelectionLineageRepair>, LineageNameDenial> {
    let mut retiring = BTreeMap::<PersistentNameIdentity, (LineageRecordTarget, LineageId)>::new();
    for &target in &transition.retired {
        let Some(name) = live_name(basis, &target.to_holder()) else {
            continue;
        };
        // The branch naming basis, not the slot, is authoritative for a live
        // named holder: a selected slot may carry its source's lineage.
        let lineage = basis
            .live_lineage(&target.to_holder())
            .or_else(|| basis_lineage_id(staged, basis, target))
            .ok_or_else(|| {
                mismatch(
                    LineagePostureKind::Retire,
                    target,
                    "selection retires a named record without a live branch lineage",
                )
            })?;
        retiring.insert(name, (target, lineage));
    }

    let mut restored_names = BTreeMap::<PersistentNameIdentity, PersistentNameHolder>::new();
    for &target in &transition.restored {
        let holder = target.to_holder();
        let Some(name) = basis.holder_name(&holder) else {
            // A record this branch never named may still be of a schemed
            // kind (selected from a sibling branch): it would go live
            // unnamed, so it refuses like any unnamed schemed birth.
            if let Some(scheme_id) = target_kind_id(staged, target).and_then(|kind| {
                schema.and_then(|schema| schema.scheme_id_for_kind(target.subject_kind(), kind))
            }) {
                return Err(LineageNameDenial::MissingBirth {
                    target,
                    scheme_id: scheme_id.to_owned(),
                });
            }
            continue;
        };
        if let Some(first) = restored_names.insert(name, holder.clone()) {
            return Err(LineageNameDenial::DuplicateBirth {
                identity: name,
                first,
                second: holder,
            });
        }
        let posture = match basis.live_holder(&name) {
            Some(live) if *live == holder => {
                return Err(mismatch(
                    LineagePostureKind::Restore,
                    target,
                    "selection restores a record whose name it already holds live",
                ))
            }
            Some(live) => {
                let Some((retired_target, source_lineage)) = retiring.remove(&name) else {
                    return Err(LineageNameDenial::DuplicateBirth {
                        identity: name,
                        first: live.clone(),
                        second: holder,
                    });
                };
                if retired_target.subject() != target.subject() {
                    return Err(mismatch(
                        LineagePostureKind::Restore,
                        target,
                        "selection continues a name across subject kinds",
                    ));
                }
                validated.claimed_basis_targets.insert(retired_target);
                ValidatedPosture::Regenerate {
                    subject: target.subject(),
                    source_lineage,
                    target,
                    name,
                }
            }
            None => {
                let (subject, source_lineage) = basis.retired_lineage(&name).ok_or_else(|| {
                    mismatch(
                        LineagePostureKind::Restore,
                        target,
                        "selection restores a name with no retired lineage on this branch",
                    )
                })?;
                if subject != target.subject() {
                    return Err(mismatch(
                        LineagePostureKind::Restore,
                        target,
                        "restored record subject does not match its retired lineage",
                    ));
                }
                ValidatedPosture::Restore {
                    target,
                    source_lineage,
                    name,
                }
            }
        };
        validated.postures.push(posture);
    }
    for (name, (target, source_lineage)) in retiring {
        validated.claimed_basis_targets.insert(target);
        validated.postures.push(ValidatedPosture::Retire {
            subject: target.subject(),
            source_lineage,
            name,
        });
    }

    let mut repairs = Vec::new();
    for &id in &transition.kept_entities {
        let holder = PersistentNameHolder::Entity(id);
        if live_name(basis, &holder).is_none() {
            continue;
        }
        if let Some(lineage_id) = basis.live_lineage(&holder)
            && entity_lineage(staged, id) != Some(lineage_id)
        {
            repairs.push(SelectionLineageRepair {
                entity_id: id,
                lineage_id,
            });
        }
    }
    Ok(repairs)
}

/// The entity an allocating event must name explicitly: one its commit did
/// not create (a selection rematerialized it at its original identity), so
/// no `Created` patch can bind the allocation. Created entities keep binding
/// by allocation order, leaving their events byte-identical.
///
/// Complexity: O(1).
pub(super) fn rematerialized_entity(
    staged: &WorkingState,
    version: VersionId,
    entity_node: Option<(EntityId, LineageId, bool)>,
) -> Option<EntityId> {
    let (entity_id, _, _) = entity_node?;
    (!is_created_in_commit(staged, LineageRecordTarget::Entity(entity_id), version))
        .then_some(entity_id)
}

fn live_name(
    basis: &dyn LineageBasisView,
    holder: &PersistentNameHolder,
) -> Option<PersistentNameIdentity> {
    basis
        .holder_name(holder)
        .filter(|name| basis.live_holder(name) == Some(holder))
}

#[cfg(test)]
#[path = "selection_continuity_tests.rs"]
mod tests;
