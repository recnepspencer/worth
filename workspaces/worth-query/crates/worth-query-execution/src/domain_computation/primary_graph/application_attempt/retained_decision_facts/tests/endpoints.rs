use super::super::{endpoints::AdmittedAdjacencyEndpoints, StoreDenial};
use super::control;
use super::custody::{authority, request, SERIAL};
use std::alloc::Layout;
use worth_execution::{
    ExecutionAllocationDenialKind as Kind, ExecutionAllocationPolicy as Policy, LeaseDenial,
};
use worth_relational::facade::identity::{EntityId, PartitionId};
fn id(slot: u64) -> EntityId {
    EntityId::new(PartitionId::main(), slot, 1)
}

#[test]
fn retained_endpoint_capture_and_union_keep_full_exact_ids_and_charge_coexistence() {
    if !super::custody::isolated(
        "retained_endpoint_capture_and_union_keep_full_exact_ids_and_charge_coexistence",
        module_path!(),
    ) {
        return;
    }
    let _serial = SERIAL.lock().unwrap();
    let quote = u64::try_from(Layout::array::<EntityId>(3).unwrap().size()).unwrap();
    let retained_two = u64::try_from(Layout::array::<EntityId>(2).unwrap().size()).unwrap();
    let budget = quote + retained_two;
    let parent = authority().request_lease(request(budget)).unwrap();
    let left = AdmittedAdjacencyEndpoints::capture(
        &[id(3), id(1), id(3)],
        control(Policy::Execution(&parent)),
    )
    .unwrap();
    assert_eq!(&*left, &[id(1), id(3)]);
    let shared = left.clone();
    let right =
        AdmittedAdjacencyEndpoints::capture(&[id(4), id(3)], control(Policy::SystemAllocation))
            .unwrap();
    // The parent holds the old two-ID backing while its child admits the new
    // three-ID backing. These charges coexist at the common parent ledger.
    let child = parent.child(request(quote)).unwrap();
    let union = left
        .union(&right, control(Policy::Execution(&child)))
        .unwrap();
    assert_eq!(&*union, &[id(1), id(3), id(4)]);
    drop(child);
    drop(left);
    assert_eq!(shared.charged_payload_bytes(), Some(retained_two));
    assert_eq!(union.charged_payload_bytes(), Some(quote));
    drop(shared);
    drop(union);
    assert_eq!(
        parent.reserve_memory(budget).unwrap().charged_bytes(),
        budget
    );
    let short = authority().request_lease(request(quote)).unwrap();
    let held =
        AdmittedAdjacencyEndpoints::capture(&[id(1)], control(Policy::Execution(&short))).unwrap();
    let incoming =
        AdmittedAdjacencyEndpoints::capture(&[id(2), id(3)], control(Policy::SystemAllocation))
            .unwrap();
    let denied = match held.union(&incoming, control(Policy::Execution(&short))) {
        Err(StoreDenial::Allocation(denied)) => denied,
        _ => panic!("old endpoint backing must remain charged during replacement"),
    };
    assert_eq!(denied.kind(), Kind::Lease(LeaseDenial::ResourceExhausted));
    assert_eq!(denied.requested_payload_bytes(), Some(quote));
    assert_eq!(&*held, &[id(1)]);
    drop(held);
    assert_eq!(short.reserve_memory(quote).unwrap().charged_bytes(), quote);
}
