//! The packetized public SDK kind reader consumes the same maintained directory.
use super::*;
use worth_relational::facade::{query::*, transactions::RecordRef};
pub(super) fn work(
    world: &crate::content::historical_kind::fixture::World,
    request: &ExecutionRequest<'_, '_>,
    original: &SerialRequest,
    expected: &[worth_relational::facade::identity::EntityId],
) -> u64 {
    let (_, basis) = world
        .runtime
        .observe_branch(&world.runtime.main_branch_identity())
        .unwrap();
    let handle = world
        .runtime
        .snapshots()
        .snapshot_for_observation(&basis.observation())
        .unwrap();
    let read = world
        .runtime
        .read_truth()
        .pin_read_capability(&handle, &[], request)
        .unwrap();
    let mut packet = PlannedQueryPacket::explicit_targets(
        "directory query",
        read.query_context().clone(),
        Vec::<RecordRef>::new(),
    );
    packet.scope = QueryScope::EntityKindScan {
        kind_id: KindId(1),
        partition_scope: None,
    };
    packet.ordering = QueryOrderingContract::CanonicalEntityIdOrder;
    let plan = world
        .runtime
        .read_truth()
        .plan_query_packet(&handle, packet)
        .unwrap();
    let (result, report) = ExecutionWorkCeiling::new(None)
        .run_serial(original, || {
            read.execute_query_plan(plan, request).unwrap().unwrap()
        })
        .unwrap();
    assert_eq!(
        result
            .outcome
            .result
            .entities
            .iter()
            .map(|row| row.entity_id)
            .collect::<Vec<_>>(),
        expected
    );
    world.runtime.snapshots().release_snapshot(&handle).unwrap();
    report.charged_work()
}
