use super::*;

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
