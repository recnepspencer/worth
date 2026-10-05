//! A second owner whose key, results and reducer can each be made to
//! misbehave, and what Query answers each time.

use serde::ser::Error as _;
use serde::{Serialize, Serializer};
use worth_foundational::facade::PartitionIdentity;
use worth_query_decl::facade::application_operation::application_computation_partition_identity;
use worth_query_host::facade::application_contribution::{
    PartitionItemId, WorthQueryComputationPartitionPlan, WorthQueryComputationPartitionStop,
    WorthQueryComputationPartitionView, WorthQueryDeterministicReducer,
    WorthQueryManagedComputationCheckpoint, WorthQueryManagedComputationDenial,
    WorthQueryManagedComputationResourceDenial, WorthQueryPartitionedComputationDenial,
    WorthQueryPartitionedComputationOwner,
};

use super::owner::{
    enter_kernel, kernels_entered, with_totals, Entries, Installed, RegionEntry, RegionFault,
    RegionTotalsHandler, Setup,
};
use super::*;

/// What the probing owner does with an entry.
#[derive(Clone, Copy, Debug)]
pub(super) enum ProbeFault {
    /// The entry's key refuses to encode.
    NoKey,
    /// The region's result holds this many more values, each a zero.
    Hold(usize),
    /// The region's result carries a value the reducer panics on.
    PoisonReducer,
}

const POISON: u64 = u64::MAX;

/// A region key that remembers the entry it was made from. Only the region is
/// encoded, so the keys of one region are the same key and differ in memory.
#[derive(Clone, Copy)]
pub(super) struct ProbeKey {
    region: u32,
    entry: u64,
    encodable: bool,
}

impl Serialize for ProbeKey {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if !self.encodable {
            return Err(S::Error::custom("the entry has no key"));
        }
        serializer.serialize_u32(self.region)
    }
}

impl ApplicationComputationPartition for ProbeKey {
    const IDENTITY: &'static str = "checkpoint-probe-region-key";
}

/// Each region's result is the entry its key remembers. The reducer gathers
/// the results and the completion sums them.
struct ProbingOwner;
impl WorthQueryPartitionedComputationOwner<CheckpointSchema, PlanarFinalOutputFeature, ProbedTotals>
    for ProbingOwner
{
    type PartitionResult = Vec<u64>;
    type Output = f64;
    type Stopped = u32;

    fn partitions(&self, entries: &Entries) -> WorthQueryComputationPartitionPlan<ProbeKey> {
        WorthQueryComputationPartitionPlan::keyed(
            entries,
            |entry| PartitionItemId(entry.id),
            |entry| ProbeKey {
                region: entry.region,
                entry: entry.id,
                encodable: !matches!(entry.fault, Some(RegionFault::Probe(ProbeFault::NoKey))),
            },
        )
    }

    fn compute_partition(
        &self,
        partition: WorthQueryComputationPartitionView<'_, ProbeKey, Entries>,
        checkpoint: &mut WorthQueryManagedComputationCheckpoint<'_>,
    ) -> Result<Vec<u64>, WorthQueryManagedComputationDenial<u32>> {
        enter_kernel();
        let mut seen = vec![partition.key().entry];
        for item in partition.items() {
            checkpoint.advance(1)?;
            match partition.input()[item.position()].fault {
                Some(RegionFault::Probe(ProbeFault::Hold(values))) => {
                    seen.resize(seen.len() + values, 0);
                }
                Some(RegionFault::Probe(ProbeFault::PoisonReducer)) => seen.push(POISON),
                _ => {}
            }
        }
        Ok(seen)
    }

    fn reducer(&self) -> WorthQueryDeterministicReducer<Vec<u64>> {
        WorthQueryDeterministicReducer::canonical(Vec::new, |left, right| {
            assert!(
                !left.iter().chain(right).any(|entry| *entry == POISON),
                "the reducer panics"
            );
            left.iter().chain(right).copied().collect()
        })
    }

    fn complete(&self, reduced: Vec<u64>) -> Result<f64, u32> {
        Ok(reduced.iter().map(|entry| *entry as f64).sum())
    }
}

struct Probed;
impl RegionTotalsBinding for Probed {
    type Computation = ProbedTotals;

    fn install(setup: &mut Setup<'_>) -> Installed {
        RegionTotalsHandler::running::<ProbedTotals, _>(setup, ProbingOwner)
    }
}

fn entry(id: u64, region: u32) -> RegionEntry {
    RegionEntry {
        id,
        region,
        value: 0.0,
        work: 1,
        fault: None,
    }
}

fn faulted(id: u64, region: u32, fault: ProbeFault) -> RegionEntry {
    RegionEntry {
        fault: Some(RegionFault::Probe(fault)),
        ..entry(id, region)
    }
}

