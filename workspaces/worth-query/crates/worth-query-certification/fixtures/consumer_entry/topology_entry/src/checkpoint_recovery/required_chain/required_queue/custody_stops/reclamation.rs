//! Reclaimed rows decide again; complete ledgers balance after queue drainage.
use super::*;
pub(super) fn overwrite_middle_output(
    application: &application_installation::WorthQueryProgramApplicationRuntime<
        CheckpointSchema,
        program::ChainProgram,
    >,
    request: &worth_query_host::facade::application_entry::WorthQueryApplicationRequest<
        '_,
        '_,
        '_,
        CheckpointSchema,
    >,
    value: u64,
    idempotency: u64,
) {
    use worth_query_consumer_values::{PlanarDerivedOutput, PlanarOperation};
    let selected = request
        .query(PlanarRead {
            body_key: "anchor-b".to_owned(),
        })
        .execute()
        .unwrap();
    let outcome = request
        .mutate(PlanarEdit(PlanarMutation {
            scope_key: "anchor-b".to_owned(),
            operation: PlanarOperation::PublishDerivedOutput(PlanarDerivedOutput {
                body_key: "anchor-b".to_owned(),
                value: length(value),
            }),
        }))
        .expect_source(selected.observed_sources()[0].clone())
        .idempotency(&idempotency)
        .execute_in_program::<program::ChainProgram>(
            application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        );
    assert!(
        matches!(
            &outcome,
            Ok(worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome::Committed { .. })
        ),
        "the middle output is overwritten: {outcome:?}"
    );
}

/// A chain node republishes the Length its own source reads, so its output
/// does not vary with the root. Where `overwritten`, another writer changes
/// the middle output every cycle: the last consumer's decision then reads a
/// value no earlier cycle published.
#[test]
fn a_dependent_whose_row_was_reclaimed_decides_again_over_a_refreshed_upstream() {
    let _guard = checkpoint_recovery_test_guard();
    let model = super::custody_model::ChainCustody::new();
    for (overwritten, reclamation_rows) in [
        (false, model.reclaim_rows()),
        (true, model.middle_write_reclaim_rows()),
    ] {
        assert!(
            !reclamation_rows.is_empty(),
            "the reclamation schedule runs cases"
        );
        for rows in reclamation_rows {
            run_reclamation(rows, overwritten, true);
        }
    }
}

#[test]
fn queue_drained_reclamation_balances_the_complete_retained_tuple() {
    let _guard = checkpoint_recovery_test_guard();
    let model = super::custody_model::ChainCustody::new();
    let rows = model.closed_chain().div_ceil(model.layout.ready);
    for overwritten in [false, true] {
        run_reclamation(rows, overwritten, true);
    }
}

#[test]
fn the_model_upper_edge_keeps_the_cached_consumer_without_a_handler_contact() {
    let _guard = checkpoint_recovery_test_guard();
    let model = super::custody_model::ChainCustody::new();
    let rows = model.reclaim_rows().end() + 1;
    assert!(rows * model.layout.ready >= model.root_refresh_before_reclaim());
    assert!((rows - 1) * model.layout.ready < model.root_refresh_before_reclaim());
    run_reclamation(rows, false, false);
    run_reclamation(rows - 1, false, true);
    // A World writer changes B's consumed value: held C still contacts once.
    // Its first ordinary refresh has no typed required-successor slot. Later
    // cycles include the written output's own history, so this edge is primed.
    let written_rows = model.middle_write_reclaim_rows().end() + 1;
    assert!(written_rows * model.layout.ready >= model.middle_write_refresh_before_reclaim());
    run_reclamation_cycles(written_rows, true, false, 1);
}

#[test]
fn below_the_model_range_the_chain_is_refused_required_capacity() {
    let _guard = checkpoint_recovery_test_guard();
    let model = super::custody_model::ChainCustody::new();
    let rows = model.reclaim_rows().start() - 1;
    assert!(rows * model.layout.ready < model.closed_chain());
    let (application, _invalidation) =
        limited_application(rows * model.layout.ready, 128 * 1_024 * 1_024, 8);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut a = start_root!(application, request);
    settled_in_one_advance!(a, request, "the lower-edge root");
    let mut b = start_consumer!(application, request, "anchor-b");
    settled_in_one_advance!(b, request, "the lower-edge middle consumer");
    let mut c = start_consumer!(application, request, "anchor-c");
    let stopped = c.advance(&request);
    assert!(
        matches!(&stopped,
        Err(WorthQueryApplicationOutputDemandDenial::Demand(denial))
            if denial.kind() == WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded),
        "the chain cannot fit below its model's closed custody: {:?}",
        stopped.as_ref().err()
    );
}

