use super::*;
use crate::durability::medium::{RelationalDurableColdSource, RelationalDurableMemberLocator};
use crate::identity::data::KindId;
use crate::publication::patch::data::PatchStreamPosition;
use crate::storage::data::{RelationalStorageSubject, RelationalStorageUnavailableCause as Cause};
use crate::storage::overlay::PartitionContentCommitment;
use crate::tests::support::*;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

#[path = "cross_region_forgery_tests.rs"]
mod cross_region_forgery;
#[path = "head_history_kind_tests.rs"]
mod head_history_kind_tests;
#[path = "head_metadata_tests.rs"]
mod head_metadata_tests;
#[path = "history_fold_refusals.rs"]
mod history_fold_refusals;
#[path = "history_prefix_tests.rs"]
mod history_prefix_tests;
#[path = "layered_history_tests.rs"]
mod layered_history_tests;
#[path = "lease_transfer_tests.rs"]
mod lease_transfer_tests;
#[path = "level_tests.rs"]
mod level_tests;
#[path = "member_custody_tests.rs"]
mod member_custody_tests;
#[path = "mixed_history_tests.rs"]
mod mixed_history_tests;
#[path = "projection_corruption_tests.rs"]
mod projection_corruption;
#[path = "symbol_admission_tests.rs"]
mod symbol_admission_tests;

#[path = "region_fixture.rs"]
mod region_fixture;
#[path = "resident_history_tests.rs"]
mod resident_history_tests;
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
        let foreign = crate::identity::data::EntityId::new(
            crate::identity::data::PartitionId(77),
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
            .commitment()
            .digest(fixture.entity.partition_id),
        fixture.expected_digest
    );
}

#[test]
fn intact_history_member_with_relabelled_content_is_refused_against_the_leaf() {
    use super::region_image::{
        RegionHistoryImage, RelationalRegionImage, RelationalRegionImageKind,
    };
    let fixture = region_with_history();
    let mut cold = fixture.cold;
    let bytes = fixture.source.members.get(&cold.history).unwrap();
    let frame = RelationalRegionImage::decode(
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
        cold.partition_id,
        cold.schema_carrier_digest,
        RelationalRegionImageKind::History,
        &history,
    )
    .unwrap();
    let mut members = fixture.source.members.clone();
    members.insert(locator, bytes);
    cold.history = locator;
    cold.source = Arc::new(CountedSource {
        members,
        reads: AtomicUsize::new(0),
        history_reads: AtomicUsize::new(0),
        history: locator,
        corrupt: AtomicBool::new(false),
    });
    let cell = RelationalRegionResidency::cold(cold).unwrap();
    let head = head_demand(&cell, 1 << 24).unwrap();
    assert!(head.head_view().entity(fixture.entity).is_some());
    assert_eq!(
        full_demand(&cell, 1 << 24).unwrap_err().cause,
        Cause::IntegrityMismatch
    );
}

#[test]
fn damaged_content_and_leaf_mismatches_are_sticky_typed_refusals() {
    let fixture = region_with_history();
    fixture.source.corrupt.store(true, Ordering::SeqCst);
    let first = head_demand(&fixture.cell, 1 << 24).unwrap_err();
    assert_eq!(
        first.subject,
        RelationalStorageSubject::Region(fixture.entity.partition_id)
    );
    assert_eq!(first.cause, Cause::IntegrityMismatch);
    fixture.source.corrupt.store(false, Ordering::SeqCst);
    let reads = fixture.source.reads.load(Ordering::SeqCst);
    assert_eq!(full_demand(&fixture.cell, 1 << 24).unwrap_err(), first);
    assert_eq!(fixture.source.reads.load(Ordering::SeqCst), reads);
    assert_eq!(fixture.cell.leases.load(Ordering::SeqCst), 0);

    let mut wrong_leaf = fixture.cold;
    wrong_leaf.content_digest[0] ^= 1;
    let cell = RelationalRegionResidency::cold(wrong_leaf).unwrap();
    assert_eq!(
        head_demand(&cell, 1 << 24).unwrap_err().cause,
        Cause::IntegrityMismatch
    );
}

#[test]
fn region_limit_refuses_before_admitting_a_lease_and_recovery_preserves_the_cause() {
    let fixture = region_with_history();
    let failure = head_demand(&fixture.cell, 1).unwrap_err();
    assert!(matches!(
        failure.cause,
        Cause::LimitExceeded { limit: 1, .. }
    ));
    let crate::durability::data::RelationalRecoveryStop::Durability(error) =
        crate::durability::data::RelationalRecoveryStop::from(failure)
    else {
        panic!("durability stop")
    };
    assert_eq!(
        error.class,
        crate::durability::data::RecoveryFailureClass::ColdStorageUnavailable
    );
    assert_eq!(error.storage_unavailable, Some(failure));
    assert_eq!(fixture.cell.leases.load(Ordering::SeqCst), 0);
}

#[path = "kind_authority_tests.rs"]
mod kind_authority_tests;
#[path = "root_view_tests.rs"]
mod root_view_tests;

#[test]
fn capture_of_a_cold_region_carries_members_without_loading_or_leasing() {
    let fixture = region_with_history();
    let captured = fixture
        .cell
        .capture_checkpoint(
            &fixture.cold.schema,
            fixture.cold.schema_carrier_digest,
            &fixture.cold.symbols.interner_snapshot(),
            None,
        )
        .unwrap();
    assert_eq!(captured.leaf.content_digest, fixture.expected_digest);
    assert!(captured.members.iter().all(|member| matches!(
        member,
        crate::durability::data::RelationalCheckpointMember::Carried { .. }
    )));
    assert_eq!(fixture.source.reads.load(Ordering::SeqCst), 0);
    assert_eq!(fixture.cell.leases.load(Ordering::SeqCst), 0);
}

fn head_demand(
    cell: &Arc<RelationalRegionResidency>,
    limit: u64,
) -> Result<RelationalResidencyLease<Head>, crate::storage::data::RelationalStorageUnavailable> {
    let request = crate::test_execution_policy::serial_request();
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
) -> Result<RelationalResidencyLease<Full>, crate::storage::data::RelationalStorageUnavailable> {
    let request = crate::test_execution_policy::serial_request();
    cell.demand_full(
        &worth_execution::ExecutionRequest::serial(&request),
        limit,
        None,
    )
    .map_err(storage_failure)
}
fn storage_failure(
    failure: RelationalResidencyDemandFailure,
) -> crate::storage::data::RelationalStorageUnavailable {
    match failure {
        RelationalResidencyDemandFailure::StorageUnavailable(failure) => failure,
        other => panic!("unbounded live test demand was interrupted: {other:?}"),
    }
}

#[path = "request_stop_tests.rs"]
mod request_stop_tests;

#[path = "probe_tests.rs"]
mod probe_tests;

#[path = "head_probe_tests.rs"]
mod head_probe_tests;
