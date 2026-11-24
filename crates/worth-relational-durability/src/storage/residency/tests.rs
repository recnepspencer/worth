use super::*;
use contracts::identity::data::primitives::KindId;
use contracts::publication::patch::data::patch_position::PatchStreamPosition;
use contracts::storage::data::storage_unavailable::{
    RelationalStorageSubject, RelationalStorageUnavailableCause as Cause,
};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use worth_relational_branch::internal_port::branch::root::capture::with_reconstruction_fixture_for_test;
use worth_relational_contracts::internal_port as contracts;
use worth_relational_storage::internal_port::durability::medium::cold_source::{
    RelationalDurableColdSource, RelationalDurableMemberLocator,
};
use worth_relational_storage::internal_port::storage::overlay::partition_content::PartitionContentCommitment;
use worth_relational_storage::internal_port::storage::residency::views::RelationalRegionFullViewFixturePort as _;

#[path = "head_history_kind_tests.rs"]
pub mod head_history_kind_tests;
#[path = "head_metadata_tests.rs"]
pub mod head_metadata_tests;
#[path = "history_fold_refusals.rs"]
pub mod history_fold_refusals;
#[path = "history_prefix_tests.rs"]
pub mod history_prefix_tests;
#[path = "layered_history_tests.rs"]
pub mod layered_history_tests;
#[path = "lease_transfer_tests.rs"]
pub mod lease_transfer_tests;
#[path = "member_custody_tests.rs"]
pub mod member_custody_tests;
#[path = "mixed_history_tests.rs"]
pub mod mixed_history_tests;
#[path = "projection_corruption_tests.rs"]
pub mod projection_corruption;
#[path = "symbol_admission_tests.rs"]
pub mod symbol_admission_tests;

#[path = "region_fixture/checkpoint_inputs.rs"]
pub mod checkpoint_inputs;
#[path = "history_consolidation_tests.rs"]
pub mod history_consolidation_tests;
#[path = "region_fixture.rs"]
pub mod region_fixture;
#[path = "resident_history_tests.rs"]
pub mod resident_history_tests;
use region_fixture::{region_with_history, CountedSource, RegionFixture};

#[test]
fn concurrent_head_demands_share_the_load_and_count_every_lease() {
    let fixture = region_with_history();
    assert_eq!(fixture.source.reads.load(Ordering::SeqCst), 0);
    let leases = std::thread::scope(|scope| {
        let handles = (0..8)
            .map(|_| {
                let fixture = &fixture;
                scope.spawn(move || head_demand(&fixture.cell, 1 << 24).unwrap())
            })
            .collect::<Vec<_>>();
        handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .collect::<Vec<_>>()
    });
    assert!(
        fixture.source.reads.load(Ordering::SeqCst) < 4,
        "one region load must serve concurrent readers"
    );
    assert_eq!(
        fixture.source.history_reads.load(Ordering::SeqCst),
        0,
        "head demand must never read history"
    );
    assert_eq!(fixture.cell.leases.load(Ordering::SeqCst), leases.len());
    for lease in &leases {
        assert_eq!(
            lease
                .head_view()
                .entity(fixture.entity)
                .unwrap()
                .generation(),
            fixture.entity.generation_value()
        );
        let view = lease.head_view();
        assert!(view.relation(fixture.relation).is_some());
        assert_eq!(
            view.outgoing(fixture.entity).to_vec(),
            vec![fixture.relation]
        );
        let foreign = contracts::identity::data::record_ids::EntityId::new(
            contracts::identity::data::primitives::PartitionId(77),
            fixture.entity.slot_index() as u64,
            fixture.entity.generation_value(),
        );
        assert!(view.outgoing(foreign).to_vec().is_empty());
    }
    drop(leases);
    assert_eq!(fixture.cell.leases.load(Ordering::SeqCst), 0);
    let full = full_demand(&fixture.cell, 1 << 24).unwrap();
    assert_eq!(
        full.full_view()
            .partition()
            .entity_arena
            .metadata_history_at(fixture.entity.slot_index())
            .unwrap()
            .len(),
        fixture.expected_history
    );
    assert_eq!(
        full.full_view()
            .full_view_commitment_for_test()
            .digest(fixture.entity.partition_id),
        fixture.expected_digest
    );
}

