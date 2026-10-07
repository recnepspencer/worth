//! The partitioned owner binding executes: every partition's kernel, the
//! canonical reduction and the completion, under a real request.

#[cfg(feature = "test-query-execution-observer")]
use worth_foundational::facade::ExecutionFallbackCause;
use worth_foundational::facade::PartitionIdentity;
use worth_query_decl::facade::application_operation::application_computation_partition_identity;
use worth_query_host::facade::application_contribution::{
    WorthQueryComputationPartitionStop, WorthQueryManagedComputationResourceDenial,
    WorthQueryMemoryLimitLevel, WorthQueryPartitionedComputationDenial,
};

use super::facts::{RegionEntry, RegionFault};
use super::owner::{
    with_region_totals, with_totals, RegionOutcome, RegionTotal, UnboundedBytesOwner,
};
use super::*;

const NEGATIVE_ZERO: u64 = (-0.0_f64).to_bits();

pub(super) fn entry(id: u64, region: u32, value: f64) -> RegionEntry {
    RegionEntry {
        id,
        region,
        value,
        work: 1,
        fault: None,
    }
}

fn total(outcome: RegionOutcome) -> RegionTotal {
    outcome.expect("the region totals complete")
}

/// The partition identity the declaration derives for a region's key.
fn partition_of(region: u32) -> PartitionIdentity {
    application_computation_partition_identity(&RegionKey(region), &mut |_| Ok::<(), ()>(()))
        .expect("a region key encodes")
        .partition()
}

/// The regions in the order their partitions reduce: partition identity order.
fn in_partition_order(regions: impl IntoIterator<Item = u32>) -> Vec<u32> {
    let mut regions = regions.into_iter().collect::<Vec<_>>();
    regions.sort_by_key(|region| partition_of(*region));
    regions
}

/// A left-to-right sum from addition's identity, as a kernel sums a region.
fn sum(values: impl IntoIterator<Item = f64>) -> f64 {
    values.into_iter().fold(-0.0, |sum, value| sum + value)
}

/// Region sums whose total depends on how the additions associate, with zeros
/// of both signs among the entries.
pub(super) fn association_sensitive_entries() -> Vec<RegionEntry> {
    vec![
        entry(1, 1, 1.0e16),
        entry(2, 1, 1.0),
        entry(3, 2, -1.0e16),
        entry(4, 3, 1.0),
        entry(5, 3, 1.0e-3),
        entry(6, 4, 3.0),
        entry(7, 4, -0.0),
        entry(8, 5, -0.0),
        entry(9, 5, 0.0),
        entry(10, 6, 2.5e15),
        entry(11, 6, 0.3),
        entry(12, 7, -2.5e15),
        entry(13, 8, 0.1),
        entry(14, 8, 0.2),
        // The first region's third value: its sum now depends on the order
        // its own entries are added in.
        entry(15, 1, 1.0),
    ]
}

#[test]
fn keyed_floating_point_sum_keeps_its_bits_and_its_charged_work_on_every_run() {
    let entries = association_sensitive_entries();
    // The fixture is only a proof if association matters to it: summing the
    // regions' sums ascending and descending gives different bits.
    let region_sums = (1..=8)
        .map(|region| {
            sum(entries
                .iter()
                .filter(|entry| entry.region == region)
                .map(|entry| entry.value))
        })
        .collect::<Vec<_>>();
    assert_ne!(
        sum(region_sums.iter().copied()).to_bits(),
        sum(region_sums.iter().rev().copied()).to_bits()
    );

    // Exactly representable values total exactly under any association, so
    // every entry is counted once.
    let exact = (1..=40_u32)
        .map(|id| entry(u64::from(id), id % 7, f64::from(id)))
        .collect::<Vec<_>>();

    with_region_totals(&[("sensitive", &entries), ("exact", &exact)], |demand| {
        let first = total(demand("sensitive"));
        assert!(f64::from_bits(first.bits).is_finite());
        // No lease reaches a managed computation, so the serial backend ran.
        #[cfg(feature = "test-query-execution-observer")]
        {
            assert_eq!(
                first.report.fallback(),
                Some(ExecutionFallbackCause::NoLease)
            );
            assert!(first.charged_work > first.report.charged_work());
        }
        for _ in 0..3 {
            assert_eq!(total(demand("sensitive")), first);
        }
        assert_eq!(total(demand("exact")).bits, 820.0_f64.to_bits());
    });
}

#[test]
fn empty_input_totals_to_the_reducer_identity_and_one_region_to_its_own_sum() {
    // One region's values, met out of identity order. The kernel sums them in
    // entry identity order, which here loses the small values.
    let one_region = [
        entry(4, 9, -1.0e16),
        entry(1, 9, 1.0e16),
        entry(3, 9, 1.0),
        entry(2, 9, 1.0),
    ];
    with_region_totals(&[("empty", &[]), ("one-region", &one_region)], |demand| {
        let empty = total(demand("empty"));
        assert_eq!(empty.bits, NEGATIVE_ZERO);
        assert_eq!(total(demand("empty")), empty);

        let single = total(demand("one-region"));
        assert_eq!(single.bits, sum([1.0e16, 1.0, 1.0, -1.0e16]).to_bits());
        assert_ne!(
            single.bits,
            sum(one_region.iter().map(|entry| entry.value)).to_bits()
        );
        assert_eq!(total(demand("one-region")), single);
    });
}

