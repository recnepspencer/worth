//! The same caller stop classification applies after publication and delivery.
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::scheduling_interruption::*;
use worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial as Advance;
use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;

#[test]
fn checkpoint_finishes_keep_caller_stops_off_the_shared_row() {
    use WorthQueryOutputDemandDenialKind as Kind;
    for selected in [false, true] {
        for (kind, fails_row) in [
            (Kind::Cancelled, false),
            (Kind::TimedOut, false),
            (
                Kind::ExecutionRequest(Advance::Resource(Resource::WorkExhausted)),
                false,
            ),
            (
                Kind::ExecutionRequest(Advance::Resource(Resource::RetainedBytesExhausted)),
                false,
            ),
            (
                Kind::ExecutionRequest(Advance::Resource(Resource::ScratchCapacityExceeded)),
                false,
            ),
            (Kind::ProducerUnavailable, true),
            (Kind::RequiredSuccessorStateUnavailable, true),
        ] {
            let fixture = Fixture::new(false);
            {
                let mut state = (*fixture.registry.state_for_test()).lock().unwrap();
                (*(*state.records_mut_for_test()).get_mut(fixture.interest.key_for_test()).unwrap().state_mut_for_test()) = DemandState::Output(
                    WorthQueryOutputProgress::new_for_test(WorthQueryOutputCheckpoint::Published {
                        receipt: fixture.receipt.clone(),
                        delivery: worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryPendingOutputDelivery::NoChange,
                        ready_backing: PreparedReadyBacking::for_test_for_test(),
                    }),
                );
            }
            let prepared = selected.then(|| {
                fixture
                    .registry
                    .prepare_selected_checkpoint_finish_for_test(&fixture.interest)
                    .unwrap()
            });
            let (prepared, claim, checkpoint) = if let Some(prepared) = prepared {
                let (prepared, claim, checkpoint) = prepared
                    .claim_for_test()
                    .unwrap()
                    .expect("published row is claimable");
                (Some(prepared), claim, checkpoint)
            } else {
                let WorthQueryOutputDemandAdvanceAdmission::AdvanceCheckpoint { claim, checkpoint } =
                    fixture.registry.begin_for_test(&fixture.interest)
                else {
                    panic!("published row is claimable");
                };
                (None, claim, checkpoint)
            };
            let mut denial = WorthQueryOutputDemandDenial::new_for_test(
                kind.clone(),
                "caller stopped delivery".to_owned(),
            );
            (*denial.readmission_failure_mut_for_test()) = Some("original delivery refusal");
            if let Some(prepared) = prepared {
                assert!(prepared
                    .finish_for_test(claim, checkpoint, Some(&denial))
                    .is_ok());
            } else {
                fixture
                    .registry
                    .finish_checkpoint_for_test(&fixture.interest, claim, checkpoint, Some(&denial))
                    .unwrap();
            }
            {
                let state = (*fixture.registry.state_for_test()).lock().unwrap();
                let DemandState::Output(output) =
                    (*state.records_for_test())[fixture.interest.key_for_test()].state_for_test()
                else {
                    panic!("publication remains retained");
                };
                assert!((*output.checkpoint_for_test()).is_some());
                if fails_row {
                    assert_eq!(
                        worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::required_stop::head_stop(
                            state.records_for_test(),
                            fixture.interest.key_for_test(),
                        ),
                        Some(denial.clone())
                    );
                    assert!(
                        matches!(&output.advancement_for_test(), WorthQueryOutputAdvancement::Stopped { denial, .. }
                        if denial.kind() == kind)
                    );
                } else {
                    assert!(matches!(
                        output.advancement_for_test(),
                        WorthQueryOutputAdvancement::Idle
                    ));
                }
            }
            if !fails_row {
                {
                    let state = (*fixture.registry.state_for_test()).lock().unwrap();
                    let row = &(*state.records_for_test())[fixture.interest.key_for_test()];
                    assert_eq!((*row.prerequisites_for_test()).len(), 1);
                    assert_eq!(
                        (*row.prerequisites_for_test())[0].as_ref(),
                        fixture.upstream.key_for_test()
                    );
                    assert_eq!((*row.framework_required_count_for_test()), 1);
                    assert_eq!(
                        (*(*state.records_for_test())[fixture.upstream.key_for_test()]
                            .framework_required_count_for_test()),
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
                        .expect("original source retained");
                    assert_eq!(
                        Arc::as_ptr(source.change_for_test()).cast::<()>(),
                        fixture.change
                    );
                    assert!(!(*row.pending_cleanup_queued_for_test()));
                }
                assert!(matches!(
                    fixture.registry.begin_for_test(&fixture.interest),
                    WorthQueryOutputDemandAdvanceAdmission::AdvanceCheckpoint { .. }
                ));
            } else {
                assert!(matches!(fixture.registry.begin_for_test(&fixture.interest),
                    WorthQueryOutputDemandAdvanceAdmission::Failed(denial) if denial.kind() == kind));
            }
        }
    }
}
