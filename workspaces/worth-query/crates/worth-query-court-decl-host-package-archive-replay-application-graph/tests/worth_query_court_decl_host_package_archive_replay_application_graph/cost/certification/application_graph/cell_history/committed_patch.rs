//! Observe the sealed public receipt after measurement, without patch copies.
use worth_query_host::facade::primary_graph::{
    RecordStructuralChange, WorthQueryApplicationCommitReceipt,
};

pub fn assert_one_updated_cell(
    receipt: &WorthQueryApplicationCommitReceipt,
    occurrence: (u32, u64, u32),
) {
    let changes = receipt.committed_changes();
    let mut entities = 0;
    let mut created = 0;
    let mut deleted = 0;
    let mut updated = 0;
    for (entity, change) in changes.entity_changes() {
        entities += 1;
        created += usize::from(change == RecordStructuralChange::Created);
        deleted += usize::from(change == RecordStructuralChange::Deleted);
        let actual = (
            entity.partition_id.0,
            entity.local_slot.0,
            entity.generation.0,
        );
        if change == RecordStructuralChange::Updated {
            updated += 1;
            assert_eq!(
                actual, occurrence,
                "the updated patch targets the queried Cell GEN"
            );
        } else {
            assert_eq!(change, RecordStructuralChange::Created);
            assert_ne!(
                actual, occurrence,
                "provider registration cannot replace the Cell"
            );
        }
    }
    let total = receipt.changed_record_count();
    // RecordRef is exactly Entity | Relation; both observations derive from
    // the same canonical authoritative_record_patches at receipt construction.
    let relations = total.checked_sub(entities).unwrap();
    println!(
        "cell publication version={:?} changed_records={total} entity_records={entities} relation_records={relations} created={created} deleted={deleted} updated={updated} updated_cell={occurrence:?}",
        changes.commit_reference().version_id
    );
    // The ordinary provider appends one real idempotency registration record;
    // the authored operation itself updates exactly the existing Cell.
    assert_eq!(
        (total, entities, relations, created, deleted, updated),
        (2, 2, 0, 1, 0, 1)
    );
}
