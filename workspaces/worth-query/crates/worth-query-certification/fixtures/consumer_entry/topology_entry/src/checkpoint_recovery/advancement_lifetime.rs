use super::*;
#[test]
fn a_leased_output_demand_carries_its_request_through_bridge_and_signal() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let controls = WorthQueryOutputDemandControls::new(
        NonZeroUsize::new(4_096).unwrap(),
        NonZeroUsize::new(8_192).unwrap(),
    )
    .source_currentness_work(
        NonZeroUsize::new(
            application
                .runtime()
                .output_demand_resource_profile()
                .limits()
                .source_currentness_work(),
        )
        .unwrap(),
    );
    let mut output = request
        .start_program_outputs::<CheckpointProgram, CheckpointRoot>(
            &application,
            PlanarOutputDemand::new("anchor-a"),
            controls,
        )
        .unwrap();

    use worth_query_host::facade::primary_graph::{
        advancement_requests_on_this_thread_for_test as reports,
        bound_advancement_requests_on_this_thread_for_test as bound,
        place_managed_computations_on_this_thread_for_test as place,
        signal_request_charges_on_this_thread_for_test as signal_charges,
        WorthQueryExecutionPlacementForTest as Placement,
    };
    struct Restore(Placement, Option<worth_foundational::ExecutionBudget>);
    impl Drop for Restore {
        fn drop(&mut self) {
            place(self.0);
            bound(self.1);
        }
    }
    let _restore = Restore(
        place(Placement::Leased(NonZeroUsize::MIN)),
        bound(Some(super::support::CHECKPOINT_EXECUTION_POLICY.budget())),
    );
    use worth_query_host::facade::primary_graph::partitioned_computation_runs_on_this_thread_for_test as computation_runs;
    reports();
    signal_charges();
    computation_runs();
    let outcome = output
        .advance(&application, &request)
        .expect("the consumer shares its advancement request");
    // Each producer reserves the installed 26 indexes before publication.
    // Root changes body + idempotency; each child creates three bodies, three
    // successor relations and idempotency. Terms are patches, reads, routing,
    // adjacency, edits, comparisons, and retained nonempty/empty paths.
    let indexes = 2 * super::advancement_custody::PRIMARY_INDEX_COUNT;
    let root_index_work = indexes + 2 + 8 + 8 + 1 + 2;
    let final_index_work = indexes + 7 + 22 + 21 + 6 + 13 + 9 + (14 * 64 + 8);
    let alternate_index_work = indexes + 7 + 22 + 21 + 6 + 13 + 24 + (18 * 64 + 4);
    let index_units = root_index_work + final_index_work + alternate_index_work;
    let WorthQueryApplicationProgramOutputProgress::Settled(settled) = &outcome else {
        panic!("the three producer publications settle");
    };
    let assert_work =
        |receipt: &worth_query_host::facade::primary_graph::WorthQueryApplicationCommitReceipt,
         rows,
         expected| {
            let evidence = receipt.mutation_work().unwrap();
            let work = evidence.index_maintenance_work();
            assert_eq!(
                (work.patch_records, work.generation_publications_reserved),
                (
                    rows,
                    super::advancement_custody::PRIMARY_INDEX_COUNT as usize
                )
            );
            assert_eq!(work.work_units as u64, expected);
        };
    assert_work(settled.root_receipt().unwrap(), 2, root_index_work);
    for (_, output) in settled.outputs_for::<CheckpointSchema, PlanarOutputToFinalConnection>() {
        assert_work(
            output.application_commit_receipt().unwrap(),
            7,
            final_index_work,
        );
    }
    for (_, output) in
        settled.outputs_for::<CheckpointSchema, PlanarOutputToAlternateFinalConnection>()
    {
        assert_work(
            output.application_commit_receipt().unwrap(),
            7,
            alternate_index_work,
        );
    }
    let reports = reports();
    assert_eq!(
        reports.len(),
        1,
        "one request spans the caller pass and delivery"
    );
    let charged = reports[0].as_ref().unwrap().charged_work();
    assert!(
        charged
            <= super::support::CHECKPOINT_EXECUTION_POLICY
                .budget()
                .work_ceiling()
    );
    let signal_units: u64 = signal_charges().into_iter().sum();
    assert!(
        signal_units > 0,
        "the Signal owner executed its own checkpoints"
    );
    let query_units: u64 = computation_runs()
        .into_iter()
        .filter_map(|(_, report)| report)
        .map(|report| report.charged_work())
        .sum();
    assert_eq!(
        query_units, 0,
        "this producer invokes no managed computation"
    );
    // This advance's plan sequence contains ten principal-target traversals;
    // each has the seeded shape used by the independent read-work oracle.
    let relational_units = 10 * super::advancement_custody::PRINCIPAL_TARGET_READ_WORK;
    assert_eq!(
        charged,
        query_units + signal_units + relational_units + index_units,
        "the parent includes Query, Signal, native read and index work exactly"
    );
    let bridge_units = charged - query_units - signal_units - relational_units - index_units;
    assert_eq!(
        bridge_units, 0,
        "Bridge adds no checkpoint work on this path"
    );
}

