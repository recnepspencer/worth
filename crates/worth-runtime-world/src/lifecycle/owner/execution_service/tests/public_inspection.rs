use super::*;
use crate::facade::{
    ComponentBasisDependencyClass, PerformedPublicationRecoveryDenial,
    RuntimeWorldPublicationOutcome, RuntimeWorldRetentionKey, RuntimeWorldServiceDenial,
};
#[test]
fn public_performed_recovery_is_exclusive_and_history_has_real_pin_counts() {
    let (fixture, owner, expected) = super::public_ports::public_world();
    let inspection = owner.inspection_port();
    let key = RuntimeWorldRetentionKey::relational(expected.basis());
    let entry = inspection.inspect_retention(&key).unwrap().unwrap();
    assert_eq!(
        entry
            .dependencies()
            .get(ComponentBasisDependencyClass::RetainedCompositeHistory),
        1
    );
    assert!(entry.owner_lease_present());
    assert_eq!(
        owner
            .lifecycle_port()
            .reclaim_retention(&[key], 1)
            .unwrap()
            .reclaimed(),
        0
    );
    let cancellation = RuntimeWorldCancellationSource::new();
    let prepared = owner
        .publication_port()
        .prepare_without_signal(
            expected,
            CompositePublicationIntent::without_signal(RelationalTransactionIntent::ordinary())
                .with_prepared_relational_candidate(
                    fixture.prepare_relational_owner_candidate("inspection"),
                ),
            &cancellation.token(),
            None,
        )
        .unwrap();
    let performed = match owner
        .publication_port()
        .execute_without_signal(prepared, &cancellation.token())
    {
        RuntimeWorldPublicationOutcome::Performed(value) => value,
        other => panic!("{other:?}"),
    };
    let commit = performed.commit().identity().clone();
    let owner_result = performed
        .component_results()
        .relational_commit_result()
        .expect("canonical Relational result survives public handoff");
    assert_eq!(
        performed
            .component_results()
            .relational_publication_identity()
            .unwrap()
            .commit_id(),
        owner_result.outcome().commit.commit_id
    );
    assert!(performed
        .component_results()
        .relational_settlement()
        .is_some());
    assert_eq!(performed.cost_counters().history_slots_reserved(), 1);
    assert!(matches!(
        owner.recovery_port().recover_performed(&commit),
        Err(RuntimeWorldServiceDenial::Denied(
            PerformedPublicationRecoveryDenial::Claimed
        ))
    ));
    drop(performed);
    let recovered = owner.recovery_port().recover_performed(&commit).unwrap();
    assert_eq!(recovered.cost_counters().cas_wins(), 1);
    drop(recovered.consume());
    assert!(matches!(
        owner.recovery_port().recover_performed(&commit),
        Err(RuntimeWorldServiceDenial::Denied(
            PerformedPublicationRecoveryDenial::Consumed
        ))
    ));
    assert_eq!(
        inspection.history_snapshot().unwrap().installed_commits(),
        2
    );
    let upgraded = owner.root.clone();
    drop(owner);
    assert!(inspection.history_snapshot().is_err());
    use crate::lifecycle::{RuntimeWorldLifecycleService, RuntimeWorldRecoveryService};
    assert!(matches!(
        upgraded.recover_performed(&commit),
        Err(PerformedPublicationRecoveryDenial::OwnerUnavailable(_))
    ));
    let request = crate::history::CompositeHistoryReclamationRequest::new(
        commit.owner_identity(),
        vec![commit],
        1,
    );
    assert!(matches!(
        upgraded.reclaim_history(request),
        Err(crate::history::HistoryReclamationDenial::OwnerUnavailable(
            _
        ))
    ));
    assert!(matches!(
        upgraded.reclaim_retention(&[], 1),
        Err(crate::inspection::RuntimeWorldRetentionInspectionDenial::OwnerUnavailable(_))
    ));
}
#[test]
fn recovery_pages_bound_active_work_and_age_starts_at_admission() {
    let mut fixture = reference_test_fixture::real_fixture(12, 12);
    let clock = MutableClock::new(10);
    let inputs = fixture.owner_inputs(budgets(4), RuntimeWorldClock::from_source(clock.clone()));
    let owner = TestOwner::new(inputs).unwrap();
    let expected = match owner.bootstrap_root(fixture.bootstrap_intent()) {
        RuntimeWorldBootstrapOutcome::Performed(value) => value.product_branch().clone(),
        other => panic!("{other:?}"),
    };
    let cancellation = RuntimeWorldCancellationSource::new();
    let prepare = || {
        owner
            .prepare_publication(
                expected.clone(),
                CompositePublicationIntent::with_signal(None),
                &cancellation.token(),
                None,
            )
            .unwrap()
    };
    let first = prepare();
    let second = prepare();
    let third = prepare();
    clock.set(17);
    let page = owner
        .state
        .recovery
        .page(None, std::num::NonZeroUsize::new(1).unwrap(), clock.now())
        .unwrap();
    assert_eq!(page.examined(), 1);
    assert_eq!(
        page.rows()[0].state(),
        crate::inspection::RuntimeWorldRecoveryRecordState::Active
    );
    assert_eq!(page.rows()[0].age_ticks(), Some(7));
    drop(first);
    drop(second);
    let page2 = owner
        .state
        .recovery
        .page(
            page.next_after(),
            std::num::NonZeroUsize::new(1).unwrap(),
            clock.now(),
        )
        .unwrap();
    assert_eq!(page2.examined(), 1);
    assert!(
        page2.rows().is_empty(),
        "a removed row still spends its slot examination"
    );
    let page3 = owner
        .state
        .recovery
        .page(
            page2.next_after(),
            std::num::NonZeroUsize::new(1).unwrap(),
            clock.now(),
        )
        .unwrap();
    assert_eq!(page3.examined(), 1);
    assert!(page3.next_after().is_none());
    assert_eq!(page3.rows().len(), 1);
    assert_ne!(
        page.rows()[0].handle().identity(),
        page3.rows()[0].handle().identity()
    );
    let foreign = TestOwner::new(
        fixture.owner_inputs(budgets(4), RuntimeWorldClock::from_source(clock.clone())),
    )
    .unwrap();
    assert!(matches!(
        foreign.state.recovery.page(
            page.next_after(),
            std::num::NonZeroUsize::new(1).unwrap(),
            clock.now()
        ),
        Err(crate::recovery::RuntimeWorldRecoveryDenial::ForeignHandle)
    ));
    for _ in 0..64 {
        drop(prepare());
    }
    let after_churn = owner
        .state
        .recovery
        .page(None, std::num::NonZeroUsize::new(128).unwrap(), clock.now())
        .unwrap();
    assert_eq!(
        after_churn.examined(),
        3,
        "free slots are reused instead of retaining lifetime churn"
    );
    assert_eq!(after_churn.rows().len(), 1);
    drop(third);
}

