//! Adopt producer-authenticated histories and compare against complete rows.
use super::*;
use crate::identity::data::VersionId;
use crate::storage::overlay::{
    PartitionContentDigestError, PartitionMutationJournal, PartitionState,
};
use crate::storage::substrate::{EntityArena, EntityExtra, SlotInit};
use std::collections::BTreeSet;

#[test]
fn replacing_a_prefix_in_either_direction_reseeds_only_the_selected_history() {
    let fixture = region_with_history();
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let first = region_load::load_head(&fixture.cold, execution, 1 << 24).unwrap();
    let full = region_load::load_full(&fixture.cold, &first, execution, 1 << 24).unwrap();
    let mut later = full.partition.clone();
    let slot = fixture.entity.slot_index();
    let extra = later.entity_arena.get_slot(slot).unwrap().extra().clone();
    later
        .entity_arena
        .apply_extra_update(slot, extra, VersionId(2_000));
    let (other_cold, other_source) = recapture(&fixture.cold, &later);
    let second = region_load::load_head(&other_cold, execution, 1 << 24).unwrap();
    let symbols = fixture.cold.symbols.interner_snapshot();
    let selected = BTreeSet::from([slot]);
    let journal = PartitionMutationJournal {
        entity_slots: selected.clone(),
        ..Default::default()
    };
    let mut head = first.partition.clone();
    let mut oracle = full.partition.clone();
    let mut commitment = first.commitment.clone().unwrap();
    for (source, complete) in [
        (&second.partition, &later),
        (&first.partition, &full.partition),
        (&second.partition, &later),
        (&later, &later),
        (&second.partition, &later),
    ] {
        let previous = head.clone();
        head.entity_arena
            .merge_slots_from_owned(&mut source.entity_arena.clone(), &selected);
        oracle
            .entity_arena
            .merge_slots_from_owned(&mut complete.entity_arena.clone(), &selected);
        let (next, _) = PartitionContentCommitment::capture(
            &head,
            &symbols,
            Some((&commitment, &previous, &journal)),
        )
        .unwrap();
        assert_content(&head, &oracle, &symbols, &next);
        commitment = next;
    }
    let previous = head.clone();
    let extra = head.entity_arena.get_slot(slot).unwrap().extra().clone();
    head.entity_arena
        .apply_extra_update(slot, extra.clone(), VersionId(3_000));
    oracle
        .entity_arena
        .apply_extra_update(slot, extra, VersionId(3_000));
    let (next, _) = PartitionContentCommitment::capture(
        &head,
        &symbols,
        Some((&commitment, &previous, &journal)),
    )
    .unwrap();
    assert_content(&head, &oracle, &symbols, &next);
    assert_eq!(fixture.source.history_reads.load(Ordering::SeqCst), 1);
    assert_eq!(other_source.history_reads.load(Ordering::SeqCst), 0);
}

#[test]
fn removing_a_prefix_discards_its_opaque_tree_before_rematerialization() {
    let fixture = region_with_history();
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let head = region_load::load_head(&fixture.cold, execution, 1 << 24).unwrap();
    let full = region_load::load_full(&fixture.cold, &head, execution, 1 << 24).unwrap();
    let symbols = fixture.cold.symbols.interner_snapshot();
    assert_eq!(
        head.partition.authoritative_content_digest(&symbols),
        Err(PartitionContentDigestError::InvalidHistoryCoverage)
    );
    let slot = fixture.entity.slot_index();
    let journal = PartitionMutationJournal {
        entity_slots: BTreeSet::from([slot]),
        ..Default::default()
    };
    let mut written = head.partition.clone();
    let mut oracle = full.partition.clone();
    written.entity_arena.suspend_materialization(slot).unwrap();
    oracle.entity_arena.suspend_materialization(slot).unwrap();
    let (suspended, _) = PartitionContentCommitment::capture(
        &written,
        &symbols,
        Some((head.commitment.as_ref().unwrap(), &head.partition, &journal)),
    )
    .unwrap();
    assert_content(&written, &oracle, &symbols, &suspended);
    let previous = written.clone();
    let extra = head
        .partition
        .entity_arena
        .get_slot(slot)
        .unwrap()
        .extra()
        .clone();
    for partition in [&mut written, &mut oracle] {
        partition
            .entity_arena
            .rematerialize(&fixture.entity, KindId(1), VersionId(2_000), extra.clone())
            .unwrap();
    }
    let (rematerialized, _) = PartitionContentCommitment::capture(
        &written,
        &symbols,
        Some((&suspended, &previous, &journal)),
    )
    .unwrap();
    assert_content(&written, &oracle, &symbols, &rematerialized);
}

