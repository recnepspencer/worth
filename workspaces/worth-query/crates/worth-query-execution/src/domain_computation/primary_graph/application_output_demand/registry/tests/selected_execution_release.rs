use super::*;

#[test]
fn selected_release_refuses_before_running_then_releases_without_new_allowance() {
    let demand_key = key("selected-request-rejection", 1, 1);
    let product_occurrence = occurrence();
    crate::domain_computation::primary_graph::with_test_advancement(|_active_phase| {
        let registry = WorthQueryOutputDemandRegistry::default();
        let scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(root(1));
        let admit = || {
            registry
                .admit(
                    demand_key.clone(),
                    None,
                    scope,
                    product_occurrence,
                    super::super::DemandAdmissionKind::Ordinary,
                    None,
                    None,
                    &mut record_admission(),
                )
                .expect("the registry creates and joins actual retained membership")
        };
        let denied = admit();
        let peer = admit();
        assert!(matches!(
            registry.begin(&denied),
            super::super::WorthQueryOutputDemandAdvanceAdmission::Schedule(None)
        ));
        registry.finish_scheduling(
            &denied,
            None,
            &mut Ok(WorthQueryOutputSchedulingResult::Scheduled),
        );
        let prior_wake = denied.notifications.generation();
        let mut measured = record_admission();
        drop(
            registry
                .prepare_selected_execution_finish(&denied, &mut measured)
                .expect("the actual owner prepares release before Running"),
        );
        let full_work = measured.charged_work();
        let budget = |work| {
            InvalidationEditAdmission::new(CompanionPreflightBudget {
                maximum_work_visits: work,
                maximum_preparation_bytes: 8 * 1024 * 1024,
            })
        };
        let mut short = budget(full_work - 1);
        let refusal = registry
            .prepare_selected_execution_finish(&denied, &mut short)
            .err()
            .expect("one short unit refuses before any execution-state change");
        assert_eq!(
            refusal.kind(),
            WorthQueryOutputDemandDenialKind::WorkBudgetExceeded
        );
        assert!(matches!(
            registry.state.lock().unwrap().records[&demand_key].state,
            DemandState::Scheduled
        ));
        assert_eq!(denied.notifications.generation(), prior_wake);

        let mut exact = budget(full_work);
        let release = registry
            .prepare_selected_execution_finish(&denied, &mut exact)
            .expect("the same retained membership retries with its exact allowance");
        assert_eq!(exact.remaining_work(), 0);
        assert!(matches!(
            registry.begin(&denied),
            super::super::WorthQueryOutputDemandAdvanceAdmission::Execute { successor_of: None }
        ));
        assert!(matches!(
            registry.begin(&peer),
            super::super::WorthQueryOutputDemandAdvanceAdmission::Pending
        ));
        release.relinquish();
        assert_eq!(denied.notifications.generation(), prior_wake + 1);
        assert!(matches!(
            registry.begin(&peer),
            super::super::WorthQueryOutputDemandAdvanceAdmission::Execute { successor_of: None }
        ));
    });
}

