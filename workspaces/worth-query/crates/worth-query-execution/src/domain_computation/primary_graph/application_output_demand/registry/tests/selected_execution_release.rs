use super::*;

#[test]
fn selected_release_refuses_before_running_then_releases_without_new_allowance() {
    let registry = WorthQueryOutputDemandRegistry::default();
    let demand_key = key("selected-request-rejection", 1, 1);
    let scope = crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(root(1));
    let admit = || {
        registry
            .admit(
                demand_key.clone(),
                None,
                scope,
                occurrence(),
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
}
