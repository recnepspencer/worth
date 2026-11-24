//! `#314`: a `KeepParentInPlace` merge whose kept parent lives in a partition
//! the merging commit never touches. The commit stages only the retiring
//! parent's partition; finalization must still find the kept parent live at
//! the commit basis, keep it in place, and resolve both parent names to it.

use std::collections::BTreeMap;

use super::finalize_postures_in_working_state_for_test;
use super::posture_fixture::LineageCommitPostureInput;
use crate::contracts::history::data::coordinates::{BranchId, CommitId};
use crate::contracts::identity::data::{
    primitives::{KindId, LineageId, PartitionId, VersionId},
    record_ids::{EntityId, RelationId},
};
use crate::contracts::lineage::data::{
    LineageEventKind, LineageMergeKeeper, LineageNameDenial, LineagePostureClaim,
    LineageRecordTarget, LineageSubject, PersistentNameHolder, PersistentNameIdentity,
    PersistentNameKey, PersistentNameResolution, PersistentNameSelection, PersistentNameTarget,
    PreparedLineageFinalization,
};
use crate::contracts::transactions::data::intents::mutation_intent::{
    DeleteEntityIntent, DeleteRelationIntent, EntityMutationIntent, MutationIntent,
    RelationMutationIntent,
};
use crate::contracts::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::runtime::state::subsystems::lineage_identity::LineageIdentityAllocator;
use crate::storage::storage::overlay::{CurrentPartitionAccess, WorkingState};
use crate::storage::storage::substrate::{
    EntityRecordKind, RelationEndpoints, RelationExtra, RelationRecordKind, SlotInit,
};

use super::posture_fixture::records::{
    cad_scheme_registry, empty_working_state, face_birth, receipt_at, retire_entity,
    retire_relation, seam_birth,
};
use crate::contracts::lineage::data::artifacts::finalization_artifact::PreparedLineageFinalizationFixturePort as _;
use crate::lineage::index::{
    batch::{PersistentNameCommittedBatch, PersistentNameCommittedBatchFixturePort as _},
    PersistentNameBranchIndex,
};
use crate::storage::storage::substrate::record_arena::arena::slot_lifecycle::RecordArenaFixturePort as _;

/// The keeper's partition: never staged by the merging commit.
fn keeper_partition() -> PartitionId {
    PartitionId::new(7)
}

fn push_entity_in(staged: &mut WorkingState, partition: PartitionId) -> EntityId {
    let (slot, generation, _) = staged
        .get_partition_mut(partition)
        .entity_arena
        .push_slot_for_test(SlotInit::<EntityRecordKind> {
            partition_id: partition,
            kind_id: KindId(1),
            version_id: VersionId(1),
            extra: Default::default(),
        });
    EntityId::new(partition, slot as u64, generation)
}

fn push_relation_in(
    staged: &mut WorkingState,
    partition: PartitionId,
    ends: (EntityId, EntityId),
) -> RelationId {
    let (slot, generation, _) = staged
        .get_partition_mut(partition)
        .relation_arena
        .push_slot_for_test(SlotInit::<RelationRecordKind> {
            partition_id: partition,
            kind_id: KindId(10),
            version_id: VersionId(1),
            extra: RelationExtra::from_funded_parts(
                Some(RelationEndpoints {
                    source: ends.0,
                    target: ends.1,
                }),
                None,
                None,
            ),
        });
    RelationId::new(partition, slot as u64, generation)
}

/// Two named parents born in commit 1: `kept` in [`keeper_partition`],
/// `absorbed` in the main partition.
struct Born {
    kept: LineageRecordTarget,
    absorbed: LineageRecordTarget,
    names: (PersistentNameIdentity, PersistentNameIdentity),
    lineages: (LineageId, LineageId),
    batch: PersistentNameCommittedBatch,
}

/// Stages the main partition exactly as commit 1 left it: `absorbed` is its
/// first record, so a fresh stage reproduces its identity.
fn stage_main(relation: bool) -> (WorkingState, LineageRecordTarget) {
    let mut staged = empty_working_state();
    let anchor = push_entity_in(&mut staged, PartitionId::main());
    let target = if relation {
        let other = push_entity_in(&mut staged, PartitionId::main());
        LineageRecordTarget::Relation(push_relation_in(
            &mut staged,
            PartitionId::main(),
            (anchor, other),
        ))
    } else {
        LineageRecordTarget::Entity(anchor)
    };
    (staged, target)
}

