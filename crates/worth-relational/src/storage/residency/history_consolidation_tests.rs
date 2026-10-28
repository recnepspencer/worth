//! Repeated real member placement and readmission exercise capture's fold bound.
use super::layered_history_tests::{counted, place};
use super::*;
use crate::durability::data::RelationalHistoryMemberDirectory;
use crate::storage::residency::history_capture_bound::{
    HISTORY_CAPTURE_INPUT_TARGET, HISTORY_CAPTURE_MAX_LAYERS,
};

#[test]
fn a_legacy_overdeep_cold_binding_is_consolidated_instead_of_carried() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let fixture = region_with_history();
    let mut cold = fixture.cold;
    for _ in 0..HISTORY_CAPTURE_MAX_LAYERS {
        let cell = RelationalRegionResidency::cold(cold.clone()).unwrap();
        let head = head_demand(&cell, 1 << 24).unwrap();
        cold = layered_history_tests::encode_layer(
            &cold,
            &counted(archive(&cold), cold.history),
            &head.resident_region().partition,
        )
        .0;
    }
    let cell = RelationalRegionResidency::cold(cold.clone()).unwrap();
    let destination = archive(&cold).keys().copied().collect();
    let captured = cell
        .capture_checkpoint(
            execution,
            &cold.schema,
            cold.schema_carrier_digest,
            &cold.symbols.interner_snapshot(),
            Some(&destination),
        )
        .unwrap();
    assert!(captured.history_dependencies.is_empty());
    assert_eq!(captured.leaf.content_digest, fixture.expected_digest);
}

#[test]
fn restart_cycles_keep_full_history_input_bounded_with_headroom() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let fixture = region_with_history();
    let flat_bytes = fixture.cold.history.length;
    let mut cold = fixture.cold.clone();
    let mut consolidations = 0;
    for _ in 0..(HISTORY_CAPTURE_MAX_LAYERS * 8) {
        let cell = RelationalRegionResidency::cold(cold.clone()).unwrap();
        head_demand(&cell, 1 << 24).unwrap();
        let prior = cold.history_members.fold_span(cold.history).0;
        let captured = capture_next(execution, &cell, &cold);
        let directory = Arc::new(
            RelationalHistoryMemberDirectory::readmit(
                &captured.history_dependencies,
                [captured.leaf.history],
            )
            .unwrap(),
        );
        let (layers, input) = directory.fold_span(captured.leaf.history);
        assert!(layers < HISTORY_CAPTURE_MAX_LAYERS + 1);
        assert!(input < HISTORY_CAPTURE_INPUT_TARGET);
        // This independent byte sum catches growth even if depth accounting lies.
        let members = place(&captured.members, &archive(&cold));
        let actual = directory
            .members(captured.leaf.history)
            .iter()
            .map(|member| members[member].len() as u64)
            .sum::<u64>();
        assert_eq!(input, actual);
        assert!(actual < flat_bytes * (HISTORY_CAPTURE_MAX_LAYERS as u64 + 4));
        consolidations += usize::from(layers < prior);
        let source = counted(members, captured.leaf.history);
        cold = RelationalColdRegion {
            source,
            head: captured.leaf.head,
            history: captured.leaf.history,
            history_members: directory,
            kind_authority: captured.leaf.kind_authority,
            ..cold
        };
        let full_cell = RelationalRegionResidency::cold(cold.clone()).unwrap();
        let full = full_demand(&full_cell, 1 << 24).unwrap();
        assert_eq!(
            full.full_view()
                .partition()
                .authoritative_content_digest(&cold.symbols.interner_snapshot())
                .unwrap(),
            fixture.expected_digest
        );
        assert_eq!(
            full.full_view()
                .partition()
                .entity_arena
                .metadata_history_at(fixture.entity.slot_index())
                .unwrap()
                .len(),
            fixture.expected_history
        );
    }
    assert!(
        consolidations >= 4,
        "the fixture must cross the policy repeatedly"
    );
}