#[test]
fn an_adopted_opening_is_independent_of_the_destination_physical_row() {
    let fixture = region_with_history();
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let head = region_load::load_head(&fixture.cold, execution, 1 << 24).unwrap();
    let full = region_load::load_full(&fixture.cold, &head, execution, 1 << 24).unwrap();
    let symbols = fixture.cold.symbols.interner_snapshot();
    let mut destination = head.partition.clone();
    destination.entity_arena = EntityArena::with_capacity(0);
    destination
        .entity_arena
        .write_reserved_slot(
            SlotInit {
                partition_id: fixture.entity.partition_id,
                kind_id: KindId(1),
                version_id: VersionId(2_000),
                extra: EntityExtra::default(),
            },
            90,
            1,
        )
        .unwrap();
    let mut oracle = destination.clone();
    oracle.relation_arena = full.partition.relation_arena.clone();
    let selected = BTreeSet::from([fixture.entity.slot_index()]);
    destination
        .entity_arena
        .merge_slots_from_owned(&mut head.partition.entity_arena.clone(), &selected);
    oracle
        .entity_arena
        .merge_slots_from_owned(&mut full.partition.entity_arena.clone(), &selected);
    let physical = destination
        .entity_arena
        .physical_index(fixture.entity.slot_index())
        .unwrap();
    assert_ne!(
        physical,
        destination.entity_arena.history_prefixes[physical]
            .as_ref()
            .unwrap()
            .row_index
    );
    let (commitment, _) =
        PartitionContentCommitment::capture_layered(&destination, &symbols).unwrap();
    assert_content(&destination, &oracle, &symbols, &commitment);
}

#[test]
fn relation_adoption_and_append_preserve_the_independent_entity_prefixes() {
    let fixture = region_with_history();
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let first = region_load::load_head(&fixture.cold, execution, 1 << 24).unwrap();
    let full = region_load::load_full(&fixture.cold, &first, execution, 1 << 24).unwrap();
    let mut later = full.partition.clone();
    let slot = fixture.relation.slot_index();
    let extra = later.relation_arena.get_slot(slot).unwrap().extra().clone();
    later
        .relation_arena
        .apply_extra_update(slot, extra, VersionId(2_000));
    let (cold, source) = recapture(&fixture.cold, &later);
    let second = region_load::load_head(&cold, execution, 1 << 24).unwrap();
    let symbols = fixture.cold.symbols.interner_snapshot();
    let selected = BTreeSet::from([slot]);
    let journal = PartitionMutationJournal {
        relation_slots: selected.clone(),
        ..Default::default()
    };
    let mut head = first.partition.clone();
    let mut oracle = full.partition.clone();
    head.relation_arena
        .merge_slots_from_owned(&mut second.partition.relation_arena.clone(), &selected);
    oracle
        .relation_arena
        .merge_slots_from_owned(&mut later.relation_arena.clone(), &selected);
    let (adopted, _) = PartitionContentCommitment::capture(
        &head,
        &symbols,
        Some((
            first.commitment.as_ref().unwrap(),
            &first.partition,
            &journal,
        )),
    )
    .unwrap();
    assert_content(&head, &oracle, &symbols, &adopted);
    let previous = head.clone();
    let extra = head.relation_arena.get_slot(slot).unwrap().extra().clone();
    head.relation_arena
        .apply_extra_update(slot, extra.clone(), VersionId(3_000));
    oracle
        .relation_arena
        .apply_extra_update(slot, extra, VersionId(3_000));
    let (appended, _) =
        PartitionContentCommitment::capture(&head, &symbols, Some((&adopted, &previous, &journal)))
            .unwrap();
    assert_content(&head, &oracle, &symbols, &appended);
    assert_eq!(source.history_reads.load(Ordering::SeqCst), 0);
}

pub(super) fn recapture(
    cold: &RelationalColdRegion,
    partition: &PartitionState,
) -> (RelationalColdRegion, Arc<CountedSource>) {
    let symbols = cold.symbols.interner_snapshot();
    let (commitment, _) = PartitionContentCommitment::capture(partition, &symbols, None).unwrap();
    let members = capture_region_members(
        partition,
        &commitment,
        &cold.schema,
        cold.schema_carrier_digest,
    )
    .unwrap();
    let source = Arc::new(CountedSource {
        history: members.history.0,
        members: BTreeMap::from([members.head.clone(), members.history.clone()]),
        reads: AtomicUsize::new(0),
        history_reads: AtomicUsize::new(0),
        corrupt: AtomicBool::new(false),
    });
    (
        RelationalColdRegion {
            head: members.head.0,
            history: members.history.0,
            kind_authority: members.kind_authority,
            content_digest: commitment.digest(partition.partition_id),
            source: source.clone(),
            ..cold.clone()
        },
        source,
    )
}

fn assert_content(
    head: &PartitionState,
    full: &PartitionState,
    symbols: &crate::symbols::data::StringInterner,
    incremental: &PartitionContentCommitment,
) {
    let oracle = full.authoritative_content_digest(symbols).unwrap();
    assert_eq!(incremental.digest(head.partition_id), oracle);
    assert_eq!(
        PartitionContentCommitment::capture_layered(head, symbols)
            .unwrap()
            .0
            .digest(head.partition_id),
        oracle
    );
}
