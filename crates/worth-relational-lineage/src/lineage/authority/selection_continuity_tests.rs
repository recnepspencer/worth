//! The selection planner's continuity classes over an exact branch basis:
//! retire, restore, in-place continuation, live conflicts and slot repair.
use worth_relational_storage::internal_port::storage::substrate::record_arena::arena::slot_lifecycle::RecordArenaFixturePort as _;
use crate::contracts::lineage::data::naming::{PersistentNameHolder, PersistentNameIdentity, PersistentNameIdentityOwnerPort as _};
use std::collections::{BTreeMap, BTreeSet};

use super::{plan, SelectionTransition};
use crate::contracts::config::data::policies::{AdjacencyBackend, AdjacencyPolicy};
use crate::contracts::identity::data::primitives::{KindId, LineageId, PartitionId, VersionId};
use crate::contracts::identity::data::record_ids::{EntityId, RelationId};
use crate::contracts::lineage::data::events::LineageSubject;
use crate::contracts::lineage::data::naming_denial::LineageNameDenial;
use crate::contracts::lineage::data::postures::{LineageBasisView, LineageRecordTarget};
use crate::lineage::authority::posture_validation::{ValidatedPosture, ValidatedPostureBatch};
use worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState;
use worth_relational_storage::internal_port::storage::substrate::record_arena::arena::SlotInit;
use worth_relational_storage::internal_port::storage::substrate::record_arena::kinds::EntityRecordKind;

#[derive(Default)]
struct Basis {
    live: BTreeMap<PersistentNameIdentity, PersistentNameHolder>,
    lineages: BTreeMap<PersistentNameHolder, LineageId>,
    retired: BTreeMap<PersistentNameIdentity, (LineageSubject, LineageId)>,
    names: BTreeMap<PersistentNameHolder, PersistentNameIdentity>,
}

impl Basis {
    fn live(&mut self, holder: PersistentNameHolder, name: PersistentNameIdentity, lineage: u64) {
        self.live.insert(name, holder.clone());
        self.lineages.insert(holder.clone(), LineageId(lineage));
        self.names.insert(holder, name);
    }
    fn retired(
        &mut self,
        holder: PersistentNameHolder,
        name: PersistentNameIdentity,
        lineage: u64,
    ) {
        self.retired
            .insert(name, (LineageSubject::Entity, LineageId(lineage)));
        self.names.insert(holder, name);
    }
}

impl LineageBasisView for Basis {
    fn live_holder(&self, id: &PersistentNameIdentity) -> Option<PersistentNameHolder> {
        self.live.get(id).cloned()
    }
    fn live_lineage(&self, holder: &PersistentNameHolder) -> Option<LineageId> {
        self.lineages.get(holder).copied()
    }
    fn retired_lineage(&self, id: &PersistentNameIdentity) -> Option<(LineageSubject, LineageId)> {
        self.retired.get(id).copied()
    }
    fn holder_name(&self, holder: &PersistentNameHolder) -> Option<PersistentNameIdentity> {
        self.names.get(holder).copied()
    }
}

fn name(byte: u8) -> PersistentNameIdentity {
    PersistentNameIdentity::from_minted_digest([byte; 32])
}

fn staged_with(count: usize) -> (WorkingState, Vec<EntityId>) {
    let mut staged = WorkingState::working_state_for_test(
        BTreeMap::new(),
        AdjacencyPolicy {
            backend: AdjacencyBackend::InlineSmallDegreeAdjacency,
            small_degree_inline_capacity: 4,
        },
    );
    let part = staged.get_partition_mut(PartitionId::main());
    let ids = (0..count)
        .map(|_| {
            let (slot, generation, _) = part.entity_arena.push_slot_for_test(SlotInit::<
                EntityRecordKind,
            > {
                partition_id: PartitionId::main(),
                kind_id: KindId(1),
                version_id: VersionId(1),
                extra: Default::default(),
            });
            EntityId::new(PartitionId::main(), slot as u64, generation)
        })
        .collect();
    (staged, ids)
}

fn empty_batch() -> ValidatedPostureBatch {
    ValidatedPostureBatch {
        postures: Vec::new(),
        minted_births: BTreeMap::new(),
        claimed_created_targets: BTreeSet::new(),
        restored_created_targets: BTreeSet::new(),
        claimed_basis_targets: BTreeSet::new(),
    }
}

fn transition(
    retired: &[EntityId],
    restored: &[EntityId],
    kept: &[EntityId],
) -> SelectionTransition {
    SelectionTransition {
        retired: retired
            .iter()
            .copied()
            .map(LineageRecordTarget::Entity)
            .collect(),
        restored: restored
            .iter()
            .copied()
            .map(LineageRecordTarget::Entity)
            .collect(),
        kept_entities: kept.to_vec(),
    }
}

#[test]
fn a_retired_named_record_retires_its_live_branch_lineage() {
    let (staged, ids) = staged_with(2);
    let mut basis = Basis::default();
    basis.live(PersistentNameHolder::Entity(ids[0]), name(1), 40);
    let mut batch = empty_batch();
    plan(
        &staged,
        &basis,
        None,
        &transition(&ids, &[], &[]),
        &mut batch,
    )
    .unwrap();
    assert!(
        matches!(
            batch.postures.as_slice(),
            [ValidatedPosture::Retire { source_lineage: LineageId(40), name: n, .. }] if *n == name(1)
        ),
        "only the named record retires"
    );
    assert!(batch
        .claimed_basis_targets
        .contains(&LineageRecordTarget::Entity(ids[0])));
}