fn partition_of(region: u32) -> PartitionIdentity {
    let key = ProbeKey {
        region,
        entry: 0,
        encodable: true,
    };
    application_computation_partition_identity(&key, &mut |_| Ok::<(), ()>(()))
        .expect("a region key encodes")
        .partition()
}

const BYTES_EXHAUSTED: WorthQueryManagedComputationResourceDenial =
    WorthQueryManagedComputationResourceDenial::RetainedBytesExhausted;

#[test]
fn partition_key_is_its_least_items_key_whatever_order_the_input_holds() {
    // Two regions. The least entries are 3 and 4, and neither is met first in
    // either order.
    let entries = [
        entry(5, 1),
        entry(3, 1),
        entry(9, 2),
        entry(4, 2),
        entry(7, 2),
    ];
    let mut reversed = entries;
    reversed.reverse();
    with_totals::<Probed>(|demand| {
        let planned = demand(&entries).expect("the probed totals complete");
        assert_eq!(planned.bits, 7.0_f64.to_bits());
        assert_eq!(demand(&reversed), Ok(planned));
    });
}

#[test]
fn result_larger_than_the_declared_bytes_is_a_typed_denial() {
    // The computation declares 8,192 bytes. A result fits them in memory and
    // once more as its canonical bits, which take 24 bytes a value.
    with_totals::<Probed>(|demand| {
        // One region's result is too large on its own.
        let oversized = [
            entry(1, 1),
            faulted(2, 2, ProbeFault::Hold(2_000)),
            entry(3, 3),
        ];
        assert_eq!(
            demand(&oversized).map(|total| total.bits),
            Err(WorthQueryPartitionedComputationDenial::Partition {
                partition: partition_of(2),
                cause: WorthQueryComputationPartitionStop::Resource(BYTES_EXHAUSTED),
            })
        );
        // A result fits alone, and what the reducer makes of two does not.
        let alone = [faulted(1, 1, ProbeFault::Hold(250))];
        assert_eq!(
            demand(&alone).map(|total| total.bits),
            Ok(1.0_f64.to_bits())
        );
        let together = [
            faulted(1, 1, ProbeFault::Hold(250)),
            faulted(2, 2, ProbeFault::Hold(250)),
        ];
        assert_eq!(
            demand(&together).map(|total| total.bits),
            Err(WorthQueryPartitionedComputationDenial::Resource(
                BYTES_EXHAUSTED
            ))
        );
        let fitting = [
            faulted(1, 1, ProbeFault::Hold(100)),
            faulted(2, 2, ProbeFault::Hold(100)),
        ];
        assert_eq!(
            demand(&fitting).map(|total| total.bits),
            Ok(3.0_f64.to_bits())
        );
    });
}

#[test]
fn reducer_panic_is_a_typed_failure_and_the_next_demand_runs() {
    with_totals::<Probed>(|demand| {
        let poisoned = [
            entry(1, 1),
            faulted(2, 2, ProbeFault::PoisonReducer),
            entry(3, 3),
        ];
        assert_eq!(
            demand(&poisoned).map(|total| total.bits),
            Err(WorthQueryPartitionedComputationDenial::ReducerPanicked)
        );
        assert_eq!(
            kernels_entered(),
            3,
            "every partition ran before the reducer"
        );
        let clean = [entry(1, 1), entry(2, 2), entry(3, 3)];
        assert_eq!(
            demand(&clean).map(|total| total.bits),
            Ok(6.0_f64.to_bits())
        );
    });
}

#[test]
fn key_that_cannot_be_encoded_is_denied_naming_its_item_and_nothing_runs() {
    // Two entries have no key. The denial names the lesser, whichever the
    // input meets first.
    let entries = [
        entry(1, 1),
        faulted(8, 2, ProbeFault::NoKey),
        entry(2, 3),
        faulted(6, 3, ProbeFault::NoKey),
        entry(9, 1),
    ];
    let mut reversed = entries;
    reversed.reverse();
    with_totals::<Probed>(|demand| {
        for input in [&entries, &reversed] {
            match demand(input) {
                Err(WorthQueryPartitionedComputationDenial::KeyNotEncodable { item, .. }) => {
                    assert_eq!(item, PartitionItemId(6));
                }
                other => panic!("the key has no encoding: {other:?}"),
            }
            assert_eq!(kernels_entered(), 0, "no partition runs");
        }
        let encodable = [entry(1, 1), entry(2, 3), entry(9, 1)];
        assert_eq!(
            demand(&encodable).map(|total| total.bits),
            Ok(3.0_f64.to_bits())
        );
        assert_eq!(kernels_entered(), 2);
    });
}
