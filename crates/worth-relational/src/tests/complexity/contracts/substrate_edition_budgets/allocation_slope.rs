//! An allocation witness that no counter can forge.
//!
//! Counters are maintained by hand, so a counter alone cannot prove that a
//! bounded read stopped touching the source. These probes measure the process
//! allocator directly across two axes of source breadth that the answer does
//! not depend on: fanout past the budget, and partitions the anchor is not in.
//! Ordinary read allocation must be flat in both.
//!
//! `stats_alloc` instruments the whole process, so a region measured while a
//! sibling test allocates on another thread measures that sibling too. Each
//! slope claim therefore re-executes this binary filtered to one isolated
//! probe, which is the only test that then runs at all.

#[cfg(feature = "allocation-probes")]
use crate::tests::support::*;

#[cfg(feature = "allocation-probes")]
use stats_alloc::{Region, StatsAlloc, INSTRUMENTED_SYSTEM};

#[cfg(feature = "allocation-probes")]
#[global_allocator]
static TEST_ALLOCATOR: &StatsAlloc<std::alloc::System> = &INSTRUMENTED_SYSTEM;

#[cfg(feature = "allocation-probes")]
mod probe_shape {
    pub(super) const RELATION_KIND: crate::facade::identity::KindId =
        crate::facade::identity::KindId(2);
    pub(super) const REFUSAL_BUDGET: usize = 6;
    pub(super) const MEASURED_FANOUT: usize = 16;
    pub(super) const WIDE_FANOUT: usize = 512;
    pub(super) const UNRELATED_PARTITIONS: usize = 96;
    pub(super) const PROBE_GATE: &str = "WORTH_RELATIONAL_ADJACENCY_ALLOCATION_PROBE";
}

#[cfg(feature = "allocation-probes")]
use probe_shape::*;

#[test]
#[cfg(feature = "allocation-probes")]
fn bounded_adjacency_allocation_is_flat_in_fanout_beyond_the_budget() {
    run_isolated_probe("isolated_bounded_adjacency_fanout_allocation_slope_probe");
}

#[test]
#[cfg(feature = "allocation-probes")]
fn bounded_adjacency_allocation_is_flat_in_partitions_the_anchor_is_not_in() {
    run_isolated_probe("isolated_bounded_adjacency_partition_allocation_slope_probe");
}

#[test]
#[cfg(feature = "allocation-probes")]
fn isolated_bounded_adjacency_fanout_allocation_slope_probe() {
    if std::env::var_os(PROBE_GATE).is_none() {
        return;
    }
    let narrow = measured_bounded_read(MEASURED_FANOUT, 0);
    let wide = measured_bounded_read(WIDE_FANOUT, 0);

    // Same budget, same refusal, thirty-two times the fanout. Materializing the
    // kind slice before applying the bound would put that factor here.
    assert_eq!(narrow.allocations, wide.allocations);
    assert_eq!(narrow.reallocations, wide.reallocations);
    assert_eq!(narrow.bytes_allocated, wide.bytes_allocated);
}

#[test]
#[cfg(feature = "allocation-probes")]
fn isolated_bounded_adjacency_partition_allocation_slope_probe() {
    if std::env::var_os(PROBE_GATE).is_none() {
        return;
    }
    let narrow = measured_bounded_read(MEASURED_FANOUT, 0);
    let wide = measured_bounded_read(MEASURED_FANOUT, UNRELATED_PARTITIONS);

    // The answer never leaves one partition, so ninety-six unrelated ones must
    // cost nothing. Copying the partition map per record lookup would put a
    // per-record multiple of the partition count here.
    assert_eq!(narrow.allocations, wide.allocations);
    assert_eq!(narrow.reallocations, wide.reallocations);
    assert_eq!(narrow.bytes_allocated, wide.bytes_allocated);
}

/// Re-execute this test binary filtered to one probe, so the measured region
/// contains only that probe's own allocations.
#[cfg(feature = "allocation-probes")]
fn run_isolated_probe(probe_name: &str) {
    let output = crate::process_deadline::output(
        std::process::Command::new(
            std::env::current_exe().expect("the test binary knows its own path"),
        )
        .arg(probe_name)
        .arg("--test-threads=1")
        .env(PROBE_GATE, "1"),
    )
    .expect("the isolated probe process should start");
    let report = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "isolated allocation probe {probe_name} failed:\n{report}\n{}",
        String::from_utf8_lossy(&output.stderr),
    );
    // A filter that matches nothing also exits zero, which would turn
    // this slope claim into a claim about nothing.
    assert!(
        report.contains("1 passed"),
        "isolated allocation probe {probe_name} ran no test:\n{report}",
    );
}

/// Allocation performed by one budget-refused adjacency read.
///
/// Everything the answer depends on is held fixed: the same anchor degree
/// within the budget, the same work bound, the same refusal. Only the source
/// breadth around it moves.
#[cfg(feature = "allocation-probes")]
fn measured_bounded_read(fanout: usize, unrelated_partitions: usize) -> stats_alloc::Stats {
    let runtime = runtime_with_test_schema();
    let anchor = create_entity(&runtime, "anchor-0000");
    for edge in 0..fanout {
        let target = create_entity(&runtime, &format!("target-{edge:04}"));
        let _ = create_relation(&runtime, anchor, target, &format!("edge-{edge:04}"));
    }
    for partition in 0..unrelated_partitions {
        let _ = create_entity_in_partition(
            &runtime,
            &format!("unrelated-{partition:04}"),
            PartitionId(64 + partition as u32),
        );
    }
    let version_id = runtime.current_version_id();

    let region = Region::new(TEST_ALLOCATOR);
    let refusal = runtime
        .read_truth()
        .bounded_outgoing_relations_of_kind_at_version(
            anchor,
            RELATION_KIND,
            version_id,
            REFUSAL_BUDGET,
        )
        .expect_err("the budget is smaller than every measured fanout");
    let stats = region.change();
    assert_eq!(refusal.consumed_work_units(), REFUSAL_BUDGET);
    stats
}