#[test]
fn reordering_the_input_changes_neither_the_total_nor_the_charged_work() {
    let entries = association_sensitive_entries();
    let mut reversed = entries.clone();
    reversed.reverse();
    let mut interleaved = entries.clone();
    interleaved.rotate_left(5);
    interleaved.swap(0, 9);
    // Summed as met, the two orders disagree, for the whole input and inside
    // the first region: the invariance below is the partitioned binding's,
    // not the data's.
    let as_met = |entries: &[RegionEntry], region: Option<u32>| {
        sum(entries
            .iter()
            .filter(|entry| region.is_none_or(|region| entry.region == region))
            .map(|entry| entry.value))
        .to_bits()
    };
    assert_ne!(as_met(&entries, None), as_met(&reversed, None));
    assert_ne!(as_met(&entries, Some(1)), as_met(&reversed, Some(1)));

    let sets: facts::Sets<'_> = &[
        ("planned", &entries),
        ("reversed", &reversed),
        ("interleaved", &interleaved),
    ];
    with_region_totals(sets, |demand| {
        let planned = total(demand("planned"));
        assert_eq!(total(demand("reversed")), planned);
        assert_eq!(total(demand("interleaved")), planned);
    });
}

#[test]
fn partitions_whose_declared_bytes_pass_the_request_memory_are_denied_and_the_next_demand_is_answered(
) {
    // The computation declares the largest byte count there is, and a gather
    // holds its declared bytes before it runs. No request's memory admits
    // that: the first gather is refused by the request's policy, with one
    // region or two, and the demand after a refusal is answered the same way.
    let two_regions = [entry(1, 1, 2.0), entry(2, 2, 3.0)];
    let one_region = [entry(1, 1, 2.0), entry(2, 1, 3.0)];
    let sets: facts::Sets<'_> = &[("two-regions", &two_regions), ("one-region", &one_region)];
    with_totals::<UnboundedBytesOwner>(sets, |demand| {
        for set in ["two-regions", "one-region", "two-regions"] {
            let outcome = demand(set);
            assert!(
                matches!(
                    outcome,
                    Err(WorthQueryPartitionedComputationDenial::Resource(
                        WorthQueryManagedComputationResourceDenial::MemoryLimit {
                            requested: u64::MAX,
                            level: WorthQueryMemoryLimitLevel::Policy,
                            ..
                        }
                    ))
                ),
                "{set}: {outcome:?}"
            );
        }
    });
}

/// Six regions met in descending partition order, of which the second,
/// fourth and fifth partitions fail, and the least failure: the last met.
pub(super) fn several_refusals() -> (Vec<RegionEntry>, RegionOutcome) {
    let regions = in_partition_order(1..=6);
    let entries = regions
        .iter()
        .enumerate()
        .rev()
        .map(|(place, region)| RegionEntry {
            fault: match place {
                1 | 3 => Some(RegionFault::Refuse),
                4 => Some(RegionFault::Panic),
                _ => None,
            },
            ..entry(u64::from(*region), *region, 1.0)
        })
        .collect();
    let least = Err(WorthQueryPartitionedComputationDenial::Partition {
        partition: partition_of(regions[1]),
        cause: WorthQueryComputationPartitionStop::Owner(regions[1]),
    });
    (entries, least)
}

#[test]
fn refusals_in_several_regions_report_the_least_partition_identity() {
    let (entries, least) = several_refusals();
    let mut ascending = entries.clone();
    ascending.reverse();

    let sets: facts::Sets<'_> = &[("descending", &entries), ("ascending", &ascending)];
    with_region_totals(sets, |demand| {
        assert_eq!(demand("descending"), least);
        assert_eq!(demand("ascending"), least);
    });
}

#[test]
fn kernel_panic_is_a_typed_stop_and_the_next_demand_runs() {
    let clean = [entry(1, 1, 2.0), entry(2, 2, 3.0)];
    let mut panicking = clean;
    panicking[1].fault = Some(RegionFault::Panic);
    with_region_totals(&[("panicking", &panicking), ("clean", &clean)], |demand| {
        assert_eq!(
            demand("panicking"),
            Err(WorthQueryPartitionedComputationDenial::Partition {
                partition: partition_of(2),
                cause: WorthQueryComputationPartitionStop::Panicked,
            })
        );
        assert_eq!(total(demand("clean")).bits, 5.0_f64.to_bits());
    });
}

/// Three regions of 1,500 units of work each, and where the computation's
/// declared 4,096 stop them: two regions fit and the third does not,
/// whichever region the input meets first.
pub(super) fn heavy_regions() -> (Vec<RegionEntry>, RegionOutcome) {
    let regions = in_partition_order(1..=3);
    let heavy = regions
        .iter()
        .map(|region| RegionEntry {
            work: 1_500,
            ..entry(u64::from(*region), *region, 1.0)
        })
        .collect();
    let exhausted = Err(WorthQueryPartitionedComputationDenial::Partition {
        partition: partition_of(regions[2]),
        cause: WorthQueryComputationPartitionStop::Resource(
            WorthQueryManagedComputationResourceDenial::WorkExhausted,
        ),
    });
    (heavy, exhausted)
}

#[test]
fn work_ceiling_stops_at_the_same_partition_with_the_same_outcome_on_every_run() {
    let (heavy, exhausted) = heavy_regions();
    let mut reversed = heavy.clone();
    reversed.reverse();

    // The same regions inside the ceiling complete.
    let light = heavy
        .iter()
        .map(|entry| RegionEntry { work: 1, ..*entry })
        .collect::<Vec<_>>();
    let sets: facts::Sets<'_> = &[
        ("heavy", &heavy),
        ("reversed", &reversed),
        ("light", &light),
    ];
    with_region_totals(sets, |demand| {
        assert_eq!(demand("heavy"), exhausted);
        assert_eq!(demand("reversed"), exhausted);
        assert_eq!(demand("heavy"), exhausted);
        assert_eq!(total(demand("light")).bits, 3.0_f64.to_bits());
    });
}