#[test]
fn intact_history_member_with_relabelled_content_is_refused_against_the_leaf() {
    with_reconstruction_fixture_for_test(|fixture_lane| {
        use worth_relational_storage::internal_port::storage::residency::region_image::{
            RegionHistoryImage, RelationalRegionImage, RelationalRegionImageKind,
        };
        let fixture = region_with_history();
        let mut cold = fixture.cold;
        let bytes = fixture.source.members.get(&cold.history).unwrap();
        let frame = RelationalRegionImage::decode(
            fixture_lane,
            &mut |_, _| Ok(()),
            bytes,
            cold.history,
            cold.partition_id,
            cold.schema_carrier_digest,
            RelationalRegionImageKind::History,
        )
        .unwrap();
        let mut history: RegionHistoryImage = rmp_serde::from_slice(frame.payload).unwrap();
        history.entities[0].earlier[0].generation ^= 1;
        let (locator, bytes) = RelationalRegionImage::encode(
            fixture_lane,
            &mut |_, _| Ok(()),
            cold.partition_id,
            cold.schema_carrier_digest,
            RelationalRegionImageKind::History,
            &history,
        )
        .unwrap();
        let mut members = fixture.source.members.clone();
        members.insert(locator, bytes);
        cold.history = locator;
        cold.source = Some(Arc::new(CountedSource {
            members,
            reads: AtomicUsize::new(0),
            history_reads: AtomicUsize::new(0),
            history: locator,
            corrupt: AtomicBool::new(false),
        }));
        let cell = RelationalRegionResidency::cold(cold).unwrap();
        let head = head_demand(&cell, 1 << 24).unwrap();
        assert!(head.head_view().entity(fixture.entity).is_some());
        assert_eq!(
            full_demand(&cell, 1 << 24).unwrap_err().cause,
            Cause::IntegrityMismatch
        );
    })
}

#[path = "residency_refusal_tests.rs"]
mod residency_refusal_tests;

#[path = "kind_authority_tests.rs"]
pub mod kind_authority_tests;
#[path = "root_view_tests.rs"]
pub mod root_view_tests;

#[test]
fn capture_of_a_cold_region_carries_members_without_loading_or_leasing() {
    let request = worth_harness::facade::execution_fixture::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let fixture = region_with_history();
    let captured = fixture
        .cell
        .capture_checkpoint(
            execution,
            fixture.cold.schema.as_ref(),
            fixture.cold.schema_carrier_digest,
            &fixture.cold.symbols.interner_snapshot(),
            fixture.cold.readmission.capture_members,
            None,
        )
        .unwrap();
    assert_eq!(captured.leaf.content_digest, fixture.expected_digest);
    assert!(captured.members.iter().all(|member| matches!(
        member,
        worth_relational_storage::internal_port::storage::data::checkpoint_member::RelationalCheckpointMember::Carried { .. }
    )));
    assert_eq!(fixture.source.reads.load(Ordering::SeqCst), 0);
    assert_eq!(fixture.cell.leases.load(Ordering::SeqCst), 0);
}

fn head_demand(
    cell: &Arc<RelationalRegionResidency>,
    limit: u64,
) -> Result<
    RelationalResidencyLease<Head>,
    contracts::storage::data::storage_unavailable::RelationalStorageUnavailable,
> {
    let request = worth_harness::facade::execution_fixture::serial_request();
    cell.demand_head(
        &worth_execution::ExecutionRequest::serial(&request),
        limit,
        None,
    )
    .map_err(storage_failure)
}
fn full_demand(
    cell: &Arc<RelationalRegionResidency>,
    limit: u64,
) -> Result<
    RelationalResidencyLease<Full>,
    contracts::storage::data::storage_unavailable::RelationalStorageUnavailable,
> {
    let request = worth_harness::facade::execution_fixture::serial_request();
    cell.demand_full(
        &worth_execution::ExecutionRequest::serial(&request),
        limit,
        None,
    )
    .map_err(storage_failure)
}
fn storage_failure(
    failure: RelationalResidencyDemandFailure,
) -> contracts::storage::data::storage_unavailable::RelationalStorageUnavailable {
    match failure {
        RelationalResidencyDemandFailure::StorageUnavailable(failure) => failure,
        other => panic!("unbounded live test demand was interrupted: {other:?}"),
    }
}

#[path = "head_probe_tests.rs"]
pub mod head_probe_tests;
#[path = "probe_tests.rs"]
pub mod probe_tests;
#[path = "request_stop_tests.rs"]
pub mod request_stop_tests;

