use crate::domain_computation::primary_graph::application_output_demand::registry::tests::*;
use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;

/// A selected execution that stops for its request leaves the shared row to
/// the next claim; only a stop intrinsic to the row fails it for every peer.
#[test]
fn selected_execution_failure_fails_the_row_only_for_an_intrinsic_stop() {
    let demand_key = key("selected-execution-failure", 1, 1);
    let product_occurrence = occurrence();
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|_active_phase| {
        use worth_query_execution::facade::domain_computation::WorthQueryApplicationOneShotDenialKind as Read;
        use WorthQueryOutputDemandDenialKind as Kind;
        for (kind, peer_executes) in [
            (Kind::WorkBudgetExceeded, true),
            (
                Kind::RequestAuthorization(AuthorizationRefusal::StalePrincipal),
                true,
            ),
            (Kind::SourceQueryExecution(Read::StaleScope), true),
            (Kind::ProducerUnavailable, false),
            (Kind::ProducerDomainDenied, false),
            (Kind::SourceQueryExecution(Read::CardinalityMismatch), false),
            (
                Kind::RequestAuthorization(AuthorizationRefusal::MutationPreconditionRejected),
                true,
            ),
            (Kind::SourceQueryExecution(Read::BasisUnavailable), true),
            (Kind::NoEffect, false),
        ] {
            let registry = WorthQueryOutputDemandRegistry::default();
            let admit = || {
                registry
                .admit_for_test(demand_key.clone(), None, worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(root(1)), product_occurrence, worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandAdmissionKind::Ordinary, None, None)
                .expect("the registry creates and joins actual retained membership").0
            };
            let running = admit();
            let peer = admit();
            assert!(matches!(
                registry.begin_for_test(&running),
                worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::advance_admission::WorthQueryOutputDemandAdvanceAdmission::Schedule(None)
            ));
            registry.finish_scheduling_for_test(
                &running,
                None,
                &mut Ok(WorthQueryOutputSchedulingResult::Scheduled),
            );
            let finish = registry
                .prepare_selected_execution_finish_for_test(&running)
                .expect("the owner prepares its finish before Running");
            assert!(matches!(
                registry.begin_for_test(&running),
                worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::advance_admission::WorthQueryOutputDemandAdvanceAdmission::Execute {
                    successor_of: None
                }
            ));
            let mut denial = if kind == Kind::ProducerDomainDenied {
                WorthQueryOutputDemandDenial::producer_domain_denied_for_test(
                    "the original producer".to_owned(),
                    Some("original domain refusal"),
                    worth_query_execution::facade::domain_computation::WorthQueryProducerDenialValue::DeclaredValue {
                        value_identity: "original.domain-refusal.v1",
                        bytes: b"original".to_vec(),
                    },
                )
            } else {
                WorthQueryOutputDemandDenial::new_for_test(kind.clone(), "the original producer".to_owned())
            };
            (*denial.readmission_failure_mut_for_test()) = Some("original readmission refusal");
            finish.failure(&mut denial);
            if !peer_executes {
                let state = (*registry.state_for_test()).lock().unwrap();
                let dependent = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::required_stop::head_stop(&(*state.records_for_test()), &demand_key)
                    .expect("a dependent receives the original intrinsic failure");
                assert_eq!(dependent, denial);
            }
            assert_eq!(
            denial.recovery_posture(),
            worth_query_execution::facade::domain_computation::WorthQueryOutputDemandRecoveryPosture::Terminal,
            "{kind:?} stays Terminal for the request that met it"
        );
            let next = registry.begin_for_test(&peer);
            assert_eq!(
                matches!(
                    next,
                    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::advance_admission::WorthQueryOutputDemandAdvanceAdmission::Execute {
                        successor_of: None
                    }
                ),
                peer_executes,
                "{kind:?}: a later claim reruns the row only for a request-local stop"
            );
            assert_eq!(
                matches!(
                    next,
                    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::advance_admission::WorthQueryOutputDemandAdvanceAdmission::Failed(_)
                ),
                !peer_executes
            );
            // The peer is told the original refusal and its declared value,
            // not a denial rebuilt by the code under test.
            if kind == Kind::ProducerDomainDenied {
                let worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::advance_admission::WorthQueryOutputDemandAdvanceAdmission::Failed(peer_denial) =
                    &next
                else {
                    unreachable!("a domain denial fails the row for every peer")
                };
                assert_eq!(peer_denial.kind(), Kind::ProducerDomainDenied);
                assert_eq!(peer_denial.domain_reason(), Some("original domain refusal"));
                assert_eq!(
                    peer_denial.producer_denial_bytes(),
                    Some(b"original".as_slice())
                );
            }
        }
    });
}

/// The row is keyed by producer and source epoch, not by principal: one
/// caller's authorization stop leaves it for an authorized peer, which
/// executes the same row to Ready.
#[test]
fn unauthorized_caller_leaves_the_row_for_an_authorized_peer_to_ready() {
    use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::advance_admission::WorthQueryOutputDemandAdvanceAdmission as Admission;
    let registry = WorthQueryOutputDemandRegistry::default();
    let admit = || {
        registry
            .admit_for_test(key("unauthorized-caller", 1, 1), None, worth_query_execution::facade::domain_computation::WorthQueryOperationScopeEntityBinding::from_entity_for_test(root(1)), occurrence(), worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::DemandAdmissionKind::Ordinary, None, None)
            .expect("the registry creates and joins actual retained membership").0
    };
    let unauthorized = admit();
    let authorized = admit();
    assert!(matches!(
        registry.begin_for_test(&unauthorized),
        Admission::Schedule(None)
    ));
    registry.finish_scheduling_for_test(
        &unauthorized,
        None,
        &mut Ok(WorthQueryOutputSchedulingResult::Scheduled),
    );
    let finish = registry
        .prepare_selected_execution_finish_for_test(&unauthorized)
        .expect("the unauthorized caller prepares its finish before Running");
    assert!(matches!(
        registry.begin_for_test(&unauthorized),
        Admission::Execute { successor_of: None }
    ));
    let mut denial = WorthQueryOutputDemandDenial::new_for_test(
        WorthQueryOutputDemandDenialKind::RequestAuthorization(
            AuthorizationRefusal::PermissionDenied,
        ),
        "",
    );
    finish.failure(&mut denial);
    assert_eq!(
        denial.recovery_posture(),
        worth_query_execution::facade::domain_computation::WorthQueryOutputDemandRecoveryPosture::Terminal,
        "the unauthorized caller keeps its terminal denial"
    );

    let finish = registry
        .prepare_selected_execution_finish_for_test(&authorized)
        .expect("the authorized peer prepares its finish on the rescheduled row");
    assert!(matches!(
        registry.begin_for_test(&authorized),
        Admission::Execute { successor_of: None }
    ));
    let receipt = worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let completion = worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryCompletedOutputDemand::new_for_test(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::WorthQueryAcceptedOutputAuthority::Committed(receipt), worth_query_execution::facade::domain_computation::WorthQueryOutputReadinessDeliveryEvidence::for_test_for_test());
    if finish
        .publish_for_test(worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::checkpoint::WorthQueryOutputCheckpoint::Ready(
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_output_demand::registry::ready_backing::ReadyCompletion::for_test(completion),
        ))
        .is_err()
    {
        panic!("the authorized peer publishes the row it executed");
    }
    assert!(matches!(registry.begin_for_test(&authorized), Admission::Ready(_)));
}