#[test]
fn consolidation_damage_is_sticky_and_a_stopped_capture_is_retryable() {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let fixture = region_with_history();
    let mut cold = fixture.cold;
    for _ in 1..HISTORY_CAPTURE_MAX_LAYERS {
        let cell = RelationalRegionResidency::cold(cold.clone()).unwrap();
        head_demand(&cell, 1 << 24).unwrap();
        let captured = capture_next(execution, &cell, &cold);
        cold = reopened(&cold, captured);
    }
    let cell = RelationalRegionResidency::cold(cold.clone()).unwrap();
    head_demand(&cell, 1 << 24).unwrap();
    let cancellation = worth_execution::CancellationSource::new();
    let stopped = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(None),
        cancellation.token(),
        None,
    );
    cancellation.cancel();
    let stopped = worth_execution::ExecutionRequest::serial(&stopped);
    let destination = archive(&cold)
        .keys()
        .copied()
        .filter(|member| *member != cold.head)
        .collect();
    let refusal = cell.capture_checkpoint(
        &stopped,
        &cold.schema,
        cold.schema_carrier_digest,
        &cold.symbols.interner_snapshot(),
        Some(&destination),
    );
    assert!(matches!(
        refusal,
        Err(RelationalResidencyDemandFailure::Request(_))
    ));
    cell.check_damage().unwrap();
    let successful = capture_next(execution, &cell, &cold);
    assert!(successful.history_dependencies.is_empty());

    // A different cold cell must actually read the corrupt prefix at compaction.
    let damaged_source = counted(archive(&cold), cold.history);
    let damaged = RelationalRegionResidency::cold(RelationalColdRegion {
        source: damaged_source.clone(),
        ..cold.clone()
    })
    .unwrap();
    head_demand(&damaged, 1 << 24).unwrap();
    damaged_source.corrupt.store(true, Ordering::SeqCst);
    for attempt in 0..2 {
        let before = damaged_source.reads.load(Ordering::SeqCst);
        let refusal = damaged.capture_checkpoint(
            execution,
            &cold.schema,
            cold.schema_carrier_digest,
            &cold.symbols.interner_snapshot(),
            Some(&destination),
        );
        assert!(
            matches!(refusal, Err(RelationalResidencyDemandFailure::StorageUnavailable(failure))
            if failure.subject == RelationalStorageSubject::Region(cold.partition_id)
                && failure.cause.is_proven_damage())
        );
        if attempt > 0 {
            assert_eq!(before, damaged_source.reads.load(Ordering::SeqCst));
        }
    }
}

fn archive(cold: &RelationalColdRegion) -> BTreeMap<RelationalDurableMemberLocator, Vec<u8>> {
    cold.history_members
        .members(cold.history)
        .into_iter()
        .chain([cold.head])
        .map(|member| (member, cold.source.read_member(member, 1 << 24).unwrap()))
        .collect()
}

fn capture_next(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    cell: &Arc<RelationalRegionResidency>,
    cold: &RelationalColdRegion,
) -> super::super::checkpoint_capture::RelationalCapturedRegionCheckpoint {
    // Excluding Head models a destination that must re-encode the admitted row.
    let destination = cold.history_members.members(cold.history);
    cell.capture_checkpoint(
        execution,
        &cold.schema,
        cold.schema_carrier_digest,
        &cold.symbols.interner_snapshot(),
        Some(&destination),
    )
    .unwrap()
}

fn reopened(
    cold: &RelationalColdRegion,
    captured: super::super::checkpoint_capture::RelationalCapturedRegionCheckpoint,
) -> RelationalColdRegion {
    let source = counted(
        place(&captured.members, &archive(cold)),
        captured.leaf.history,
    );
    RelationalColdRegion {
        source,
        head: captured.leaf.head,
        history: captured.leaf.history,
        history_members: Arc::new(
            RelationalHistoryMemberDirectory::readmit(
                &captured.history_dependencies,
                [captured.leaf.history],
            )
            .unwrap(),
        ),
        kind_authority: captured.leaf.kind_authority,
        ..cold.clone()
    }
}
