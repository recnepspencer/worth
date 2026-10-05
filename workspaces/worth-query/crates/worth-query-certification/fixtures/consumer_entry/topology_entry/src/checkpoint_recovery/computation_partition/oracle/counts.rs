//! What a one-entry edit costs the producer's next run, at a small set and at
//! the largest set its decision is declared for: one contact, one gather and
//! one kernel, and no membership or item key read again.

use worth_query_host::facade::application_contribution::{
    WorthQueryPartitionedComputationFullCause as FullCause,
    WorthQueryPartitionedComputationRun as Run,
};

use super::super::entry_edit::EntryFact;
use super::*;

/// A small set, and the largest one.
const SIZES: [usize; 2] = [100, LARGEST_SET];
/// The entry the edit changes.
const EDITED: usize = 7;

#[test]
fn one_entry_edit_gathers_and_computes_one_partition_at_every_size() {
    let _guard = checkpoint_recovery_test_guard();
    for size in SIZES {
        // Every entry is its own region, so every region is one partition.
        let application = install(|graph| {
            facts::seed_set(graph, "even", -0.0);
            for place in 0..size {
                let entry = RegionEntry {
                    id: u64::try_from(place).unwrap(),
                    region: u32::try_from(place).unwrap(),
                    value: 1.0,
                    work: 1,
                    fault: None,
                };
                seed_entry(graph, &["even"], place, entry);
            }
        });
        let (scope, principal) = authenticate(&application);
        let request = application.request(&principal, &scope);

        let (contacts, runs) = demand(&request, &application);
        assert_eq!(contacts, 1, "size {size}: the output's first run");
        let [first] = runs.as_slice() else {
            panic!("size {size}: one decision, observed {runs:?}")
        };
        assert_eq!(first.runs, [Run::Full(FullCause::NoPriorRecord)]);
        assert_eq!(
            first.calls,
            OwnerCalls {
                plans: 1,
                keys: size,
                gathers: size,
                kernels: size,
            },
            "size {size}: a full run calls the owner for every entry"
        );

        let edited = EntryEdit::new(
            "even",
            u64::try_from(EDITED).unwrap(),
            EntryFact::Value,
            2.5_f64.to_bits(),
        );
        edit(&request, &application, edited, 1);
        let (contacts, runs) = demand(&request, &application);
        assert_eq!(
            contacts, 1,
            "size {size}: the edit contacts the producer once"
        );
        let [next] = runs.as_slice() else {
            panic!("size {size}: one decision, observed {runs:?}")
        };
        assert_eq!(next.runs, [Run::Incremental], "size {size}");
        assert_eq!(
            next.calls,
            OwnerCalls {
                plans: 0,
                keys: 0,
                gathers: 1,
                kernels: 1,
            },
            "size {size}: only the edited entry's partition is gathered and computed"
        );
        let total = f64::from(u32::try_from(size).unwrap()) + 1.5;
        assert_eq!(
            next.outcome.as_ref().map(|(bits, _)| *bits),
            Ok(total.to_bits()),
            "size {size}: the total holds the edit"
        );
        assert_eq!(
            next.outcome.as_ref().map(|(_, charged)| *charged),
            first.outcome.as_ref().map(|(_, charged)| *charged),
            "size {size}: a reused run is charged what the full run was"
        );
    }
}
