//! Scheduling classification is supplied by the production scheduling owner.
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::scheduling_interruption::*;

/// Test doors on a library type, owned by the moved courts.
pub trait BridgeDeferralWorthQueryOutputDemandRegistryForTest: Sized {
    fn verify_bridge_scheduling_for_test(
        first: Result<WorthQueryOutputSchedulingResult, WorthQueryOutputDemandDenial>,
        later: impl FnOnce() -> WorthQueryOutputSchedulingResult,
        temporary: bool,
    );
}

impl BridgeDeferralWorthQueryOutputDemandRegistryForTest for WorthQueryOutputDemandRegistry {
    fn verify_bridge_scheduling_for_test(
        first: Result<WorthQueryOutputSchedulingResult, WorthQueryOutputDemandDenial>,
        later: impl FnOnce() -> WorthQueryOutputSchedulingResult,
        temporary: bool,
    ) {
        let fixture = Fixture::new(false);

        let WorthQueryOutputDemandAdvanceAdmission::Schedule(source) =
            fixture.registry.begin_for_test(&fixture.interest)
        else {
            panic!("the row enters scheduling");
        };
        let mut first = first;
        fixture
            .registry
            .finish_scheduling_for_test(&fixture.interest, source, &mut first);
        if !temporary {
            let denial = first.err().unwrap();
            assert_eq!(denial.recovery_posture(), Posture::Terminal);
            assert!(matches!(
                fixture.registry.begin_for_test(&fixture.interest),
                WorthQueryOutputDemandAdvanceAdmission::Failed(_)
            ));
            let state = (*fixture.registry.state_for_test()).lock().unwrap();
            assert!(matches!(
                state.records[&(*fixture.interest.key_for_test())].state,
                DemandState::Failed(_)
            ));
            assert!(state.records[&(*fixture.interest.key_for_test())]
                .performed_obligations
                .is_empty());

            return;
        }
        assert!(
            matches!(first, Ok(WorthQueryOutputSchedulingResult::Deferred)),
            "a busy native Signal slot must defer"
        );

        {
            let state = (*fixture.registry.state_for_test()).lock().unwrap();
            let row = &state.records[&(*fixture.interest.key_for_test())];
            assert!(matches!(row.state, DemandState::Admitted));
            assert_eq!(row.performed_obligations.len(), 1);
            assert_eq!(row.prerequisites.len(), 1);
            assert!(row.performed_source.is_some());
        }
        let WorthQueryOutputDemandAdvanceAdmission::Schedule(source) =
            fixture.registry.begin_for_test(&fixture.interest)
        else {
            panic!("a later advance can reclaim the admitted row");
        };
        let mut result = Ok(later());
        fixture
            .registry
            .finish_scheduling_for_test(&fixture.interest, source, &mut result);
        assert!(matches!(
            result,
            Ok(WorthQueryOutputSchedulingResult::Scheduled)
        ));
        let finish = fixture
            .registry
            .prepare_selected_execution_finish_for_test(&fixture.interest)
            .unwrap();
        assert!(matches!(
            fixture.registry.begin_for_test(&fixture.interest),
            WorthQueryOutputDemandAdvanceAdmission::Execute { .. }
        ));
        let completion = WorthQueryCompletedOutputDemand {
            authority: WorthQueryAcceptedOutputAuthority::Committed(fixture.receipt.clone()),
            readiness: WorthQueryOutputReadinessDeliveryEvidence::for_test_for_test(),
        };
        assert!(finish
            .publish(WorthQueryOutputCheckpoint::Ready(
                worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::ready_backing::ReadyCompletion::for_test(completion)
            ))
            .is_ok());
        assert!(matches!(
            fixture.registry.begin_for_test(&fixture.interest),
            WorthQueryOutputDemandAdvanceAdmission::Ready(_)
        ));
    }
}
