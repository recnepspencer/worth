//! A history restore rebinds the exact original record to a fresh lineage.
//! Pinned reads must keep the binding valid at their basis: born live under
//! `L1`, ended under `L1` after deletion, and live under `L2` only from the
//! restoring commit on, for entities and relations, live and rebuilt.
use std::collections::BTreeMap;

use crate::facade::history::{BranchId, CommitId};
use crate::facade::identity::LineageId;
use crate::facade::lineage::{
    LineageEventKind, LineageEventRecord, LineageRecordTarget, LineageSubject,
    PersistentNameBranchIndex, PersistentNameCommittedBatch, PersistentNameHolder,
    PersistentNameOfRecord,
};

use super::name_index_fixtures::{main_branch, rebuild};
use super::name_world::{key, record_target, LineageWorld, Subject};
use super::posture_fixtures::receipt_at;

/// The lineage the `Create` in `batch` bound.
fn created_lineage(batch: &PersistentNameCommittedBatch) -> LineageId {
    batch
        .lineage_events
        .iter()
        .find(|event| event.kind() == LineageEventKind::Create)
        .map(|event| event.targets()[0])
        .expect("the birth commit creates one lineage")
}

/// The canonical batch a selection restore publishes: `Restore [l1] -> [l2]`
/// with the kept name, bound to the exact original record.
fn restore_batch(
    commit: u64,
    holder: &PersistentNameHolder,
    name: crate::facade::lineage::PersistentNameIdentity,
    l1: LineageId,
    l2: LineageId,
) -> PersistentNameCommittedBatch {
    let (subject, relation_id, entity_id) = match holder {
        PersistentNameHolder::Relation(relation) => {
            (LineageSubject::Relation, Some(*relation), None)
        }
        PersistentNameHolder::Entity(entity) => (LineageSubject::Entity, None, Some(*entity)),
        _ => unreachable!("worlds hold only entity and relation records"),
    };
    let event = LineageEventRecord {
        event_id: 10_000 + commit,
        commit: receipt_at(commit, commit),
        branch_id: BranchId("main".to_owned()),
        kind: LineageEventKind::Restore,
        sources: vec![l1],
        targets: vec![l2],
        subject,
        name_birth: None,
        kept_name: Some(name),
        relation_id,
        entity_id,
    };
    let mut batch = PersistentNameCommittedBatch::new(CommitId(commit));
    batch.lineage_events = vec![event];
    match holder {
        PersistentNameHolder::Entity(entity) => {
            batch.entity_holders = BTreeMap::from([(l2, *entity)]);
        }
        PersistentNameHolder::Relation(relation) => {
            batch.relation_holders = BTreeMap::from([(l2, *relation)]);
        }
        _ => unreachable!("worlds hold only entity and relation records"),
    }
    batch
}

fn assert_epochs(subject: Subject) {
    let mut world = LineageWorld::new();
    let holder = world.create(subject, "restored");
    let born = world.commit();
    let name = world.name_of(&holder);
    let l1 = created_lineage(&born);
    world.retire(name);
    world.commit();
    let mut log = world.log().to_vec();
    let restore = log.len() as u64 + 1;
    let l2 = LineageId(l1.0 + 1_000);
    log.push(restore_batch(restore, &holder, name, l1, l2));
    let live =
        PersistentNameBranchIndex::rebuild_from_log(main_branch(), None, &log[..log.len() - 1])
            .and_then(|mut index| index.apply(log.last().unwrap()).map(|()| index))
            .expect("the restore batch ingests");
    let rebuilt = rebuild(&log);
    assert_eq!(live, rebuilt, "rebuild reproduces the live index");
    let named = key(&holder, name);
    let record: LineageRecordTarget = record_target(&holder);
    for index in [&live, &rebuilt] {
        assert_eq!(
            index.name_of_record(record, born.commit()),
            PersistentNameOfRecord::Live {
                name: named,
                lineage: l1
            },
            "{subject:?} at its birth"
        );
        assert_eq!(
            index.name_of_record(record, CommitId(restore - 1)),
            PersistentNameOfRecord::Ended {
                name: named,
                lineage: l1
            },
            "{subject:?} after deletion"
        );
        assert_eq!(
            index.name_of_record(record, CommitId(restore)),
            PersistentNameOfRecord::Live {
                name: named,
                lineage: l2
            },
            "{subject:?} after the restore"
        );
    }
}

#[test]
fn pinned_entity_reads_keep_the_lineage_bound_at_their_basis_across_a_restore() {
    assert_epochs(Subject::Entity);
}

#[test]
fn pinned_relation_reads_keep_the_lineage_bound_at_their_basis_across_a_restore() {
    assert_epochs(Subject::Relation);
}

#[test]
fn a_refused_restore_batch_leaves_every_binding_epoch_untouched() {
    let mut world = LineageWorld::new();
    let holder = world.create(Subject::Entity, "refused");
    let born = world.commit();
    let name = world.name_of(&holder);
    let l1 = created_lineage(&born);
    let mut index = rebuild(world.log());
    let before = index.clone();
    // The name is still live, so a restore of it refuses and must roll back.
    let refused = restore_batch(born.commit().0 + 1, &holder, name, l1, LineageId(l1.0 + 7));
    assert!(index.apply(&refused).is_err());
    assert_eq!(index, before);
    assert_eq!(
        index.name_of_record(record_target(&holder), CommitId(born.commit().0 + 1)),
        PersistentNameOfRecord::Live {
            name: key(&holder, name),
            lineage: l1
        }
    );
}
