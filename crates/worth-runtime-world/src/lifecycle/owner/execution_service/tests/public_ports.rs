use super::*;
use crate::facade::{NoEffectCause, RuntimeWorldOwner, RuntimeWorldPublicationOutcome};

pub(super) fn public_world() -> (
    RealReferenceFixture,
    RuntimeWorldOwner<(), (), (), (), ()>,
    ProductBranchObservation,
) {
    let mut fixture = reference_test_fixture::real_fixture(12, 12);
    let (relational, signal, bridge, budgets, clock) = fixture
        .owner_inputs(budgets(4), RuntimeWorldClock::from_source(FixedClock))
        .into_parts();
    let owner = RuntimeWorldOwner::builder()
        .with_bridge_correspondence(bridge)
        .with_relational_services(relational)
        .with_signal_services(signal)
        .with_budgets(budgets)
        .with_clock(clock)
        .build()
        .unwrap();
    let performed = match owner
        .lifecycle_port()
        .bootstrap_root(fixture.bootstrap_intent())
        .unwrap()
    {
        RuntimeWorldBootstrapOutcome::Performed(performed) => performed,
        other => panic!("public bootstrap: {other:?}"),
    };
    (fixture, owner, performed.product_branch().clone())
}

#[test]
fn public_ports_complete_publication_and_consumption_permanently_closes_delivery() {
    let (fixture, owner, expected) = public_world();
    let port = owner.publication_port();
    let cancellation = RuntimeWorldCancellationSource::new();
    let prepared = port
        .prepare_without_signal(
            expected.clone(),
            CompositePublicationIntent::without_signal(RelationalTransactionIntent::ordinary())
                .with_prepared_relational_candidate(
                    fixture.prepare_relational_owner_candidate("public-relational"),
                ),
            &cancellation.token(),
            None,
        )
        .unwrap();
    let performed = match port.execute_without_signal(prepared, &cancellation.token()) {
        RuntimeWorldPublicationOutcome::Performed(performed) => performed,
        other => panic!("public execution must finish its CAS: {other:?}"),
    };
    let commit = performed.commit().identity().clone();
    let receipt = performed.consume();
    assert_eq!(receipt.old_product_head(), expected.snapshot());
    let next = owner
        .observation_port()
        .observe_product_branch(expected.branch_identity())
        .unwrap();
    assert_eq!(receipt.new_product_head(), next.snapshot());
    assert_eq!(receipt.cost_counters().cas_wins(), 1);
    drop(receipt);
    assert!(owner
        .root
        .recover_performed_publication(&commit)
        .unwrap()
        .is_none());
    let stale = port
        .prepare_with_signal(
            expected,
            CompositePublicationIntent::with_signal(None),
            &cancellation.token(),
            None,
        )
        .unwrap_err();
    assert_eq!(stale.cause(), NoEffectCause::StaleExpectedProductHead);
    let prepared = port
        .prepare_with_signal(
            next,
            CompositePublicationIntent::with_signal(None),
            &cancellation.token(),
            None,
        )
        .unwrap();
    assert!(matches!(
        port.execute_with_signal(prepared, &mut (), &cancellation.token(), |_| Ok(())),
        RuntimeWorldPublicationOutcome::Performed(_)
    ));
}

#[test]
fn public_owner_loss_denies_ports_even_when_an_in_flight_upgrade_keeps_engine_alive() {
    let (fixture, owner, expected) = public_world();
    let engine = Arc::clone(&owner.root);
    let observation = owner.observation_port();
    let publication = owner.publication_port();
    let cancellation = RuntimeWorldCancellationSource::new();
    let prepared = publication
        .prepare_with_signal(
            expected.clone(),
            CompositePublicationIntent::with_signal(None),
            &cancellation.token(),
            None,
        )
        .unwrap();
    drop(owner);
    assert!(observation
        .observe_product_branch(expected.branch_identity())
        .is_err());
    let outcome = publication.execute_with_signal(prepared, &mut (), &cancellation.token(), |_| {
        panic!("lost owner cannot call Signal")
    });
    assert!(
        matches!(outcome,RuntimeWorldPublicationOutcome::NoEffect(no) if no.cause()==NoEffectCause::OwnerUnavailable)
    );
    assert_eq!(engine.state.operation.active(), 0);
    assert_eq!(
        fixture.observe_signal_current_basis().observation(),
        expected.basis().signal_basis().observation()
    );
}

#[test]
fn public_foreign_prepared_token_never_contacts_a_component() {
    let (_fixture, owner, expected) = public_world();
    let (_foreign_fixture, foreign, _) = public_world();
    let cancellation = RuntimeWorldCancellationSource::new();
    let prepared = owner
        .publication_port()
        .prepare_with_signal(
            expected,
            CompositePublicationIntent::with_signal(None),
            &cancellation.token(),
            None,
        )
        .unwrap();
    let outcome = foreign.publication_port().execute_with_signal(
        prepared,
        &mut (),
        &cancellation.token(),
        |_| panic!("foreign token cannot execute"),
    );
    assert!(
        matches!(outcome,RuntimeWorldPublicationOutcome::NoEffect(no) if no.cause()==NoEffectCause::OwnerDeniedBeforeEffect)
    );
    assert_eq!(owner.root.state.operation.active(), 0);
    assert_eq!(foreign.root.state.operation.active(), 0);
}

#[test]
fn public_close_denies_saved_ports_without_closing_components() {
    let (fixture, owner, expected) = public_world();
    let observation = owner.observation_port();
    let report = owner.lifecycle_port().close().unwrap();
    drop(report);
    assert!(observation
        .observe_product_branch(expected.branch_identity())
        .is_err());
    assert_eq!(
        fixture.observe_signal_current_basis().observation(),
        expected.basis().signal_basis().observation()
    );
}

#[test]
fn public_partial_recovery_preserves_foreign_and_live_capability_denials() {
    let (fixture, owner, expected) = public_world();
    let (_foreign_fixture, foreign, _) = public_world();
    let cancellation = RuntimeWorldCancellationSource::new();
    let prepared = owner
        .publication_port()
        .prepare_with_signal(
            expected,
            CompositePublicationIntent::with_signal(Some(RelationalTransactionIntent::ordinary()))
                .with_prepared_relational_candidate(
                    fixture.prepare_relational_owner_candidate("public-partial"),
                ),
            &cancellation.token(),
            None,
        )
        .unwrap();
    let record = match owner.publication_port().execute_with_signal(
        prepared,
        &mut (),
        &cancellation.token(),
        |_| {
            cancellation.cancel();
            Ok(())
        },
    ) {
        RuntimeWorldPublicationOutcome::ProductUnpublished(record) => record,
        other => panic!("Relational moved before Signal cancellation: {other:?}"),
    };
    let handle = record.recovery_handle();
    assert!(matches!(
        foreign.recovery_port().inspect_effects(&handle),
        Err(crate::facade::RuntimeWorldRecoveryDenial::ForeignHandle)
    ));
    assert!(matches!(
        owner.recovery_port().release_effects(&handle, 0),
        Err(crate::facade::RuntimeWorldRecoveryDenial::CallerCapabilityLive)
    ));
    drop(record);
    owner.recovery_port().release_effects(&handle, 0).unwrap();
    assert!(matches!(
        owner.recovery_port().inspect_effects(&handle),
        Err(crate::facade::RuntimeWorldRecoveryDenial::MissingRecord)
    ));
}
