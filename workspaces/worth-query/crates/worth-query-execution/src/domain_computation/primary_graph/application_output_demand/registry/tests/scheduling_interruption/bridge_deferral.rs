//! Scheduling classification is supplied by the production scheduling owner.
use super::*;

impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn verify_bridge_scheduling_for_test(
        first: Result<WorthQueryOutputSchedulingResult, WorthQueryOutputDemandDenial>,
        later: impl FnOnce() -> WorthQueryOutputSchedulingResult,
        temporary: bool,
    ) {
        let fixture = Fixture::new(false);
        let custody = fixture.custody();
        let WorthQueryOutputDemandAdvanceAdmission::Schedule(source) =
            fixture.registry.begin(&fixture.interest)
        else {
            panic!("the row enters scheduling");
        };
        let mut first = first;
        fixture
            .registry
            .finish_scheduling(&fixture.interest, source, &mut first);
        if !temporary {
            let denial = first.err().unwrap();
            assert_eq!(denial.recovery_posture(), Posture::Terminal);
            assert!(matches!(
                fixture.registry.begin(&fixture.interest),
                WorthQueryOutputDemandAdvanceAdmission::Failed(_)
            ));
            let state = fixture.registry.state.lock().unwrap();
            assert!(matches!(
                state.records[&fixture.interest.key].state,
                DemandState::Failed(_)
            ));
            assert!(state.records[&fixture.interest.key]
                .performed_obligations
                .is_empty());
            assert_eq!(state.obligation_reserved_bytes, 0);
            return;
        }
        assert!(
            matches!(first, Ok(WorthQueryOutputSchedulingResult::Deferred)),
            "a busy native Signal slot must defer"
        );
        assert_eq!(fixture.custody(), custody);
        {
            let state = fixture.registry.state.lock().unwrap();
            let row = &state.records[&fixture.interest.key];
            assert!(matches!(row.state, DemandState::Admitted));
            assert_eq!(row.performed_obligations.len(), 1);
            assert_eq!(row.prerequisites.len(), 1);
            assert!(row.performed_source.is_some());
        }
        let WorthQueryOutputDemandAdvanceAdmission::Schedule(source) =
            fixture.registry.begin(&fixture.interest)
        else {
            panic!("a later advance can reclaim the admitted row");
        };
        let mut result = Ok(later());
        fixture
            .registry
            .finish_scheduling(&fixture.interest, source, &mut result);
        assert!(matches!(
            result,
            Ok(WorthQueryOutputSchedulingResult::Scheduled)
        ));
        let finish = fixture
            .registry
            .prepare_selected_execution_finish(&fixture.interest, &mut record_admission())
            .unwrap();
        assert!(matches!(
            fixture.registry.begin(&fixture.interest),
            WorthQueryOutputDemandAdvanceAdmission::Execute { .. }
        ));
        let completion = WorthQueryCompletedOutputDemand {
            authority: WorthQueryAcceptedOutputAuthority::Committed(fixture.receipt.clone()),
            readiness: WorthQueryOutputReadinessDeliveryEvidence::for_test(),
            resources: None,
        };
        assert!(finish
            .publish(WorthQueryOutputCheckpoint::Ready(
                super::super::super::super::ReadyCompletion::for_test(completion)
            ))
            .is_ok());
        assert!(matches!(
            fixture.registry.begin(&fixture.interest),
            WorthQueryOutputDemandAdvanceAdmission::Ready(_)
        ));
    }
}
