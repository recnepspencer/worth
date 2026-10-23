//! A pass is bounded by a fresh map, including at the serial memory edge.
use super::{
    entry_edits::{grouped, PendingEdit},
    persistent_allocation::prepare_empty_map,
    request_payment,
};
use crate::indexes::data::{
    DerivedIndexEntryMap, DerivedIndexMaintenanceBudget,
    DerivedIndexMaintenanceDenialKind as Denial, DerivedIndexRows,
};
use std::{collections::BTreeMap, mem::size_of, sync::Arc};
use worth_execution::ExecutionRequest;
const MEMORY: u64 = 64 * 1024 * 1024;
fn height(mut n: usize) -> usize {
    let mut h = 1;
    while n > 64 {
        n = n.div_ceil(32);
        h += 1;
    }
    h
}
fn full_bytes(n: usize) -> u64 {
    // im node: 64 key/inline-vector pairs, 65 child pointers, four offsets,
    // two Arc words. 31-key minimum occupancy, two roots, one 3h+2 transient path.
    let node = 64 * size_of::<(Arc<u64>, DerivedIndexRows<u64>)>() + 71 * size_of::<usize>();
    ((n.div_ceil(31) + 2 + 3 * height(n) + 2) * node
        + n * (2 * size_of::<u64>() + 4 * size_of::<usize>())) as u64
}
fn budget() -> DerivedIndexMaintenanceBudget {
    DerivedIndexMaintenanceBudget {
        maximum_work_units: Some(1_000_000_000),
        maximum_cold_record_slots: Some(1_000_000),
        maximum_derived_rows: Some(1_000_000),
    }
}
fn edits(n: usize, insert: bool) -> BTreeMap<u64, Vec<PendingEdit<u64>>> {
    (0..n as u64)
        .map(|k| (k, vec![PendingEdit { row: k, insert }]))
        .collect()
}
fn serial_host() -> worth_execution::SerialRequest {
    worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(Some(MEMORY)),
        worth_execution::CancellationToken::new(),
        None,
    )
}
#[test]
fn cold_reconstruction_meets_the_derived_serial_memory_edge() {
    // The serial singleton context's own bytes are Execution's private layout;
    // the pass's room is measured on the same request shape, never restated.
    let room =
        request_payment::pass_memory_room(&ExecutionRequest::serial(&serial_host()), MEMORY + 1);
    assert!(room < MEMORY);
    let mut low = 0;
    let mut high = (MEMORY / 48) as usize;
    while low < high {
        let mid = (low + high).div_ceil(2);
        if full_bytes(mid) <= room {
            low = mid
        } else {
            high = mid - 1
        }
    }
    let rows = low;
    assert!(full_bytes(rows) <= room && full_bytes(rows + 1) > room);
    eprintln!(
        "cold u64 map rows={rows}, room={room}, bytes={}, next={}",
        full_bytes(rows),
        full_bytes(rows + 1)
    );
    for n in [rows, rows + 1] {
        let host = serial_host();
        let execution = ExecutionRequest::serial(&host);
        let mut offered = edits(n, true);
        let result = request_payment::run(&execution, budget(), |work| {
            prepare_empty_map::<u64, u64>(work)?;
            let mut map = DerivedIndexEntryMap::default();
            grouped(
                &mut map,
                std::mem::take(&mut offered),
                Ord::cmp,
                |_| 1,
                true,
                work,
            )?;
            Ok(map)
        });
        if n == rows {
            assert_eq!(result.unwrap().0.value.len(), rows);
        } else {
            let denied = result.unwrap_err();
            let Denial::Execution(
                crate::execution::RelationalExecutionDenialCause::PolicyMemoryExhausted {
                    requested,
                    admitted,
                    ancestor: 0,
                },
            ) = denied.kind
            else {
                panic!("the first unaffordable reservation must refuse: {denied:?}");
            };
            // The real pass meets the same room the empty probe measured.
            assert_eq!(admitted, room);
            assert!(requested > admitted && requested <= full_bytes(rows + 1));
        }
    }
}
#[test]
fn edit_credit_is_monotone_and_bounded_by_a_fresh_map() {
    let baseline = DerivedIndexEntryMap::from(
        (0..2000_u64)
            .map(|k| (k, vec![k]))
            .collect::<BTreeMap<_, _>>(),
    );
    let mut previous = 0;
    for k in [1, 2, 64, 2000] {
        let host = crate::test_execution_policy::serial_request();
        let execution = ExecutionRequest::serial(&host);
        let mut offered = edits(k, false);
        for (key, rows) in &mut offered {
            rows.push(PendingEdit {
                row: *key + 2000,
                insert: true,
            });
        }
        let mut map = baseline.clone();
        let result = request_payment::run(&execution, budget(), |work| {
            grouped(
                &mut map,
                std::mem::take(&mut offered),
                Ord::cmp,
                |_| 1,
                false,
                work,
            )?;
            Ok(())
        });
        result.unwrap();
        let charged = request_payment::last_allocation_bytes();
        assert!(charged >= previous, "k={k}");
        assert!(charged <= full_bytes(2000), "k={k}");
        previous = charged;
    }
}

#[test]
fn row_copy_bound_prices_inline_leaf_and_branch_representations() {
    use super::persistent_allocation::row_copy_bounds;
    let inline = sized_chunks::InlineArray::<Arc<u64>, im::vector::Rrb<Arc<u64>>>::CAPACITY;
    assert_eq!(row_copy_bounds::<u64>(inline, true).unwrap(), (0, 0));
    let leaf = (64 * size_of::<Arc<u64>>()) + 4 * size_of::<usize>();
    assert_eq!(
        leaf,
        size_of::<sized_chunks::Chunk<Arc<u64>>>() + 2 * size_of::<usize>()
    );
    assert_eq!(row_copy_bounds::<u64>(8, false).unwrap(), (leaf, leaf));
    assert_eq!(row_copy_bounds::<u64>(64, false).unwrap(), (leaf, leaf));
    let h = height(2000);
    let branch = 332 * size_of::<usize>();
    let leaves = 2 * 2000usize.div_ceil(32) + 4;
    let branches = 2 * leaves.div_ceil(32) + 4 * h + 4;
    assert_eq!(
        row_copy_bounds::<u64>(2000, false).unwrap(),
        (
            (5 * h + 4) * branch + 4 * leaf,
            leaves * leaf + branches * branch
        )
    );
}