#[test]
fn retained_age_is_carried_and_cleanup_uses_the_explicit_minimum() {
    let mut fixture = reference_test_fixture::real_fixture(12, 12);
    let clock = MutableClock::new(10);
    let inputs = fixture.owner_inputs(budgets(4), RuntimeWorldClock::from_source(clock.clone()));
    let owner = TestOwner::new(inputs).unwrap();
    let expected = match owner.bootstrap_root(fixture.bootstrap_intent()) {
        RuntimeWorldBootstrapOutcome::Performed(value) => value.product_branch().clone(),
        other => panic!("{other:?}"),
    };
    let cancel = RuntimeWorldCancellationSource::new();
    let prepared = owner
        .prepare_publication(
            expected,
            CompositePublicationIntent::with_signal(Some(RelationalTransactionIntent::ordinary()))
                .with_prepared_relational_candidate(
                    fixture.prepare_relational_owner_candidate("aged-effects"),
                ),
            &cancel.token(),
            None,
        )
        .unwrap();
    clock.set(13);
    let effects = match owner.execute_with_signal(prepared, &mut (), &cancel.token(), |_| {
        cancel.cancel();
        Ok(())
    }) {
        OwnerExecutionOutcome::ProductUnpublished(value) => value,
        other => panic!("{other:?}"),
    };
    let handle = effects.recovery_handle();
    drop(effects);
    let page = owner
        .state
        .recovery
        .page(None, std::num::NonZeroUsize::new(1).unwrap(), clock.now())
        .unwrap();
    assert_eq!(page.rows()[0].age_ticks(), Some(3));
    assert_eq!(
        page.rows()[0].state(),
        crate::inspection::RuntimeWorldRecoveryRecordState::Retained
    );
    use crate::lifecycle::RuntimeWorldRecoveryService;
    assert_eq!(
        owner.release_effects(&handle, 4).unwrap_err(),
        crate::recovery::RuntimeWorldRecoveryDenial::TooYoung
    );
    clock.set(14);
    owner.release_effects(&handle, 4).unwrap();
    let snapshot = owner.state.recovery.snapshot();
    assert_eq!(snapshot.installed(), 0);
    assert_eq!(snapshot.costs().retained_records_created(), 1);
    assert_eq!(snapshot.costs().records_cleaned(), 1);
}
