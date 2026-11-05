//! Actual control allocations, independently measured, including final Weak.
use super::*;
use crate::identity::data::PartitionId;
use crate::storage::substrate::{EntityArena, RelationArena};
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};

fn partition() -> PartitionState {
    PartitionState {
        partition_id: PartitionId(1),
        adjacency_policy: crate::config::data::AdjacencyPolicy {
            backend: crate::config::data::AdjacencyBackend::InlineSmallDegreeAdjacency,
            small_degree_inline_capacity: 4,
        },
        relation_overlay_is_sparse: false,
        entity_arena: EntityArena::with_capacity(0),
        relation_arena: RelationArena::with_capacity(0),
        adjacency: Default::default(),
        reverse_adjacency: Default::default(),
    }
}

fn allocation<T>() -> u64 {
    let layout = std::alloc::Layout::new::<(usize, usize)>()
        .extend(std::alloc::Layout::new::<T>())
        .unwrap()
        .0
        .pad_to_align();
    u64::try_from(layout.size()).unwrap()
}

#[test]
fn controlled_resident_cell_keeps_exact_charge_until_last_real_weak() {
    let budget = SerialMemoryBudget::new(Some(4 * 1024 * 1024));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let region = allocation::<ResidentRegion>();
    let control =
        allocation::<RelationalRegionResidency>() + allocation::<ExecutionMemoryReservation>();
    let lease = RelationalRegionResidency::resident_for_request(
        partition(),
        PartitionContentCommitment::default(),
        &ExecutionRequest::serial(&host),
    )
    .unwrap();
    assert_eq!(budget.charged_memory_bytes(), region + control);
    let weak = RelationalRegionResidency::downgrade(lease.cell());
    let remaining = weak.clone();
    drop(lease);
    assert!(
        weak.upgrade().is_none(),
        "charge never retains operational cell data"
    );
    assert_eq!(budget.charged_memory_bytes(), control);
    drop(weak);
    assert_eq!(budget.charged_memory_bytes(), control);
    drop(remaining);
    assert_eq!(budget.charged_memory_bytes(), 0);
    println!("regionArc={region} cell+grantArc={control} weak={} rawWeak={} cellFooter={} regionFooter={}",
        std::mem::size_of::<RegionCellWeak>(),
        std::mem::size_of::<std::sync::Weak<RelationalRegionResidency>>(),
        std::mem::size_of::<Option<Arc<ExecutionMemoryReservation>>>(),
        std::mem::size_of::<Option<ExecutionMemoryReservation>>());
}

#[test]
fn controlled_resident_cell_refuses_exact_control_before_any_new_arc() {
    let budget = SerialMemoryBudget::new(Some(4 * 1024 * 1024));
    let calibration =
        SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let mut controller = 0;
    ExecutionRequest::serial(&calibration)
        .run_native_with_checkpoint(|_| {
            controller = budget.charged_memory_bytes();
        })
        .unwrap();
    assert_eq!(budget.charged_memory_bytes(), 0);
    let region = allocation::<ResidentRegion>();
    let bill =
        allocation::<RelationalRegionResidency>() + allocation::<ExecutionMemoryReservation>();
    let budget = SerialMemoryBudget::new(Some(controller + region + bill - 1));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let result = RelationalRegionResidency::resident_for_request(
        partition(),
        PartitionContentCommitment::default(),
        &ExecutionRequest::serial(&host),
    );
    let Err(RegionContentFailure::Execution(cause)) = result else {
        panic!("entered actual control admission must refuse");
    };
    match cause {
        crate::execution::RelationalExecutionDenialCause::PolicyMemoryExhausted {
            requested,
            admitted,
            ancestor: 0,
        } => {
            assert_eq!(requested, bill);
            assert_eq!(admitted, bill - 1);
        }
        other => panic!("exact control admission cause required: {other:?}"),
    }
    assert_eq!(budget.charged_memory_bytes(), 0);
    let cancel = CancellationSource::new();
    let budget = SerialMemoryBudget::new(Some(4 * 1024 * 1024));
    let host = SerialRequest::from_memory(budget.clone(), cancel.token(), None);
    cancel.cancel();
    assert!(matches!(
        RelationalRegionResidency::resident_for_request(
            partition(),
            PartitionContentCommitment::default(),
            &ExecutionRequest::serial(&host)
        ),
        Err(RegionContentFailure::Execution(_))
    ));
    assert_eq!(budget.charged_memory_bytes(), 0);
}