fn birth(target: LineageRecordTarget, label: &str) -> LineagePostureClaim {
    let birth = match target {
        LineageRecordTarget::Entity(entity) => {
            face_birth("occ.314", "authored", vec![], label, entity)
        }
        LineageRecordTarget::Relation(relation) => {
            seam_birth("occ.314", "seam", vec![], label, relation)
        }
    };
    LineagePostureClaim::Create { target, birth }
}

fn born(allocator: &LineageIdentityAllocator, relation: bool) -> Born {
    let (mut staged, absorbed) = stage_main(relation);
    let kept = if relation {
        let ends = (
            push_entity_in(&mut staged, keeper_partition()),
            push_entity_in(&mut staged, keeper_partition()),
        );
        LineageRecordTarget::Relation(push_relation_in(&mut staged, keeper_partition(), ends))
    } else {
        LineageRecordTarget::Entity(push_entity_in(&mut staged, keeper_partition()))
    };
    let input = LineageCommitPostureInput {
        scheme_registry: cad_scheme_registry(),
        postures: vec![birth(kept, "kept"), birth(absorbed, "absorbed")],
        ..LineageCommitPostureInput::default()
    };
    let changed = [kept.to_record_ref(), absorbed.to_record_ref()];
    let prepared = finalize_postures_in_working_state_for_test(
        allocator,
        &mut staged,
        &receipt_at(1, 1),
        &[],
        &changed,
        &input,
    )
    .expect("commit 1 names both parents");
    let mut holders = BTreeMap::new();
    for node in prepared.new_nodes_for_test() {
        holders.insert(
            node.lineage_id(),
            PersistentNameHolder::Entity(node.entity_id()),
        );
    }
    for (relation, lineage) in prepared.relation_lineages_for_test() {
        holders.insert(*lineage, PersistentNameHolder::Relation(*relation));
    }
    let mut by_holder = BTreeMap::new();
    for event in prepared.artifact().event_batch().events() {
        let birth = event.name_birth().expect("every commit-1 event is a birth");
        let lineage = event.targets()[0];
        by_holder.insert(holders[&lineage].clone(), (birth.identity(), lineage));
    }
    let (kept_name, kept_lineage) = by_holder[&kept.to_holder()];
    let (absorbed_name, absorbed_lineage) = by_holder[&absorbed.to_holder()];
    Born {
        kept,
        absorbed,
        names: (kept_name, absorbed_name),
        lineages: (kept_lineage, absorbed_lineage),
        batch: PersistentNameCommittedBatch::new(CommitId(1))
            .with_prepared_lineage_for_test(&prepared),
    }
}

/// The basis the merging commit starts from, optionally with `kept`'s name
/// pointing at `kept_holder` instead.
fn basis(born: &Born, kept_holder: Option<PersistentNameHolder>) -> LineageCommitPostureInput {
    let subject = match born.kept {
        LineageRecordTarget::Entity(_) => LineageSubject::Entity,
        LineageRecordTarget::Relation(_) => LineageSubject::Relation,
    };
    let mut input = LineageCommitPostureInput {
        scheme_registry: cad_scheme_registry(),
        ..LineageCommitPostureInput::default()
    };
    match kept_holder {
        Some(holder) => {
            input.live_holders.insert(born.names.0, holder);
        }
        None => {
            input
                .retired_lineages
                .insert(born.names.0, (subject, born.lineages.0));
        }
    }
    input
        .live_holders
        .insert(born.names.1, born.absorbed.to_holder());
    input
        .live_lineages
        .insert(born.kept.to_holder(), born.lineages.0);
    input
        .live_lineages
        .insert(born.absorbed.to_holder(), born.lineages.1);
    input
}

