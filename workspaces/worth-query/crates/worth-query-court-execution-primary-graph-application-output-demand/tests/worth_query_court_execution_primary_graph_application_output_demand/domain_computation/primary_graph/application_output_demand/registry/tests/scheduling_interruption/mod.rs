//! Scheduling interruption preserves registry custody, not output currentness.

pub use std::sync::Arc;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandState;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::scheduling_interruption::Finish;
pub use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenial;
pub use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenialKind;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputDemandInterest;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputDemandRegistry;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputSchedulingResult;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::interest;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::key;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::occurrence;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::recovery_posture;

pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandAdmissionKind;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::refresh_predecessor::OutputRefreshPredecessor;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::ready_backing::PreparedReadyBacking;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryAcceptedOutputAuthority;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryCompletedOutputDemand;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputAdvancement;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputCheckpoint;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::advance_admission::WorthQueryOutputDemandAdvanceAdmission;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputProgress;
pub use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryRetainedOutputDemandSource;
pub use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitReceipt;
pub use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandRecoveryPosture as Posture;
pub use worth_query_execution::facade::domain_computation::WorthQueryOutputReadinessDeliveryEvidence;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::scheduling_interruption::fixture::Fixture;
#[test]
fn both_scheduling_finishes_preserve_claims_and_custody_on_interruption() {
    for finish in [Finish::Ordinary, Finish::Selected] {
        for reopened in [false, true] {
            let fixture = Fixture::new(reopened);
            let denial = finish.stop(&fixture, WorthQueryOutputDemandDenialKind::Cancelled);
            assert_eq!(denial.kind(), WorthQueryOutputDemandDenialKind::Cancelled);
            assert_eq!(
                denial.recovery_posture(),
                Posture::Retryable,
                "{finish:?}: interruption leaves the row for a later claim"
            );
            {
                let state = (*fixture.registry.state_for_test()).lock().unwrap();
                let row = &(*state.records_for_test())[fixture.interest.key_for_test()];
                assert_eq!((*row.prerequisites_for_test()).len(), 1);
                assert_eq!(
                    (*row.prerequisites_for_test())[0].as_ref(),
                    fixture.upstream.key_for_test()
                );
                assert_eq!(
                    (*row.framework_required_count_for_test()),
                    1,
                    "dependent claim retained"
                );
                assert_eq!(
                    (*(*state.records_for_test())[fixture.upstream.key_for_test()]
                        .framework_required_count_for_test()),
                    1
                );
                assert_eq!(
                    (*(*state.records_for_test())[fixture.dependent.key_for_test()]
                        .prerequisites_for_test())
                    .len(),
                    1
                );
                assert_eq!((*row.performed_obligations_for_test()).len(), 1);
                assert_eq!(
                    (*(*row.performed_obligations_for_test())[0].source_commit_for_test()),
                    fixture.commit
                );
                assert_eq!(
                    (*(*row.performed_obligations_for_test())[0].source_for_test()),
                    (*(*fixture.interest.key_for_test()).source_for_test())
                );
                let source = (*row.performed_source_for_test())
                    .as_ref()
                    .expect("performed source retained");
                assert_eq!(
                    Arc::as_ptr(source.change_for_test()).cast::<()>(),
                    fixture.change
                );
                assert_eq!(
                    *(*source.receipt_for_test())
                        .committed_product_publication()
                        .composite_commit(),
                    fixture.commit
                );
                assert!(!(*row.pending_cleanup_queued_for_test()));
                if reopened {
                    let DemandState::Output(output) = row.state_for_test() else {
                        panic!("a reopened Ready is given back, not discarded");
                    };
                    assert!(matches!(
                        output.advancement_for_test(),
                        WorthQueryOutputAdvancement::Idle
                    ));
                    let Some(WorthQueryOutputCheckpoint::Ready(ready)) =
                        output.checkpoint_for_test()
                    else {
                        panic!("the exact accepted Ready remains");
                    };
                    assert_eq!(std::ptr::from_ref(&**ready).cast::<()>(), fixture.ready);
                    assert!((*row.successor_of_for_test()).is_none());
                } else {
                    assert!(matches!(*row.state_for_test(), DemandState::Admitted));
                    assert_eq!(
                            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::succession::Succession::predecessor_of_for_test(row.successor_of_for_test()),
                            Some(*fixture.receipt.idempotency_binding().key_identity())
                        );
                }
            }
            // Ready is certified and reopened before a later scheduling claim.
            let next = fixture.next_interest(reopened);
            assert!(matches!(
                fixture.registry.begin_for_test(&next),
                WorthQueryOutputDemandAdvanceAdmission::Schedule(_)
            ));
        }
    }
}
#[test]
fn other_scheduling_stops_still_fail_the_row_and_release_obligations() {
    for finish in [Finish::Ordinary, Finish::Selected] {
        let fixture = Fixture::new(true);
        let denial = finish.stop(
            &fixture,
            WorthQueryOutputDemandDenialKind::ProducerUnavailable,
        );
        assert_eq!(
            denial.kind(),
            WorthQueryOutputDemandDenialKind::ProducerUnavailable
        );
        assert_eq!(denial.recovery_posture(), Posture::Terminal);
        let state = (*fixture.registry.state_for_test()).lock().unwrap();
        let row = &(*state.records_for_test())[fixture.interest.key_for_test()];
        assert!(matches!(row.state_for_test(), DemandState::Failed(stored)
            if stored == &denial));
        assert_eq!(
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::required_stop::head_stop(state.records_for_test(), fixture.interest.key_for_test()),
            Some(denial)
        );
        assert!((*row.performed_source_for_test()).is_none());
        assert!((*row.performed_obligations_for_test()).is_empty());
    }
}
#[test]
fn interruption_variants_leave_the_owning_registry_boundary_claimable() {
    // Variant classification needs no performed World or Ready fixture.
    for scheduling in [true, false] {
        for selected in [false, true] {
            for kind in [
            WorthQueryOutputDemandDenialKind::Cancelled,
            WorthQueryOutputDemandDenialKind::TimedOut,
            WorthQueryOutputDemandDenialKind::ExecutionRequest(
                worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial::Resource(
                    worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial::WorkExhausted,
                ),
            ),
            WorthQueryOutputDemandDenialKind::ExecutionRequest(
                worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial::Resource(
                    worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial::RetainedBytesExhausted,
                ),
            ),
            WorthQueryOutputDemandDenialKind::ExecutionRequest(
                worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial::Resource(
                    worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial::ScratchCapacityExceeded,
                ),
            ),
        ] {
            let registry = WorthQueryOutputDemandRegistry::default();
            let output = key("interruption-variant", 1, 1);
            let scope = worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(root(1));
            let owner = registry.admit_for_test(output, None, scope, occurrence(), DemandAdmissionKind::Ordinary, None, None).unwrap().0;
            let mut result = Err(WorthQueryOutputDemandDenial::new_for_test(kind.clone(), "caller interrupted"));
            if scheduling {
                if selected {
                    let (admission, finish) = registry.begin_admitted_for_test(&owner).unwrap();
                    let WorthQueryOutputDemandAdvanceAdmission::Schedule(source) = admission else { panic!("schedule"); };
                    finish.unwrap().finish_for_test(source, &mut result);
                } else {
                    let WorthQueryOutputDemandAdvanceAdmission::Schedule(source) = registry.begin_for_test(&owner) else { panic!("schedule"); };
                    registry.finish_scheduling_for_test(&owner, source, &mut result);
                }
            } else {
                assert!(matches!(registry.begin_for_test(&owner), WorthQueryOutputDemandAdvanceAdmission::Schedule(_)));
                registry.finish_scheduling_for_test(&owner,None,&mut Ok(WorthQueryOutputSchedulingResult::Scheduled));
                let finish=selected.then(||registry.prepare_selected_execution_finish_for_test(&owner).unwrap());
                assert!(matches!(registry.begin_for_test(&owner),WorthQueryOutputDemandAdvanceAdmission::Execute {..}));
                let denial=result.as_mut().err().expect("interrupted attempt");
                if let Some(finish)=finish { finish.failure(denial); }
                else { registry.finish_execution_failure_for_test(&owner,denial); }
            }
            let denial=result.err().expect("interrupted attempt");
            assert_eq!(denial.kind(),kind);
            assert_eq!(denial.recovery_posture(),Posture::Retryable);
            let next=registry.begin_for_test(&owner);
            assert!(if scheduling { matches!(next,WorthQueryOutputDemandAdvanceAdmission::Schedule(_)) }
                else { matches!(next,WorthQueryOutputDemandAdvanceAdmission::Execute {..}) }, "interruption preserves an actual next claim");
        }
        }
    }
}
#[test]
fn a_new_key_stays_claimable_after_a_caller_resource_stop() {
    use worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial as Advance;
    use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
    for finish in [Finish::Ordinary, Finish::Selected] {
        let fixture = Fixture::new(false);
        (*(*(*fixture.registry.state_for_test())
            .lock()
            .unwrap()
            .records_mut_for_test())
        .get_mut(fixture.interest.key_for_test())
        .unwrap()
        .successor_of_mut_for_test()) = None;
        let kind = WorthQueryOutputDemandDenialKind::ExecutionRequest(Advance::Resource(
            Resource::WorkExhausted,
        ));
        let denial = finish.stop(&fixture, kind.clone());
        assert_eq!(denial.kind(), kind);
        assert_eq!(denial.recovery_posture(), Posture::Retryable);
        let peer = fixture.next_interest(false);
        assert!(matches!(
            fixture.registry.begin_for_test(&peer),
            WorthQueryOutputDemandAdvanceAdmission::Schedule(_)
        ));
    }
}

#[path = "checkpoint/mod.rs"]
pub mod checkpoint;
