//! The seeded sequence of edits on every placement of the worker axis: each
//! demand contacts the producer as often, and each of its runs reuses the
//! same way and keeps the serial run's bits, its charged work and its least
//! failing partition. On two workers or more the sequence holds two kernels
//! at once. A run that recomputes one region has one kernel, so the proof is
//! the sequence's, not each demand's.

use super::super::worker_axis::{axis, placed, workers_of, Overlapping};
use super::differential::sequence;
use super::*;

/// A run's outcome and how it ran.
type Ran = (
    Result<(u64, u64), WorthQueryPartitionedComputationDenial<u32>>,
    Vec<WorthQueryPartitionedComputationRun>,
);

/// Each demand of the sequence: its producer contacts and its runs.
fn demands() -> Vec<(usize, Vec<Ran>)> {
    let mut demands = Vec::new();
    sequence(|_, demanded| {
        let runs = demanded.runs.into_iter().map(|run| (run.outcome, run.runs));
        demands.push((demanded.contacts, runs.collect()));
    });
    demands
}

#[test]
fn every_demand_of_the_sequence_runs_the_same_at_every_worker_count() {
    let _guard = checkpoint_recovery_test_guard();
    let mut placements = axis().into_iter();
    let serial = placed(placements.next().expect("the axis is not empty"), demands);
    assert!(
        serial
            .iter()
            .flat_map(|(_, runs)| runs)
            .any(|(outcome, _)| outcome.is_err()),
        "the sequence meets a failing partition"
    );
    for placement in placements {
        let demands = placed(placement, || {
            let _overlapping = (workers_of(placement) >= 2).then(Overlapping::arm);
            demands()
        });
        assert_eq!(demands.len(), serial.len(), "{placement:?}");
        for (at, (demand, serial)) in demands.iter().zip(&serial).enumerate() {
            assert_eq!(demand, serial, "{placement:?}, demand {at}");
        }
    }
}