#[test]
fn reconstructed_heads_reuse_original_custody_without_serving_and_retire_it() {
    with_reconstruction_fixture_for_test(|fixture_lane| {
        use worth_relational_storage::internal_port::storage::residency::source_retirement::OriginalResidentSlot::RawHead;
        let fixture = region_with_history();
        let host = worth_harness::facade::execution_fixture::serial_request();
        let execution = worth_execution::ExecutionRequest::serial(&host);
        let checkpoint = checkpoint_inputs::readmit(&fixture, &execution);
        let cell = checkpoint.cell;
        let source = checkpoint.source;
        let active = checkpoint.active;
        let lane = fixture_lane;
        let first = cell.reconstruct_head(lane, &execution, None).unwrap();
        assert!(first.view().entity(fixture.entity).is_some());
        let original = cell.source_retirement_resident(RawHead).unwrap().unwrap();
        assert!(first
            .view()
            .same_region_view(RelationalRegionHeadView::new(&original)));
        let retained = Arc::downgrade(&original);
        drop(original);
        drop(first);
        let second = cell.reconstruct_head(lane, &execution, None).unwrap();
        let original = retained
            .upgrade()
            .expect("original cell retains authenticated rows");
        assert!(second
            .view()
            .same_region_view(RelationalRegionHeadView::new(&original)));
        assert_eq!(checkpoint.archive.reads.load(Ordering::SeqCst), 1);
        assert_eq!(checkpoint.archive.history_reads.load(Ordering::SeqCst), 0);
        assert_eq!(cell.leases.load(Ordering::SeqCst), 0);
        assert!(!cell.has_checkpoint_contents_for_test().unwrap());
        assert!(active.load(Ordering::SeqCst) > 0);
        drop(original);
        drop(second);
        for pass in [NativeSourcePass::Validate, NativeSourcePass::Retire] {
            let mut retirement = NativeRegionSourceRetirement::new(cell.clone(), pass);
            while retirement.advance(&execution, source.as_ref()).unwrap()
                != NativeRegionSourceRetirementProgress::Complete
            {}
        }
        assert_eq!(active.load(Ordering::SeqCst), 0);
        assert!(matches!(cell.reconstruct_head(lane, &execution, None),
        Err(RelationalResidencyDemandFailure::StorageUnavailable(error))
            if error.cause == Cause::SourceUnavailable));
        drop(cell);
        assert!(retained.upgrade().is_none());
    })
}

#[test]
fn reconstructed_heads_do_not_alias_distinct_cells_with_equal_partition_ids() {
    with_reconstruction_fixture_for_test(|fixture_lane| {
        let fixtures = [region_with_history(), region_with_history()];
        assert_eq!(fixtures[0].cold.partition_id, fixtures[1].cold.partition_id);
        let host = worth_harness::facade::execution_fixture::serial_request();
        let execution = worth_execution::ExecutionRequest::serial(&host);
        let lane = fixture_lane;
        let first = fixtures[0]
            .cell
            .reconstruct_head(lane, &execution, None)
            .unwrap();
        let second = fixtures[1]
            .cell
            .reconstruct_head(lane, &execution, None)
            .unwrap();
        assert!(!first.view().same_region_view(second.view()));
        for fixture in &fixtures {
            assert_eq!(fixture.source.reads.load(Ordering::SeqCst), 1);
            assert_eq!(fixture.cell.leases.load(Ordering::SeqCst), 0);
            assert!(!fixture.cell.has_checkpoint_contents_for_test().unwrap());
        }
    })
}

#[test]
fn reconstructed_heads_check_stopped_requests_and_retain_only_proven_damage() {
    with_reconstruction_fixture_for_test(|fixture_lane| {
        let fixture = region_with_history();
        let cell = &fixture.cell;
        let cancellation = worth_execution::CancellationSource::new();
        cancellation.cancel();
        let stopped = worth_harness::facade::execution_fixture::serial_request()
            .with_cancellation(cancellation.token());
        let stopped = worth_execution::ExecutionRequest::serial(&stopped);
        let host = worth_harness::facade::execution_fixture::serial_request();
        let execution = worth_execution::ExecutionRequest::serial(&host);
        let lane = fixture_lane;
        for expected_reads in [0, 1] {
            assert!(matches!(
                cell.reconstruct_head(lane, &stopped, None),
                Err(RelationalResidencyDemandFailure::Request(_))
            ));
            assert_eq!(fixture.source.reads.load(Ordering::SeqCst), expected_reads);
            drop(cell.reconstruct_head(lane, &execution, None).unwrap());
            assert_eq!(fixture.source.reads.load(Ordering::SeqCst), 1);
        }
        let damaged = region_with_history();
        damaged.source.corrupt.store(true, Ordering::SeqCst);
        let first = damaged
            .cell
            .reconstruct_head(lane, &execution, None)
            .err()
            .unwrap();
        assert!(
            matches!(&first, RelationalResidencyDemandFailure::StorageUnavailable(error)
        if error.cause == Cause::IntegrityMismatch)
        );
        damaged.source.corrupt.store(false, Ordering::SeqCst);
        let reads = damaged.source.reads.load(Ordering::SeqCst);
        assert_eq!(
            damaged
                .cell
                .reconstruct_head(lane, &execution, None)
                .err()
                .unwrap(),
            first
        );
        assert_eq!(damaged.source.reads.load(Ordering::SeqCst), reads);
        assert_eq!(damaged.cell.leases.load(Ordering::SeqCst), 0);
    })
}
