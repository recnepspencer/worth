//! The same caller stop classification applies after publication and delivery.
use crate::domain_computation::primary_graph::application_output_demand::registry::tests::scheduling_interruption::*;
use worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial as Advance;
use worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial as Resource;

#[test]
fn checkpoint_finishes_preserve_performed_custody_or_report_the_terminal_stop() {
    use WorthQueryOutputDemandDenialKind as Kind;
    for selected in [false, true] {
        for (kind, fails_row) in [(Kind::Cancelled, false), (Kind::ProducerUnavailable, true)] {
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

#[test]
fn checkpoint_denial_variants_preserve_the_exact_custody_transition() {
    use WorthQueryOutputDemandDenialKind as Kind;
    let mut receipt=worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application_with_output_demand_observation();
    let change = Arc::new(receipt.take_performed_relational_product_change().unwrap());
    let observation = receipt
        .committed_product_publication()
        .take_output_demand_observation_for_test()
        .unwrap();
    let source = WorthQueryRetainedOutputDemandSource::new_for_test(
        receipt.clone(),
        Arc::clone(&change),
        observation,
        None,
    );
    let commit = receipt
        .committed_product_publication()
        .composite_commit()
        .clone();
    let scope = receipt.principal_scope().scope();
    let occurrence = receipt.product_branch().occurrence_for_test();
    // One genuinely issued source is shared as immutable setup. Each variant
    // drives a fresh registry row at the boundary that owns classification.
    for selected in [false, true] {
        for (kind, terminal) in [
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
            let registry = WorthQueryOutputDemandRegistry::default();
            let output = key("checkpoint-denial", 1, 1);
            let owner = registry
                .admit_for_test(
                    output.clone(),
                    None,
                    scope,
                    occurrence,
                    DemandAdmissionKind::Ordinary,
                    None,
                    None,
                )
                .unwrap()
                .0;
            {
                let mut state = registry.state_for_test().lock().unwrap();
                let mut row = state.records_mut_for_test().get_mut(&output).unwrap();
                *row.state_mut_for_test()=DemandState::Output(WorthQueryOutputProgress::new_for_test(WorthQueryOutputCheckpoint::Published {
                    receipt:receipt.clone(),
                    delivery:worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryPendingOutputDelivery::NoChange,
                    ready_backing:PreparedReadyBacking::new_for_test(),
                }));
                *row.performed_source_mut_for_test() = Some(source.clone());
                *row.performed_obligations_mut_for_test()=vec![worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::PerformedOutputObligation::new_for_test(commit.clone(),output.source_for_test().clone())];
            }
            let prepared = selected.then(|| {
                registry
                    .prepare_selected_checkpoint_finish_for_test(&owner)
                    .unwrap()
            });
            let (prepared, claim, checkpoint) = if let Some(prepared) = prepared {
                let (prepared, claim, checkpoint) = prepared
                    .claim_for_test()
                    .unwrap()
                    .expect("actual Published claim");
                (Some(prepared), claim, checkpoint)
            } else {
                let WorthQueryOutputDemandAdvanceAdmission::AdvanceCheckpoint { claim, checkpoint } =
                    registry.begin_for_test(&owner)
                else {
                    panic!("actual Published claim");
                };
                (None, claim, checkpoint)
            };
            let mut denial =
                WorthQueryOutputDemandDenial::new_for_test(kind.clone(), "caller delivery stop");
            *denial.readmission_failure_mut_for_test() = Some("original delivery refusal");
            if let Some(prepared) = prepared {
                prepared
                    .finish_for_test(claim, checkpoint, Some(&denial))
                    .unwrap_or_else(|_| {
                        panic!("selected finish must restore the exact claimed checkpoint")
                    });
            } else {
                registry
                    .finish_checkpoint_for_test(&owner, claim, checkpoint, Some(&denial))
                    .unwrap();
            }
            // The selected bounded finish defers variable obligation destruction
            // to its existing admitted cleanup owner.
            if terminal && selected {
                registry.drain_closed_cleanup_admitted().unwrap();
            }
            {
                let state = registry.state_for_test().lock().unwrap();
                let row = &state.records_for_test()[&output];
                assert_eq!(
                    row.performed_obligations_for_test().len(),
                    usize::from(!terminal)
                );
                // Published custody stays with the live reader; only mandatory
                // performed obligations discharge at a terminal stop.
                assert!(row.performed_source_for_test().is_some());
                if !terminal {
                    assert_eq!(
                        Arc::as_ptr(
                            row.performed_source_for_test()
                                .as_ref()
                                .unwrap()
                                .change_for_test()
                        ),
                        Arc::as_ptr(&change)
                    );
                } else {
                    assert_eq!(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::required_stop::head_stop(state.records_for_test(),&output),Some(denial.clone()));
                }
            }
            let next = registry.begin_for_test(&owner);
            if terminal {
                assert!(
                    matches!(next,WorthQueryOutputDemandAdvanceAdmission::Failed(actual) if actual==denial)
                );
            } else {
                assert!(matches!(
                    next,
                    WorthQueryOutputDemandAdvanceAdmission::AdvanceCheckpoint { .. }
                ));
            }
        }
    }
}