/// A selected execution that stops for its request leaves the shared row to
/// the next claim; only a stop intrinsic to the row fails it for every peer.
#[test]
fn selected_execution_failure_fails_the_row_only_for_an_intrinsic_stop() {
    let demand_key = key("selected-execution-failure", 1, 1);
    let product_occurrence = occurrence();
    crate::domain_computation::primary_graph::with_test_advancement(|_active_phase| {
        use crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind as Authorization;
        use crate::domain_computation::primary_graph::WorthQueryApplicationOneShotDenialKind as Read;
        use WorthQueryOutputDemandDenialKind as Kind;
        for (kind, peer_executes) in [
            (Kind::WorkBudgetExceeded, true),
            (
                Kind::RequestAuthorization(Authorization::StalePrincipal),
                true,
            ),
            (Kind::SourceQueryExecution(Read::StaleScope), true),
            (Kind::ProducerUnavailable, false),
            (Kind::SourceQueryExecution(Read::CardinalityMismatch), false),
            (
                Kind::RequestAuthorization(Authorization::MutationPreconditionRejected),
                true,
            ),
            (Kind::SourceQueryExecution(Read::BasisUnavailable), true),
            (Kind::NoEffect, false),
        ] {
            let registry = WorthQueryOutputDemandRegistry::default();
            let admit = || {
                registry
                .admit(
                    demand_key.clone(),
                    None,
                    crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(root(1)),
                    product_occurrence,
                    super::super::DemandAdmissionKind::Ordinary,
                    None,
                    None,
                    &mut record_admission(),
                )
                .expect("the registry creates and joins actual retained membership")
            };
            let running = admit();
            let peer = admit();
            assert!(matches!(
                registry.begin(&running),
                super::super::WorthQueryOutputDemandAdvanceAdmission::Schedule(None)
            ));
            registry.finish_scheduling(
                &running,
                None,
                &mut Ok(WorthQueryOutputSchedulingResult::Scheduled),
            );
            let finish = registry
                .prepare_selected_execution_finish(&running, &mut record_admission())
                .expect("the owner prepares its finish before Running");
            assert!(matches!(
                registry.begin(&running),
                super::super::WorthQueryOutputDemandAdvanceAdmission::Execute {
                    successor_of: None
                }
            ));
            let mut denial = WorthQueryOutputDemandDenial::new(kind, "");
            finish.failure(&mut denial);
            assert_eq!(
            denial.recovery_posture(),
            crate::domain_computation::primary_graph::WorthQueryOutputDemandRecoveryPosture::Terminal,
            "{kind:?} stays Terminal for the request that met it"
        );
            let next = registry.begin(&peer);
            assert_eq!(
                matches!(
                    next,
                    super::super::WorthQueryOutputDemandAdvanceAdmission::Execute {
                        successor_of: None
                    }
                ),
                peer_executes,
                "{kind:?}: a later claim reruns the row only for a request-local stop"
            );
            assert_eq!(
                matches!(
                    next,
                    super::super::WorthQueryOutputDemandAdvanceAdmission::Failed(_)
                ),
                !peer_executes
            );
        }
    });
}

/// The row is keyed by producer and source epoch, not by principal: one
/// caller's authorization stop leaves it for an authorized peer, which
/// executes the same row to Ready.
#[test]
fn unauthorized_caller_leaves_the_row_for_an_authorized_peer_to_ready() {
    use super::super::WorthQueryOutputDemandAdvanceAdmission as Admission;
    let registry = WorthQueryOutputDemandRegistry::default();
    let admit = || {
        registry
            .admit(
                key("unauthorized-caller", 1, 1),
                None,
                crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(root(1)),
                occurrence(),
                super::super::DemandAdmissionKind::Ordinary,
                None,
                None,
                &mut record_admission(),
            )
            .expect("the registry creates and joins actual retained membership")
    };
    let unauthorized = admit();
    let authorized = admit();
    assert!(matches!(
        registry.begin(&unauthorized),
        Admission::Schedule(None)
    ));
    registry.finish_scheduling(
        &unauthorized,
        None,
        &mut Ok(WorthQueryOutputSchedulingResult::Scheduled),
    );
    let finish = registry
        .prepare_selected_execution_finish(&unauthorized, &mut record_admission())
        .expect("the unauthorized caller prepares its finish before Running");
    assert!(matches!(
        registry.begin(&unauthorized),
        Admission::Execute { successor_of: None }
    ));
    let mut denial = WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::RequestAuthorization(
            crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenialKind::PermissionDenied,
        ),
        "",
    );
    finish.failure(&mut denial);
    assert_eq!(
        denial.recovery_posture(),
        crate::domain_computation::primary_graph::WorthQueryOutputDemandRecoveryPosture::Terminal,
        "the unauthorized caller keeps its terminal denial"
    );

    let finish = registry
        .prepare_selected_execution_finish(&authorized, &mut record_admission())
        .expect("the authorized peer prepares its finish on the rescheduled row");
    assert!(matches!(
        registry.begin(&authorized),
        Admission::Execute { successor_of: None }
    ));
    let receipt = crate::domain_computation::primary_graph::tests::recoverable_commit_support::committed_recoverable_application();
    let completion = super::super::WorthQueryCompletedOutputDemand {
        authority: super::super::WorthQueryAcceptedOutputAuthority::Committed(receipt),
        readiness: crate::domain_computation::primary_graph::application_output_demand::WorthQueryOutputReadinessDeliveryEvidence::for_test(),
        resources: None,
    };
    if finish
        .publish(super::super::WorthQueryOutputCheckpoint::Ready(
            super::super::ReadyCompletion::for_test(completion),
        ))
        .is_err()
    {
        panic!("the authorized peer publishes the row it executed");
    }
    assert!(matches!(registry.begin(&authorized), Admission::Ready(_)));
}
