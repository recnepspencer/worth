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

#[test]
fn retained_fact_comparison_is_dirty_bounded_as_one_native_population_grows() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(|graph| facts::seed_set(graph, "even", -0.0));
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut population = 0;
    let mut command = 90_000;
    let mut growth = Vec::new();
    for size in [1, 10, 100] {
        command += 1;
        // Ordinary authored entries have independent region partitions.
        edit(
            &request,
            &application,
            EntryEdit::create(
                &["even"],
                population as u64,
                population as u32,
                1.0_f64.to_bits(),
                1,
            )
            .batch(size - population),
            command,
        );
        population = size;
        demand(&request, &application);
        let mut equivalent = Vec::new();
        for value in [2.5_f64, 3.5] {
            command += 1;
            edit(
                &request,
                &application,
                EntryEdit::new("even", 0, EntryFact::Value, value.to_bits()),
                command,
            );
            fact_verification();
            let (_, runs) = demand(&request, &application);
            let examined = fact_verification();
            let [run] = runs.as_slice() else {
                panic!("{size}: one output decision publishes the edited sum: {runs:?}")
            };
            assert_eq!(run.runs, [Run::Incremental], "{size}: native reuse");
            assert_eq!(
                run.outcome.as_ref().map(|(bits, _)| *bits),
                Ok((size as f64 - 1.0 + value).to_bits()),
                "{size}: independently authored values survive reuse"
            );
            assert_eq!(
                examined.retained_bound_visits, 0,
                "{size}: the retained bound is cached"
            );
            assert!(
                examined.retained_computation_visits > 0 && examined.retained_native_visits > 0,
                "{size}: actual dirty facts are examined: {examined:?}"
            );
            assert_eq!(
                examined.carried_seal_visits, 0,
                "{size}: same pinned carriage is consumed at seal"
            );
            assert!(
                examined.native_payload_fact_moves > 0,
                "{size}: real changed-leaf postcommit facts reach the native carrier: {examined:?}"
            );
            assert!(
                examined.native_payload_chunks > 0,
                "{size}: the published path seals native storage: {examined:?}"
            );
            if size == 100 {
                assert!(
                    examined.native_payload_joins > 0,
                    "native publication consumes joined storage, not just a flat counterpart"
                );
            }
            assert_eq!(
                examined.sealed_computation_index_visits, 0,
                "{size}: native completion consumes checked original metadata: {examined:?}"
            );
            assert_eq!(
                examined.carried_key_expansion_visits, 0,
                "{size}: carried calls do not expand every prior key at completion: {examined:?}"
            );
            assert_eq!(examined.fallback_carried_metadata_visits, 0,
                "{size}: fixed native completion does not hide a fallback metadata walk: {examined:?}");
            assert!(
                examined.content_field_index_visits > 0,
                "{size}: changed-address validation remains measured: {examined:?}"
            );
            assert!(
                examined.content_leaf_derivations > 0 && examined.content_pair_derivations > 0,
                "{size}: real native publication consumes the new content rule: {examined:?}"
            );
            assert!(
                examined.native_patch_path_visits > 0 && examined.native_patch_fact_clones > 0,
                "{size}: actual ordinary publication consumes a funded native patch: {examined:?}"
            );
            let refreshed_keys =
                examined.handler_read_key_records + examined.fresh_read_key_records;
            assert!(
                examined.native_patch_fact_clones <= 32 * refreshed_keys,
                "{size}: copies follow changed leaf granules and actual fresh reads: {examined:?}"
            );
            assert!(
                examined.completion_call_roster_visits == 0,
                "{size}: closed native call ranges consume the checked roster: {examined:?}"
            );
            assert_eq!(examined.computation_dispatch_plan_visits, 1,
                "{size}: only the actually changed partition constructs a native input: {examined:?}");
            assert_eq!(
                examined.computation_result_receiver_visits, 1,
                "{size}: only that changed native result is looked up: {examined:?}"
            );
            assert_eq!(examined.computation_tree_plan_visits, 0,
                "{size}: an ordinary fixed-membership edit constructs no full tree plan: {examined:?}");
            assert_eq!(examined.computation_native_carried_slot_visits, 0,
                "{size}: fitting unchanged native ranges dispatch no individual slots: {examined:?}");
            assert!(
                examined.computation_prepared_partition_visits == 1,
                "{size}: only the changed gather enters actual preparation: {examined:?}"
            );
            assert!(
                examined.computation_retained_size_visits == 0,
                "{size}: immutable item and call bounds are carried: {examined:?}"
            );
            assert_eq!(
                examined.computation_call_build_visits, 0,
                "{size}: ordinary preparation builds no hidden full call index: {examined:?}"
            );
            assert!(examined.computation_call_node_copies > 0 && examined.computation_call_range_carries > 0,
                "{size}: actual changed-path call work and range charges remain visible: {examined:?}");
            assert!(
                examined.source_admission_visits > 0,
                "ordinary read admission remains real"
            );
            equivalent.push(examined);
        }
        // Compare equivalent actual runs; no fixture-fitted expected work count.
        let shape =
            |cost: &worth_query_host::facade::primary_graph::WorthQueryFactVerificationForTest| {
                (
                    cost.retained_bound_visits,
                    cost.retained_computation_visits,
                    cost.retained_native_visits,
                    cost.carried_seal_visits,
                )
            };
        assert_eq!(
            shape(&equivalent[0]),
            shape(&equivalent[1]),
            "{size}: equivalent edits examine the same retained population"
        );
        growth.push((size, equivalent));
        command += 1;
        edit(
            &request,
            &application,
            EntryEdit::new("even", 0, EntryFact::Value, 1.0_f64.to_bits()),
            command,
        );
        demand(&request, &application);
    }
    let first = growth[0].1[0];
    for (size, runs) in &growth {
        assert_eq!(
            runs[0].content_leaf_derivations, first.content_leaf_derivations,
            "{size}: logical leaves derive only at refreshed ordinals and the fresh source suffix"
        );
        assert_eq!(
            runs[0].retained_computation_visits, first.retained_computation_visits,
            "{size}: comparison depends on the edited facts, not retained population"
        );
        assert_eq!(runs[0].retained_native_visits, first.retained_native_visits);
        assert!(runs[0].rebase_preparation_visits <= first.rebase_preparation_visits * 4,
            "{size}: pre-effect rebase preparation follows refreshed facts/native matches, not every retained fact: {:?}", runs[0]);
        assert!(
            runs[0].rebase_resolution_visits <= first.rebase_resolution_visits * 4,
            "{size}: native rebase resolves only the complete selected frontier: {:?}",
            runs[0]
        );
        assert!(
            runs[0].native_payload_size_visits
                <= 32
                    * (runs[0].fresh_read_key_records
                        + runs[0].handler_read_key_records
                        + 2 * runs[0].rebase_resolution_visits)
                    + runs[0].completed_fact_moves,
            "{size}: postcommit sizing follows bounded changed leaf granules: {:?}",
            runs[0]
        );
        assert_eq!(
            runs[0].provider_application_fact_visits, first.provider_application_fact_visits,
            "{size}: fresh application binding follows changed observations and fresh source roles"
        );
        assert_eq!(
            runs[0].effect_index_fact_visits, first.effect_index_fact_visits,
            "{size}: effect routing validates changed addresses and borrows checked metadata"
        );
        assert_eq!(
            runs[0].content_field_index_visits, first.content_field_index_visits,
            "{size}: address validation does not expand to the retained population"
        );
        assert_eq!(
            runs[0].value_comparison_visits, first.value_comparison_visits,
            "{size}: genuine same-source closure leaves only fresh individual comparisons"
        );
        assert!(
            runs[0].posting_fact_visits > 0 && runs[0].posting_registration_visits > 0,
            "{size}: checked projection and exact binding work stays visible: {:?}",
            runs[0]
        );
        assert_eq!(
            runs[0].posting_fact_visits, first.posting_fact_visits,
            "{size}: posting projection follows changed native ordinals and output roles"
        );
        assert_eq!(runs[0].posting_registration_visits, first.posting_registration_visits,
            "{size}: exact successor installation and predecessor retirement do not visit unchanged postings");
        assert_eq!(runs[0].completed_fact_moves, first.completed_fact_moves,
            "{size}: current source suffix is moved once; carried decision facts stay in their backing");
        assert_eq!(
            runs[0].completion_fresh_key_visits, first.completion_fresh_key_visits,
            "{size}: current key-set validation follows newly executed reads"
        );
    }
    eprintln!(
        "native dirty comparison, result staging and remaining current-read/meta work at 1/10/100: {growth:?}"
    );
}

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
            [Run::Full(FullCause::FirstRun)],
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
        tree_work::assert_counted(first);
        let keys = tree_work::keys(size);
        tree_work::assert_edited(
            &next,
            tree_work::update_bound(&keys, tree_work::identity(u32::try_from(EDITED).unwrap())),
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

        // The first member of an absent region inserts a tree leaf; deleting
        // its final member removes that leaf. Both read membership again,
        // while deletion has no surviving region kernel to run.
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
        let inserted = tree_work::identity(JOINED);
        tree_work::assert_edited(&next, tree_work::insert_bound(&keys, inserted));
        let mut with_inserted = keys.clone();
        with_inserted.push(inserted);
        with_inserted.sort();
        edit(&request, &application, EntryEdit::delete(number), 3);
        let next = reused(&request, &application, size, "the deleted entry");
        tree_work::assert_edited(&next, tree_work::delete_bound(&with_inserted, inserted));
        assert_eq!(
            next.calls,
            OwnerCalls {
                plans: 1,
                keys: 0,
                gathers: 0,
                kernels: 0,
            },
            "size {size}: deleting the last member runs no kernel for the removed partition"
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

mod shared_fact;
