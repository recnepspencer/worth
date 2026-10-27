//! Consuming Full custody preserves an outstanding immutable observation.
use super::*;
use crate::identity::data::VersionId;

#[test]
fn consuming_full_custody_releases_its_lease_and_keeps_shared_history_immutable() {
    let fixture = region_with_history();
    let lease = full_demand(&fixture.cell, 1 << 24).unwrap();
    let observer = lease.clone();
    let reads = fixture.source.reads.load(Ordering::SeqCst);
    let mut owned = lease.into_owned_partition();
    assert_eq!(fixture.cell.leases.load(Ordering::SeqCst), 1);
    let extra = owned
        .entity_arena
        .get_slot(fixture.entity.slot_index())
        .unwrap()
        .extra()
        .clone();
    owned
        .entity_arena
        .apply_extra_update(fixture.entity.slot_index(), extra, VersionId(1_000));
    assert_eq!(
        owned
            .entity_arena
            .metadata_history_at(fixture.entity.slot_index())
            .unwrap()
            .len(),
        fixture.expected_history + 1
    );
    assert_eq!(
        observer
            .full_view()
            .partition()
            .entity_arena
            .metadata_history_at(fixture.entity.slot_index())
            .unwrap()
            .len(),
        fixture.expected_history
    );
    assert_eq!(
        observer
            .full_view()
            .partition()
            .authoritative_content_digest(&fixture.cold.symbols.interner_snapshot())
            .unwrap(),
        fixture.expected_digest
    );
    assert_eq!(fixture.source.reads.load(Ordering::SeqCst), reads);
    drop(observer);
    assert_eq!(fixture.cell.leases.load(Ordering::SeqCst), 0);
}