#[test]
fn mutation_host_calls_own_the_request_and_prepared_products_hold_only_facts() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    drop(settle(&request, &application));
    use worth_query_host::facade::{
        application_entry::{
            WorthQueryApplicationMutationOutcome as Outcome,
            WorthQueryApplicationProgramMutationPreparation as Preparation,
        },
        primary_graph::{
            advancement_requests_on_this_thread_for_test as reports,
            place_managed_computations_on_this_thread_for_test as place,
            WorthQueryExecutionPlacementForTest as Placement,
        },
    };
    struct Restore(Placement);
    impl Drop for Restore {
        fn drop(&mut self) {
            place(self.0);
        }
    }
    let _restore = Restore(place(Placement::Leased(NonZeroUsize::MIN)));
    let input = || {
        crate::PlanarEdit(crate::PlanarMutation {
            scope_key: "anchor-a".into(),
            operation: worth_query_consumer_values::PlanarOperation::VerifyCurrentOutputs(vec![
                worth_query_consumer_values::PlanarCurrentOutputExpectation {
                    producer_key: "anchor-a".into(),
                    output_key: "anchor-a".into(),
                },
            ]),
        })
    };
    let observed = request
        .query(PlanarRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    reports();
    let outcome = request
        .mutate(input())
        .expect_source(observed.observed_sources()[0].clone())
        .idempotency(&9_761_u64)
        .execute_in_program(
            &application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(matches!(outcome, Outcome::Committed { .. }));
    assert_eq!(
        reports().len(),
        1,
        "one call shares its preparation and commit request"
    );

    let observed = request
        .query(PlanarRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    reports();
    let retained = request
        .mutate(input())
        .expect_source(observed.observed_sources()[0].clone())
        .idempotency(&9_765_u64)
        .execute_retained_in_program(
            &application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(matches!(retained,
        worth_query_host::facade::application_entry::WorthQueryApplicationRetainedMutationOutcome::Committed { .. }
    ));
    assert_eq!(
        reports().len(),
        1,
        "retained one-call commits borrow the active request"
    );
    drop(retained);

    let observed = request
        .query(PlanarRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    let mut mutation = request
        .mutate(input())
        .expect_source(observed.observed_sources()[0].clone())
        .idempotency(&9_762_u64);
    reports();
    let Preparation::Prepared(prepared) = mutation
        .prepare_in_program(
            &application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
    else {
        panic!("a fresh mutation produces an unpublished candidate");
    };
    assert_eq!(
        reports().len(),
        1,
        "preparation closes before delivering the product"
    );
    assert!(matches!(
        prepared
            .commit(worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation),
        Outcome::Committed { .. }
    ));
    assert_eq!(
        reports().len(),
        1,
        "the later host commit opens its own request"
    );

    let observed = request
        .query(PlanarRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    let mut mutation = request
        .mutate(input())
        .expect_source(observed.observed_sources()[0].clone())
        .idempotency(&9_763_u64);
    reports();
    let prepared = mutation
        .prepare_in_program(
            &application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    drop(prepared);
    assert_eq!(reports().len(), 1, "dropping data opens no commit request");
}

#[test]
fn signal_reservation_uses_the_advancement_work_ceiling() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    // This computational allowance is declared by the caller, not fitted to a measured total.
    let handler_allowance = 8_192;
    let controls = WorthQueryOutputDemandControls::new(
        NonZeroUsize::new(4_096).unwrap(),
        NonZeroUsize::new(handler_allowance).unwrap(),
    )
    .source_currentness_work(
        NonZeroUsize::new(
            application
                .runtime()
                .output_demand_resource_profile()
                .limits()
                .source_currentness_work(),
        )
        .unwrap(),
    );
    let mut output = request
        .start_program_outputs::<CheckpointProgram, CheckpointRoot>(
            &application,
            PlanarOutputDemand::new("anchor-a"),
            controls,
        )
        .unwrap();
    use worth_query_host::facade::primary_graph::{
        advancement_requests_on_this_thread_for_test as reports,
        bound_advancement_requests_on_this_thread_for_test as bound,
        place_managed_computations_on_this_thread_for_test as place,
        WorthQueryExecutionPlacementForTest as Placement,
    };
    struct Restore(Placement, Option<worth_foundational::ExecutionBudget>);
    impl Drop for Restore {
        fn drop(&mut self) {
            place(self.0);
            bound(self.1);
        }
    }
    let _restore = Restore(
        place(Placement::Leased(NonZeroUsize::MIN)),
        bound(Some(worth_foundational::ExecutionBudget::new(
            NonZeroUsize::MIN,
            super::support::CHECKPOINT_EXECUTION_POLICY
                .budget()
                .charged_memory_bytes(),
            handler_allowance as u64,
        ))),
    );
    reports();
    let outcome = output.advance(&application, &request);
    let reports = reports();
    let denial = match outcome {
        Err(denial) => denial,
        Ok(_) => panic!("the Signal reservation exceeds this request"),
    };
    let worth_query_host::facade::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand(
        worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(denial),
    ) = denial else { panic!("the existing Signal scheduling path reports its refusal"); };
    // Payload typing for this existing Signal denial remains in part three.
    assert_eq!(
        denial.kind(),
        WorthQueryOutputDemandDenialKind::ExecutionRequest(
            worth_query_host::facade::application_contribution::WorthQueryAdvancementDenial::Resource(
                worth_query_host::facade::application_contribution::WorthQueryManagedComputationResourceDenial::WorkExhausted,
            ),
        )
    );
    assert_eq!(
        reports.len(),
        1,
        "the advancement never resets its request budget"
    );
}