fn run_reclamation(rows: usize, overwritten: bool, reclaims: bool) {
    run_reclamation_cycles(rows, overwritten, reclaims, 2 * 8 + 4);
}

fn run_reclamation_cycles(rows: usize, overwritten: bool, reclaims: bool, cycles: u64) {
    let row = worth_query_host::facade::primary_graph::required_ready_custody_bytes_for_test();
    let (application, invalidation) = limited_application(rows * row, 128 * 1_024 * 1_024, 8);
    application.consumed_output_custody_for_test();
    let branch_bytes = application
        .on_branch(application.current_world())
        .select()
        .unwrap()
        .product()
        .relational_basis_descriptor()
        .branch_id()
        .0
        .len();
    let native_layout = primary_graph::WorthQueryPrimaryGraphApplicationRuntime::<CheckpointSchema>::native_required_hint_layout_for_test(branch_bytes);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (a, b, c) = chain!(application, request);
    drop((a, b, c));
    assert_eq!(
        application.required_custody_bytes_for_test(),
        super::custody_model::ChainCustody::new().closed_chain(),
        "the closed chain's named owners"
    );
    let mut custody = None;
    let mut reclaimed_ledger = None;
    let mut settled_ledger = None;
    // Each retained owner has eight version positions. Two windows of
    // refreshes retire the differently shaped priming positions; the four
    // measured cycles then contain only this constructed repeated schedule.
    let steady_start = 2 * 8_u64;
    for cycle in 0..cycles {
        let boundary_baseline = invalidation.native_retained_allocations_for_test();
        let at = format!("{rows} rows, cycle {cycle}, overwritten {overwritten}");
        let y = 2 + (cycle % 2) * 3;
        let idempotency =
            0x9176_4200_u64 + u64::from(overwritten) * 0x400 + rows as u64 * 16 + cycle;
        change_root_input!(request, application, y, idempotency);
        let mut a = start_root!(application, request);
        settled_within_eight_advances!(a, request, format!("{at}: the root"));
        drop(a);
        let middle = if overwritten {
            overwrite_middle_output(&application, &request, 40 + cycle, idempotency + 8);
            40 + cycle
        } else {
            16
        };
        take_decisions("anchor-b");
        let mut b = start_consumer!(application, request, "anchor-b");
        settled_within_eight_advances!(b, request, format!("{at}: the middle consumer"));
        drop(b);
        assert_eq!(
            take_decisions("anchor-b"),
            [[y + 1]],
            "{at}: the middle consumer decides over the changed root"
        );
        // Those refreshes reclaimed the last consumer's cached row, and
        // replaced the output its lineage consumed. Restarted, it neither
        // reuses its evicted output nor awaits the replaced one.
        let selected_c = request
            .query(PlanarOutputRead {
                body_key: "anchor-c".to_owned(),
            })
            .execute()
            .unwrap();
        let c_root = selected_c.observed_sources()[0].root_entity_for_test();
        assert_eq!(
            application.has_required_row_for_test(c_root),
            !reclaims,
            "{at}: the model predicts whether the last row is reclaimed"
        );
        drop(selected_c);
        if cycle >= steady_start {
            let earlier = invalidation.native_retained_allocations_for_test();
            assert_eq!(
                earlier.len(),
                native_layout.len(),
                "all declared reservation owners are compared"
            );
            // At the minimum closed-chain budget, A and B each publish while
            // all three chain cues retain their hints until the queue drains.
            // Two completion observers belong to those two producer writers.
            if !overwritten
                && rows * row == super::custody_model::ChainCustody::new().closed_chain()
            {
                let writers = 2;
                let chain_rows = 3;
                let pending = [writers * chain_rows, writers * chain_rows, writers];
                for ((actual, baseline), (extra, size)) in earlier
                    .into_iter()
                    .zip(boundary_baseline)
                    .zip(pending.into_iter().zip(native_layout))
                {
                    assert_eq!(
                        actual.0,
                        baseline.0 + extra,
                        "{at}: undrained named Native reservations"
                    );
                    assert_eq!(
                        actual.1 - baseline.1,
                        extra as u64 * size,
                        "{at}: undrained reservations use their owner's size"
                    );
                }
            }
            drain_queue(&application, &request, c_root, &at);
            assert_eq!(
                application.queued_required_work_for_test(),
                0,
                "{at}: the queue drained before the complete retained comparison"
            );
            assert_eq!(
                invalidation.native_retained_allocations_for_test().len(),
                native_layout.len()
            );
            for ((count, bytes), declared) in invalidation
                .native_retained_allocations_for_test()
                .into_iter()
                .zip(native_layout)
            {
                assert_eq!(
                    bytes,
                    count as u64 * declared,
                    "{at}: each named Native reservation includes its capacity ticket"
                );
            }
            let bytes = (
                application.required_custody_bytes_for_test(),
                invalidation.retained_capacity_bytes(),
                application.output_lineage_retained_bytes_for_test(),
            );
            assert_eq!(
                *reclaimed_ledger.get_or_insert(bytes),
                bytes,
                "{at}: real reclamation balances the complete retained tuple"
            );
        }
        take_decisions("anchor-c");
        let owner_before = application.producer_contacts_on_this_thread_for_test();
        let mut c = start_consumer!(application, request, "anchor-c");
        settled_within_eight_advances!(c, request, format!("{at}: the last consumer"));
        drop(c);
        drain_queue(&application, &request, c_root, &at);
        assert_eq!(
            application.producer_contacts_on_this_thread_for_test() - owner_before,
            u64::from(reclaims || overwritten),
            "{at}: only lost custody or a changed consumed value contacts C"
        );
        assert_eq!(
            take_decisions("anchor-c"),
            if reclaims || overwritten {
                vec![vec![middle]]
            } else {
                vec![]
            },
            "{at}: the last consumer decides exactly when its basis requires it"
        );
        // Each restart retires what the rows it replaced had posted.
        if cycle >= steady_start {
            assert_eq!(
                application.queued_required_work_for_test(),
                0,
                "{at}: the queue drained before the complete retained comparison"
            );
            for ((count, bytes), declared) in invalidation
                .native_retained_allocations_for_test()
                .into_iter()
                .zip(native_layout)
            {
                assert_eq!(
                    bytes,
                    count as u64 * declared,
                    "{at}: each named Native reservation includes its capacity ticket"
                );
            }
            let bytes = (
                application.required_custody_bytes_for_test(),
                invalidation.retained_capacity_bytes(),
                application.output_lineage_retained_bytes_for_test(),
            );
            assert_eq!(
                *settled_ledger.get_or_insert(bytes),
                bytes,
                "{at}: real supersession and re-settlement balance the complete retained tuple"
            );
        }
        let held = application.required_custody_bytes_for_test();
        assert_eq!(
            *custody.get_or_insert(held),
            held,
            "{at}: the closed chain holds what it held a cycle before"
        );
    }
}

