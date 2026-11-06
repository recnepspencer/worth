//! A real original request retains the exact edge-index memory refusal.
use super::*;
use worth_query_host::facade::{
    application_contribution::{
        WorthQueryManagedComputationResourceDenial, WorthQueryMemoryLimitLevel,
    },
    primary_graph::{
        advancement_requests_on_this_thread_for_test,
        bound_advancement_requests_on_this_thread_for_test,
        place_managed_computations_on_this_thread_for_test, WorthQueryApplicationOneShotDenialKind,
        WorthQueryExecutionObservation, WorthQueryExecutionPlacementForTest,
        WorthQueryProviderSessionDenialKind,
    },
    runtime::ExecutionAllocationPolicy,
};

#[test]
fn original_endpoint_index_refusal_keeps_its_precise_memory_cause_and_original_carrier() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let mut world = installation::install(&crate::installed_schema());
    {
        let scope = authentication::request_scope();
        let adapter = authentication::admit(world.application.installed_schema());
        let principal = authentication::block_on(adapter.authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ))
        .unwrap();
        let request = world.application.request(&principal, &scope);
        let source = request.retain_current_read().unwrap();
        let tree = settle_tree(&world, &request);
        assert_eq!(tree.settlement().root_producer_contacts_in_this_demand(), 1);
        let closed = tree
            .close_for_delivery(&request)
            .unwrap_or_else(|stop| panic!("original close: {:?}", stop.denial()));
        let head = request.retain_current_read().unwrap();
        {
            let _bounded = WorthQueryExecutionObservation::install_on_this_thread();
            place_managed_computations_on_this_thread_for_test(
                WorthQueryExecutionPlacementForTest::Serial,
            );
            let inherited = crate::application_invariant_acceptance::resources::EXECUTION_POLICY
                .accounting()
                .unwrap();
            let roomy = worth_foundational::ExecutionBudget::new(
                inherited.max_workers(),
                Some(64 << 20),
                inherited.work_ceiling(),
            );
            bound_advancement_requests_on_this_thread_for_test(Some(roomy));
            let _ = advancement_requests_on_this_thread_for_test();
            let admitted = world
                .application
                .with_application_advancement(&scope, |phase| {
                    // The factory keeps this SAME genuine source admission alive
                    // while it grows its first temporary index entry.
                    let selected = source
                        .select_on(&phase, world.application.runtime())
                        .unwrap();
                    let execution = phase
                        .checked_execution_request_for(world.application.product_runtime())
                        .unwrap();
                    let ExecutionAllocationPolicy::Execution(parent) =
                        execution.allocation_policy()
                    else {
                        panic!("the actual original request must have its bounded lease");
                    };
                    let charged = parent.charged_memory_bytes();
                    drop(selected);
                    charged
                })
                .unwrap();
            let observed = advancement_requests_on_this_thread_for_test();
            assert_eq!(observed.len(), 1, "one actual source-admission calibration");
            let source_peak = observed[0]
                .as_ref()
                .expect("source request admitted")
                .physical()
                .peak_charged_memory_bytes();
            assert!(
                source_peak >= admitted,
                "the source probe includes its transient allocations"
            );
            let ceiling = source_peak.checked_add(1).unwrap();
            assert!(ceiling < roomy.charged_memory_bytes().unwrap());
            bound_advancement_requests_on_this_thread_for_test(Some(
                worth_foundational::ExecutionBudget::new(
                    inherited.max_workers(),
                    Some(ceiling),
                    inherited.work_ceiling(),
                ),
            ));
            world
                .application
                .with_application_advancement(&scope, |phase| {
                    drop(
                        source
                            .select_on(&phase, world.application.runtime())
                            .expect("the calibrated ceiling admits the real D before edge growth"),
                    );
                })
                .unwrap();
            let contacts = world
                .application
                .producer_contacts_on_this_thread_for_test();
            let stopped = closed.qualify_original(&request).map(|_| ()).expect_err(
                "one byte beyond the actual source peak cannot fund an original edge index",
            );
            let WorthQueryOperationDeliveryDenial::Currentness(
                WorthQueryOutputCurrentnessDenial::Output(cause),
            ) = stopped
            else {
                panic!("the intended in-kernel index growth must refuse: {stopped:?}");
            };
            assert_eq!(
                cause.subject(),
                "original output endpoint index",
                "the real edge-index growth must be reached, not an earlier or later allocation"
            );
            assert!(
                matches!(cause.kind(), WorthQueryOutputDemandDenialKind::SourceQueryExecution(
                WorthQueryApplicationOneShotDenialKind::ExecutionRequest(WorthQueryProviderSessionDenialKind::ExecutionResource {
                    denial: WorthQueryManagedComputationResourceDenial::MemoryLimit {
                        requested, admitted, level: WorthQueryMemoryLimitLevel::Policy,
                    }, policy_ancestor: Some(_), ..
                })
            ) if requested > admitted),
                "the original memory cause must survive its enclosing handoff: {cause:?}"
            );
            assert_eq!(
                world
                    .application
                    .producer_contacts_on_this_thread_for_test(),
                contacts
            );
        }
        assert_eq!(
            request.retain_current_read().unwrap().selected_commit(),
            head.selected_commit()
        );
        let contacts = world
            .application
            .producer_contacts_on_this_thread_for_test();
        assert_eq!(
            closed.qualify_original(&request).unwrap().output_commit(),
            head.selected_commit(),
            "the same original carrier qualifies after the real bounded request ends"
        );
        assert_eq!(
            world
                .application
                .producer_contacts_on_this_thread_for_test(),
            contacts
        );
    }
    world
        .application
        .close_conditional_runtime()
        .expect("both refusal and retry release original native custody");
}
