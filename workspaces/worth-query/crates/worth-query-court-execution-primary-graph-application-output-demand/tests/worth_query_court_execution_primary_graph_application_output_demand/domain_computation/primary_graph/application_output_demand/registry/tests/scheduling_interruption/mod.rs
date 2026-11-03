//! Scheduling interruption preserves registry custody, not output currentness.

pub use std::sync::Arc;
pub use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::DemandState;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::scheduling_interruption::Finish;
pub use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenial;
pub use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandDenialKind;
pub use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputDemandInterest;
pub use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputDemandRegistry;
pub use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::WorthQueryOutputSchedulingResult;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::interest;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::key;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::support::occurrence;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::recovery_posture;

pub use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::DemandAdmissionKind;
pub use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::refresh_predecessor::OutputRefreshPredecessor;
pub use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::ready_backing::PreparedReadyBacking;
pub use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::WorthQueryAcceptedOutputAuthority;
pub use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::WorthQueryCompletedOutputDemand;
pub use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputAdvancement;
pub use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputCheckpoint;
pub use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::advance_admission::WorthQueryOutputDemandAdvanceAdmission;
pub use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputProgress;
pub use worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::WorthQueryPerformedOutputDemandSource;
pub use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitReceipt;
pub use worth_query_execution::facade::domain_computation::WorthQueryOutputDemandRecoveryPosture as Posture;
pub use worth_query_execution::facade::domain_computation::WorthQueryOutputReadinessDeliveryEvidence;
pub use worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::application_output_demand::registry::tests::scheduling_interruption::fixture::Fixture;
#[test]
fn both_scheduling_finishes_preserve_claims_and_custody_on_interruption() {
    for finish in [Finish::Ordinary, Finish::Selected] {
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
            for reopened in [false, true] {
                let fixture = Fixture::new(reopened);
                let denial = finish.stop(&fixture, kind.clone());
                assert_eq!(denial.kind(), kind);
                assert_eq!(
                    denial.recovery_posture(),
                    Posture::Retryable,
                    "{finish:?}: interruption leaves the row for a later claim"
                );
                {
                    let state = fixture.registry.state.lock().unwrap();
                    let row = &state.records[&fixture.interest.key];
                    assert_eq!(row.prerequisites.len(), 1);
                    assert_eq!(row.prerequisites[0].as_ref(), &fixture.upstream.key);
                    assert_eq!(row.framework_required_count, 1, "dependent claim retained");
                    assert_eq!(
                        state.records[&fixture.upstream.key].framework_required_count,
                        1
                    );
                    assert_eq!(state.records[&fixture.dependent.key].prerequisites.len(), 1);
                    assert_eq!(row.performed_obligations.len(), 1);
                    assert_eq!(row.performed_obligations[0].source_commit, fixture.commit);
                    assert_eq!(
                        row.performed_obligations[0].source,
                        fixture.interest.key.source
                    );
                    let source = row
                        .performed_source
                        .as_ref()
                        .expect("performed source retained");
                    assert_eq!(Arc::as_ptr(&source.change).cast::<()>(), fixture.change);
                    assert_eq!(
                        *source
                            .receipt
                            .committed_product_publication()
                            .composite_commit(),
                        fixture.commit
                    );
                    assert!(!row.pending_cleanup_queued);
                    if reopened {
                        let DemandState::Output(output) = &row.state else {
                            panic!("a reopened Ready is given back, not discarded");
                        };
                        assert!(matches!(
                            output.advancement(),
                            WorthQueryOutputAdvancement::Idle
                        ));
                        let Some(WorthQueryOutputCheckpoint::Ready(ready)) = &output.checkpoint
                        else {
                            panic!("the exact accepted Ready remains");
                        };
                        assert_eq!(std::ptr::from_ref(&**ready).cast::<()>(), fixture.ready);
                        assert!(row.successor_of.is_none());
                    } else {
                        assert!(matches!(row.state, DemandState::Admitted));
                        assert_eq!(
                            worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::succession::Succession::predecessor_of(&row.successor_of),
                            Some(*fixture.receipt.idempotency_binding().key_identity())
                        );
                    }
                }
                // Ready is certified and reopened before a later scheduling claim.
                let next = fixture.next_interest(reopened);
                assert!(matches!(
                    fixture.registry.begin(&next),
                    WorthQueryOutputDemandAdvanceAdmission::Schedule(_)
                ));
            }
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
        let state = fixture.registry.state.lock().unwrap();
        let row = &state.records[&fixture.interest.key];
        assert!(matches!(&row.state, DemandState::Failed(stored)
            if stored == &denial));
        assert_eq!(
            worth_query_execution::domain_computation::primary_graph::application_output_demand::registry::required_stop::head_stop(&state.records, &fixture.interest.key),
            Some(denial)
        );
        assert!(row.performed_source.is_none());
        assert!(row.performed_obligations.is_empty());
    }
}
#[test]
fn interrupted_execution_reports_the_same_retryable_posture() {
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
            let fixture = Fixture::new(false);
            assert!(matches!(
                fixture.registry.begin(&fixture.interest),
                WorthQueryOutputDemandAdvanceAdmission::Schedule(_)
            ));
            fixture.registry.finish_scheduling(
                &fixture.interest,
                None,
                &mut Ok(WorthQueryOutputSchedulingResult::Scheduled),
            );
            let finish = selected.then(|| {
                fixture
                    .registry
                    .prepare_selected_execution_finish(&fixture.interest)
                    .unwrap()
            });
            assert!(matches!(
                fixture.registry.begin(&fixture.interest),
                WorthQueryOutputDemandAdvanceAdmission::Execute { .. }
            ));
            let mut denial = WorthQueryOutputDemandDenial::new(kind, "execution stopped");
            if let Some(finish) = finish {
                finish.failure(&mut denial);
            } else {
                fixture
                    .registry
                    .finish_execution_failure(&fixture.interest, &mut denial);
            }
            assert_eq!(denial.recovery_posture(), Posture::Retryable);
        }
    }
}
#[test]
fn a_new_key_stays_claimable_after_a_caller_resource_stop() {
    use worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial as Advance;
    use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;
    for finish in [Finish::Ordinary, Finish::Selected] {
        let fixture = Fixture::new(false);
        fixture
            .registry
            .state
            .lock()
            .unwrap()
            .records
            .get_mut(&fixture.interest.key)
            .unwrap()
            .successor_of = None;
        let kind = WorthQueryOutputDemandDenialKind::ExecutionRequest(Advance::Resource(
            Resource::WorkExhausted,
        ));
        let denial = finish.stop(&fixture, kind.clone());
        assert_eq!(denial.kind(), kind);
        assert_eq!(denial.recovery_posture(), Posture::Retryable);
        let peer = fixture.next_interest(false);
        assert!(matches!(
            fixture.registry.begin(&peer),
            WorthQueryOutputDemandAdvanceAdmission::Schedule(_)
        ));
    }
}

#[path = "checkpoint/mod.rs"]
pub mod checkpoint;
