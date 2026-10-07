//! What a one-entry edit costs the producer's next run, at a small set and at
//! the largest set its decision is declared for: one contact, one gather and
//! one kernel, and no membership or item key read again. A new entry and a
//! deleted one read the membership again, key only the new entry, and gather
//! and compute only the partition they touched.

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
/// The region a new entry joins, beside the one entry already in it.
const JOINED: u32 = 3;

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
        assert_eq!(
            first.runs,
            [Run::Full(FullCause::NoPriorRecord)],
            "size {size}: {:?}",
            first.outcome
        );
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
        let next = reused(&request, &application, size, "the value edit");
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

        // A new entry joins a region that holds one entry already, and is
        // then deleted: each reads the membership again, and only the joined
        // region's partition is gathered and computed.
        let number = u64::try_from(size).unwrap();
        let created = EntryEdit::create(&["even"], number, JOINED, 4.0_f64.to_bits(), 1);
        edit(&request, &application, created, 2);
        let next = reused(&request, &application, size, "the new entry");
        assert_eq!(
            next.calls,
            OwnerCalls {
                plans: 1,
                keys: 1,
                gathers: 1,
                kernels: 1,
            },
            "size {size}: only the new entry is keyed, and only its partition is gathered"
        );
        assert_eq!(
            next.outcome.as_ref().map(|(bits, _)| *bits),
            Ok((total + 4.0).to_bits()),
            "size {size}: the total holds the new entry"
        );
        edit(&request, &application, EntryEdit::delete(number), 3);
        let next = reused(&request, &application, size, "the deleted entry");
        assert_eq!(
            next.calls,
            OwnerCalls {
                plans: 1,
                keys: 0,
                gathers: 1,
                kernels: 1,
            },
            "size {size}: nothing is keyed, and only the partition the entry left is gathered"
        );
        assert_eq!(
            next.outcome.as_ref().map(|(bits, _)| *bits),
            Ok(total.to_bits()),
            "size {size}: the total no longer holds the deleted entry"
        );
    }
}

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