fn drain_queue(
    application: &application_installation::WorthQueryProgramApplicationRuntime<
        CheckpointSchema,
        program::ChainProgram,
    >,
    request: &worth_query_host::facade::application_entry::WorthQueryApplicationRequest<
        '_,
        '_,
        '_,
        CheckpointSchema,
    >,
    c_root: worth_query_host::facade::application_invariants::EntityId,
    at: &str,
) {
    if application.queued_required_work_for_test() == 0 {
        return;
    }
    let held = application.has_required_row_for_test(c_root);
    let before = application.producer_contacts_on_this_thread_for_test();
    let mut drain = start_consumer!(application, request, "anchor-b");
    settled_in_one_advance!(drain, request, "the queue-draining current middle consumer");
    drop(drain);
    // Reopening B can leave the root's retained publication cue pending.
    // Admit that exact root once; these are two named owners, not a wait loop.
    let mut root = start_root!(application, request);
    settled_in_one_advance!(root, request, "the queue-draining current root");
    drop(root);
    if !held {
        assert!(
            !application.has_required_row_for_test(c_root),
            "{at}: drainage cannot recreate the reclaimed last row"
        );
    }
    assert_eq!(
        application.producer_contacts_on_this_thread_for_test(),
        before,
        "{at}: queue drainage contacts no handler"
    );
}
