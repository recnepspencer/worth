//! What a one-entry edit costs the producer's next run, at a small set and at
//! the largest set its decision is declared for: one contact, one gather and
//! one kernel, and no membership or item key read again. A new entry and a
//! deleted one read the membership again, key only the new entry, and gather
//! and compute only the partition they touched.

use worth_query_host::facade::application_contribution::{
    WorthQueryPartitionedComputationFullCause as FullCause,
    WorthQueryPartitionedComputationRunForTest as Run,
};
use worth_query_host::facade::primary_graph::fact_verification_on_this_thread_for_test as fact_verification;

use super::super::entry_edit::EntryFact;
use super::*;

/// A small set, and the largest one.
const SIZES: [usize; 2] = [100, LARGEST_SET];
/// The entry the edit changes.
const EDITED: usize = 7;
/// An absent region: creating its first entry inserts an actual tree leaf.
const JOINED: u32 = 1_000_003;

/// The one run of the demand after `what`, which reuses partitions.
fn reused(
    request: &Request<'_, '_, '_>,
    application: &Application,
    size: usize,
    what: &str,
) -> OracleRun {
    let (contacts, mut runs) = demand(request, application);
    assert_eq!(
        contacts, 1,
        "size {size}: {what} contacts the producer once"
    );
    assert_eq!(
        runs.len(),
        1,
        "size {size}: one decision, observed {runs:?}"
    );
    let next = runs.pop().unwrap();
    assert_eq!(next.runs, [Run::Incremental], "size {size}: {what}");
    next
}

mod shared_fact;

#[cfg(test)]
#[path = "counts/serial.rs"]
mod serial;

/// Native history authoring is funded independently of the logical partition work assertions.
fn install_counted_population(seed: impl FnOnce(&mut Graph)) -> Application {
    let policy = worth_foundational::ExecutionRequestPolicy::new(
        worth_foundational::ExecutionPosture::Serial,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        worth_foundational::ExecutionBudget::new(
            std::num::NonZeroUsize::MIN,
            None,
            Some(8_000_000 + LARGEST_SET as u64 * 64_000),
        ),
    );
    super::installation::install_variant_with_observations::<false, TOTALS_WORK, 1, 0>(
        None,
        Default::default(),
        0,
        policy,
        seed,
    )
}
