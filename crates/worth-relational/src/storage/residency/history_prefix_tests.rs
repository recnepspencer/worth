//! Producer-written history bases follow rows rather than record generations.
use super::*;
use crate::identity::data::VersionId;
use crate::storage::data::RecordLifecycleState;
use crate::storage::substrate::{EntityArena, EntityExtra, SlotInit};
use std::collections::BTreeSet;

#[test]
fn a_head_history_prefix_survives_reset_and_reuse_until_its_rows_are_suspended() {
    let fixture = region_with_history();
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let head = region_load::load_head(&fixture.cold, execution, 1 << 24).unwrap();
    let mut arena = head.partition.entity_arena.clone();
    let slot = fixture.entity.slot_index();
    let physical = arena.physical_index(slot).unwrap();
    let prefix = arena.history_prefixes[physical].clone().unwrap();
    assert_eq!(prefix.member, fixture.cold.history);
    assert_eq!(prefix.partition_id, fixture.entity.partition_id);
    assert_eq!(
        prefix.schema_carrier_digest,
        fixture.cold.schema_carrier_digest
    );
    assert_eq!(prefix.row_index, physical);
    let source: Arc<dyn RelationalDurableColdSource> = fixture.source.clone();
    assert!(Arc::ptr_eq(&prefix.source, &source));

    arena.retire(slot, VersionId(1_000));
    arena.set_lifecycle_for_slot(slot, RecordLifecycleState::Reusable);
    arena.reset_slot(slot);
    assert_eq!(
        arena.history_prefixes[physical].as_ref().unwrap().member,
        prefix.member
    );
    arena
        .write_reserved_slot(
            SlotInit {
                partition_id: fixture.entity.partition_id,
                kind_id: KindId(1),
                version_id: VersionId(1_001),
                extra: EntityExtra::default(),
            },
            slot,
            fixture.entity.generation_value() + 1,
        )
        .unwrap();
    assert_eq!(arena.metadata_history[physical].len(), 2);
    assert_eq!(
        arena.history_prefixes[physical].as_ref().unwrap().member,
        prefix.member
    );
    assert_eq!(
        arena.history_prefixes[physical].as_ref().unwrap().row_index,
        prefix.row_index
    );
    assert_eq!(
        arena.history_prefixes[physical]
            .as_ref()
            .unwrap()
            .history_frontier
            .id(),
        prefix.history_frontier.id(),
        "generation reuse preserves the original shared history opening"
    );

    arena.suspend_materialization(slot).unwrap();
    assert!(arena.metadata_history[physical].is_empty());
    assert!(arena.history_prefixes[physical].is_none());
    assert_eq!(fixture.source.history_reads.load(Ordering::SeqCst), 0);
}

#[test]
fn sparse_adoption_preserves_the_source_history_row_when_the_physical_row_changes() {
    let fixture = region_with_history();
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let head = region_load::load_head(&fixture.cold, execution, 1 << 24).unwrap();
    let slot = fixture.entity.slot_index();
    let selected = BTreeSet::from([slot]);
    let mut overlay = head
        .partition
        .entity_arena
        .sparse_clone_slots_for_overlay(&selected);
    let source_row = overlay.physical_index(slot).unwrap();
    let prefix = overlay.history_prefixes[source_row]
        .as_ref()
        .unwrap()
        .clone();
    let mut destination = EntityArena::with_capacity(0);
    destination
        .write_reserved_slot(
            SlotInit {
                partition_id: fixture.entity.partition_id,
                kind_id: KindId(1),
                version_id: VersionId(1_000),
                extra: EntityExtra::default(),
            },
            90,
            1,
        )
        .unwrap();
    destination.merge_slots_from_owned(&mut overlay, &selected);
    let adopted_row = destination.physical_index(slot).unwrap();
    assert_ne!(adopted_row, prefix.row_index);
    let adopted = destination.history_prefixes[adopted_row].as_ref().unwrap();
    assert_eq!(adopted.member, prefix.member);
    assert_eq!(adopted.row_index, prefix.row_index);
    assert_eq!(adopted.history_frontier.id(), prefix.history_frontier.id());
    assert!(Arc::ptr_eq(&adopted.source, &prefix.source));
    assert_eq!(destination.metadata_history[adopted_row].len(), 1);
    assert_eq!(fixture.source.history_reads.load(Ordering::SeqCst), 0);
}

#[test]
fn a_prefix_carries_its_authenticated_slot_opening_and_accounts_for_its_owner() {
    let fixture = region_with_history();
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let head = region_load::load_head(&fixture.cold, execution, 1 << 24).unwrap();
    let physical = head
        .partition
        .entity_arena
        .physical_index(fixture.entity.slot_index())
        .unwrap();
    let prefix = head.partition.entity_arena.history_prefixes[physical]
        .as_ref()
        .unwrap();
    let (openings, _) = head
        .commitment
        .as_ref()
        .unwrap()
        .history_frontiers(&head.partition)
        .unwrap()
        .into_arenas();
    let (_, expected) = openings
        .into_iter()
        .find(|(slot, _)| *slot == fixture.entity.slot_index() as u64)
        .unwrap();
    assert_eq!(prefix.history_frontier.as_ref(), &expected);
    struct Owners(BTreeMap<u64, u64>);
    impl crate::storage::substrate::StorageAllocationVisitor for Owners {
        fn visit(
            &mut self,
            value: crate::storage::substrate::StorageAllocationObservation,
        ) -> bool {
            self.0.insert(value.id, value.bytes).is_none()
        }
    }
    let mut owners = Owners(BTreeMap::new());
    head.partition
        .entity_arena
        .visit_authoritative_allocations(false, &mut owners);
    let bytes = owners
        .0
        .get(&prefix.history_frontier.id())
        .expect("the actual opening owner is part of the arena footprint");
    assert!(*bytes >= prefix.history_frontier.owned_allocation_bytes());
    assert_eq!(fixture.source.history_reads.load(Ordering::SeqCst), 0);
}

#[test]
fn flat_capture_refuses_a_cold_prefix_and_full_admission_replaces_it_with_complete_rows() {
    let fixture = region_with_history();
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let head = region_load::load_head(&fixture.cold, execution, 1 << 24).unwrap();
    let symbols = fixture.cold.symbols.interner_snapshot();
    assert!(matches!(
        capture_region_members(
            &head.partition,
            head.commitment.as_ref().unwrap(),
            &fixture.cold.schema,
            fixture.cold.schema_carrier_digest,
        ),
        Err(Cause::SourceUnavailable)
    ));
    assert_eq!(fixture.source.history_reads.load(Ordering::SeqCst), 0);
    let full = region_load::load_full(&fixture.cold, &head, execution, 1 << 24).unwrap();
    assert!(full
        .partition
        .entity_arena
        .history_prefixes
        .iter()
        .all(Option::is_none));
    assert!(full
        .partition
        .relation_arena
        .history_prefixes
        .iter()
        .all(Option::is_none));
    assert_eq!(
        full.partition
            .entity_arena
            .metadata_history_at(fixture.entity.slot_index())
            .unwrap()
            .len(),
        fixture.expected_history
    );
    let (commitment, _) =
        PartitionContentCommitment::capture(&full.partition, &symbols, None).unwrap();
    let captured = capture_region_members(
        &full.partition,
        &commitment,
        &fixture.cold.schema,
        fixture.cold.schema_carrier_digest,
    )
    .unwrap();
    assert_eq!(captured.history.0, fixture.cold.history);
    assert_eq!(
        commitment.digest(fixture.entity.partition_id),
        fixture.expected_digest
    );
}