#[test]
fn a_restored_record_restores_its_names_last_retired_lineage() {
    let (staged, ids) = staged_with(1);
    let mut basis = Basis::default();
    basis.retired(PersistentNameHolder::Entity(ids[0]), name(2), 41);
    let mut batch = empty_batch();
    plan(
        &staged,
        &basis,
        None,
        &transition(&[], &ids, &[]),
        &mut batch,
    )
    .unwrap();
    assert!(matches!(
        batch.postures.as_slice(),
        [ValidatedPosture::Restore { source_lineage: LineageId(41), name: n, target }]
            if *n == name(2) && *target == LineageRecordTarget::Entity(ids[0])
    ));
}

#[test]
fn a_name_live_on_a_record_the_selection_retires_continues_in_place() {
    let (staged, ids) = staged_with(2);
    let (restored, regenerated) = (ids[0], ids[1]);
    let mut basis = Basis::default();
    basis.live(PersistentNameHolder::Entity(regenerated), name(3), 50);
    basis
        .names
        .insert(PersistentNameHolder::Entity(restored), name(3));
    let mut batch = empty_batch();
    plan(
        &staged,
        &basis,
        None,
        &transition(&[regenerated], &[restored], &[]),
        &mut batch,
    )
    .unwrap();
    assert!(
        matches!(
            batch.postures.as_slice(),
            [ValidatedPosture::Regenerate { source_lineage: LineageId(50), name: n, .. }] if *n == name(3)
        ),
        "no false Retire: {} postures",
        batch.postures.len()
    );
}

#[test]
fn a_restored_name_still_live_on_a_kept_record_refuses_duplicate_birth() {
    let (staged, ids) = staged_with(2);
    let mut basis = Basis::default();
    basis.live(PersistentNameHolder::Entity(ids[1]), name(4), 60);
    basis
        .names
        .insert(PersistentNameHolder::Entity(ids[0]), name(4));
    let refused = plan(
        &staged,
        &basis,
        None,
        &transition(&[], &ids[..1], &ids[1..]),
        &mut empty_batch(),
    );
    assert!(
        matches!(refused, Err(LineageNameDenial::DuplicateBirth { identity, .. }) if identity == name(4))
    );
}

#[test]
fn restoring_one_name_twice_refuses_duplicate_birth() {
    let (staged, ids) = staged_with(2);
    let mut basis = Basis::default();
    basis.retired(PersistentNameHolder::Entity(ids[0]), name(5), 70);
    basis
        .names
        .insert(PersistentNameHolder::Entity(ids[1]), name(5));
    let refused = plan(
        &staged,
        &basis,
        None,
        &transition(&[], &ids, &[]),
        &mut empty_batch(),
    );
    assert!(matches!(
        refused,
        Err(LineageNameDenial::DuplicateBirth { .. })
    ));
}

#[test]
fn unnamed_records_are_untouched_and_kept_named_slots_are_repaired() {
    let (staged, ids) = staged_with(3);
    let mut basis = Basis::default();
    basis.live(PersistentNameHolder::Entity(ids[2]), name(6), 80);
    let mut batch = empty_batch();
    let repairs = plan(
        &staged,
        &basis,
        None,
        &transition(&ids[..1], &ids[1..2], &ids[2..]),
        &mut batch,
    )
    .unwrap();
    assert!(batch.postures.is_empty(), "unnamed records emit nothing");
    assert!(matches!(
        repairs.as_slice(),
        [repair] if repair.entity_id == ids[2] && repair.lineage_id == LineageId(80)
    ));
}

#[test]
fn relations_retire_and_restore_by_their_own_branch_lineage() {
    let (staged, _) = staged_with(0);
    let (retired, restored) = (
        RelationId::new(PartitionId::main(), 3, 1),
        RelationId::new(PartitionId::main(), 4, 1),
    );
    let mut basis = Basis::default();
    basis.live(PersistentNameHolder::Relation(retired), name(7), 90);
    basis
        .retired
        .insert(name(8), (LineageSubject::Relation, LineageId(91)));
    basis
        .names
        .insert(PersistentNameHolder::Relation(restored), name(8));
    let mut batch = empty_batch();
    let transition = SelectionTransition {
        retired: vec![LineageRecordTarget::Relation(retired)],
        restored: vec![LineageRecordTarget::Relation(restored)],
        kept_entities: Vec::new(),
    };
    plan(&staged, &basis, None, &transition, &mut batch).unwrap();
    assert!(batch.postures.iter().any(|posture| matches!(
        posture,
        ValidatedPosture::Restore { source_lineage: LineageId(91), target, .. }
            if *target == LineageRecordTarget::Relation(restored)
    )));
    assert!(batch.postures.iter().any(|posture| matches!(
        posture,
        ValidatedPosture::Retire {
            source_lineage: LineageId(90),
            subject: LineageSubject::Relation,
            ..
        }
    )));
}

#[test]
fn any_caller_posture_beside_a_selection_refuses() {
    let (_, ids) = staged_with(1);
    let mut batch = empty_batch();
    assert!(super::caller_plan_refusal(&batch, &transition(&ids, &[], &[]), ids[0]).is_none());
    batch.postures.push(ValidatedPosture::Preserve);
    batch
        .claimed_basis_targets
        .insert(LineageRecordTarget::Entity(ids[0]));
    assert!(matches!(
        super::caller_plan_refusal(&batch, &transition(&ids, &[], &[]), ids[0]),
        Some(LineageNameDenial::LineagePostureMismatch { .. })
    ));
}