/// Commit 2 retires `absorbed` and merges it into `kept`, kept in place,
/// staging only the main partition.
fn merge(
    allocator: &LineageIdentityAllocator,
    born: &Born,
    mut input: LineageCommitPostureInput,
) -> Result<PreparedLineageFinalization, TransactionCommitError> {
    let (mut staged, absorbed) =
        stage_main(matches!(born.absorbed, LineageRecordTarget::Relation(_)));
    assert_eq!(
        absorbed, born.absorbed,
        "the fresh stage reproduces commit 1's record"
    );
    assert!(
        staged.get_current_partition(keeper_partition()).is_none(),
        "the kept parent's partition is untouched by the merging commit"
    );
    let delete = match absorbed {
        LineageRecordTarget::Entity(entity) => {
            retire_entity(&mut staged, entity, VersionId(2));
            MutationIntent::Entity(EntityMutationIntent::Delete(DeleteEntityIntent {
                entity_id: entity,
            }))
        }
        LineageRecordTarget::Relation(relation) => {
            retire_relation(&mut staged, relation, VersionId(2));
            MutationIntent::Relation(RelationMutationIntent::Delete(DeleteRelationIntent {
                relation_id: relation,
            }))
        }
    };
    input.postures = vec![LineagePostureClaim::Merge {
        parents: vec![(born.kept, born.names.0), (born.absorbed, born.names.1)],
        target: born.kept,
        keeper: LineageMergeKeeper::KeepParent(born.names.0),
    }];
    finalize_postures_in_working_state_for_test(
        allocator,
        &mut staged,
        &receipt_at(2, 2),
        &[delete],
        &[absorbed.to_record_ref()],
        &input,
    )
}

fn key(target: LineageRecordTarget, name: PersistentNameIdentity) -> PersistentNameKey {
    match target {
        LineageRecordTarget::Entity(_) => PersistentNameKey::Entity(name),
        LineageRecordTarget::Relation(_) => PersistentNameKey::Relation(name),
    }
}

fn keeps_in_place_and_resolves_both_names(relation: bool) {
    let allocator = LineageIdentityAllocator::new();
    let born = born(&allocator, relation);
    let prepared = merge(&allocator, &born, basis(&born, Some(born.kept.to_holder())))
        .unwrap_or_else(|denial| panic!("#314: untouched in-place keeper must merge: {denial:?}"));
    let events = prepared.artifact().event_batch().events();
    let merge_event = events
        .iter()
        .find(|event| event.kind() == LineageEventKind::Merge)
        .expect("one canonical merge event");
    let mut sources = merge_event.sources().to_vec();
    sources.sort_unstable();
    let mut expected = vec![born.lineages.0, born.lineages.1];
    expected.sort_unstable();
    assert_eq!(sources, expected, "both parents feed the merge");
    assert_eq!(
        merge_event.targets(),
        [born.lineages.0],
        "the kept lineage stays in place"
    );

    let log = [
        born.batch.clone(),
        PersistentNameCommittedBatch::new(CommitId(2)).with_prepared_lineage_for_test(&prepared),
    ];
    let index =
        PersistentNameBranchIndex::rebuild_from_log(BranchId("main".to_owned()), None, &log)
            .expect("canonical log rebuilds");
    let expected = match born.kept {
        LineageRecordTarget::Entity(entity) => PersistentNameTarget::Entity {
            name: born.names.0,
            entity,
            lineage: born.lineages.0,
        },
        LineageRecordTarget::Relation(relation) => PersistentNameTarget::Relation {
            name: born.names.0,
            relation,
            lineage: born.lineages.0,
        },
    };
    for (target, name) in [(born.kept, born.names.0), (born.absorbed, born.names.1)] {
        for selection in [
            PersistentNameSelection::Exact,
            PersistentNameSelection::AllDescendants,
        ] {
            let resolved = index
                .resolve(key(target, name), CommitId(2), &selection)
                .result;
            assert_eq!(
                resolved,
                Ok(PersistentNameResolution::Exact(expected)),
                "{selection:?}"
            );
        }
    }
}

fn refuses_posture_mismatch<T>(outcome: Result<T, TransactionCommitError>) {
    let Err(denial) = outcome else {
        panic!("the merge must refuse");
    };
    assert!(
        matches!(
            denial.lineage_name_denial_ref(),
            Some(LineageNameDenial::LineagePostureMismatch { .. })
        ),
        "typed posture refusal, got {denial:?}"
    );
}

#[test]
fn untouched_entity_keeper_merges_in_place() {
    keeps_in_place_and_resolves_both_names(false);
}

#[test]
fn untouched_relation_keeper_merges_in_place() {
    keeps_in_place_and_resolves_both_names(true);
}

#[test]
fn untouched_keeper_not_live_at_basis_refuses() {
    for relation in [false, true] {
        let allocator = LineageIdentityAllocator::new();
        let born = born(&allocator, relation);
        // The kept name was retired before the merge's basis.
        refuses_posture_mismatch(merge(&allocator, &born, basis(&born, None)));
        // The kept name is live on another record at the basis.
        refuses_posture_mismatch(merge(
            &allocator,
            &born,
            basis(&born, Some(born.absorbed.to_holder())),
        ));
    }
}
