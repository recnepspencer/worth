use super::fixture::real_fixture;
use crate::inspection::RuntimeWorldRetentionKey;
use crate::retention::registry::RuntimeWorldRetentionOwner;
use crate::retention::{ComponentBasisDependencyClass, ExactComponentPinRequest};

#[test]
fn equal_basis_descriptions_reclaim_the_first_issued_admission() {
    let fixture = real_fixture(4, 4);
    let relational = fixture.basis.relational_basis().clone();
    let correspondence = fixture.basis.correspondence_basis().clone();
    let reference = fixture
        .signal_port
        .issue_managed_branch_reference(fixture.basis.signal_basis())
        .unwrap();
    let encoding = fixture
        .basis
        .signal_basis()
        .observation()
        .canonical_encoding();
    let first = RuntimeWorldRetentionKey::signal(&fixture.basis);
    drop(
        fixture
            .owner
            .issue_component(ExactComponentPinRequest::signal(
                &fixture.basis,
                ComponentBasisDependencyClass::ActivePublicationAttempt,
            ))
            .unwrap(),
    );
    // The weak Signal admission registry can now issue a new exact token for
    // the same observation. World retains its description and lease position.
    drop(fixture.basis);
    let signal = fixture.signal_port.observe_current(&reference).unwrap();
    assert_eq!(signal.observation().canonical_encoding(), encoding);
    let basis = crate::basis::admit_current(
        fixture.identities.issuer(),
        &fixture
            .relational_runtime
            .owner_component_services()
            .basis_port(),
        &fixture.signal_port,
        &fixture.bridge.runtime_world_correspondence_port(),
        relational,
        signal,
        correspondence,
    )
    .unwrap();
    let second = RuntimeWorldRetentionKey::signal(&basis);
    assert_ne!(
        first, second,
        "equal descriptions have distinct admission tokens"
    );
    drop(
        fixture
            .owner
            .issue_component(ExactComponentPinRequest::signal(
                &basis,
                ComponentBasisDependencyClass::ActivePublicationAttempt,
            ))
            .unwrap(),
    );
    let report = fixture.owner.reclaim(1);
    assert_eq!(report.examined(), 1);
    assert_eq!(report.reclaimed(), 1);
    assert!(fixture.owner.inspect_key(&first).unwrap().is_none());
    assert!(fixture.owner.inspect_key(&second).unwrap().is_some());
}

#[test]
fn equal_contents_reclaim_the_declared_component_in_both_insertion_orders() {
    let fixture = real_fixture(4, 4);
    let relational = RuntimeWorldRetentionKey::relational(&fixture.basis);
    let signal = RuntimeWorldRetentionKey::signal(&fixture.basis);
    // Reversed insertion preserves the
    // declared Relational-before-Signal component order.
    for reversed in [false, true] {
        let owner = RuntimeWorldRetentionOwner::new(
            fixture.owner_identity,
            fixture
                .relational_runtime
                .owner_component_services()
                .basis_port(),
            fixture.signal_port.clone(),
            fixture.budgets.unique_exact_component_pins(),
            fixture.budgets.in_flight_pin_acquisition_reservations(),
            fixture.budgets.active_observations(),
        );
        let requests = [
            ExactComponentPinRequest::relational(
                &fixture.basis,
                ComponentBasisDependencyClass::ActivePublicationAttempt,
            ),
            ExactComponentPinRequest::signal(
                &fixture.basis,
                ComponentBasisDependencyClass::ActivePublicationAttempt,
            ),
        ];
        let order = if reversed { [1, 0] } else { [0, 1] };
        for index in order {
            drop(owner.issue_component(requests[index]).unwrap());
        }
        let report = owner.reclaim(1);
        assert_eq!(report.examined(), 1);
        assert_eq!(report.reclaimed(), 1);
        assert!(owner.inspect_key(&relational).unwrap().is_none());
        assert!(owner.inspect_key(&signal).unwrap().is_some());
    }
}

#[test]
fn reclamation_orders_two_signal_bases_inside_the_same_component_variant() {
    let mut fixture = real_fixture(4, 4);
    let services = fixture._signal_runtime.owner_component_services().unwrap();
    let mutation = services.mutation_port();
    let fork = |name| {
        mutation
            .fork_exact(
                worth_signal::facade::branch::validate_signal_branch_name(name).unwrap(),
                fixture.basis.signal_basis(),
                &worth_signal::facade::branch::SignalOwnerCancellationSource::new().token(),
            )
            .unwrap()
    };
    let children = [fork("zulu"), fork("alpha")];
    let descriptions = children
        .each_ref()
        .map(|child| child.created_basis().observation().canonical_encoding());
    assert_ne!(descriptions[0], descriptions[1]);
    let declared = if descriptions[0] < descriptions[1] {
        [0, 1]
    } else {
        [1, 0]
    };
    let correspondence = fixture.bridge.runtime_world_correspondence_port();
    let bases = children.each_ref().map(|child| {
        crate::basis::admit_current(
            fixture.identities.issuer(),
            &fixture
                .relational_runtime
                .owner_component_services()
                .basis_port(),
            &fixture.signal_port,
            &correspondence,
            fixture.basis.relational_basis().clone(),
            child.created_basis().clone(),
            fixture.basis.correspondence_basis().clone(),
        )
        .unwrap()
    });
    for index in declared.into_iter().rev() {
        drop(
            fixture
                .owner
                .issue_component(ExactComponentPinRequest::signal(
                    &bases[index],
                    ComponentBasisDependencyClass::ActivePublicationAttempt,
                ))
                .unwrap(),
        );
    }
    let report = fixture.owner.reclaim(1);
    assert_eq!(report.examined(), 1);
    assert_eq!(report.reclaimed(), 1);
    assert!(fixture
        .owner
        .inspect_key(&RuntimeWorldRetentionKey::signal(&bases[declared[0]]))
        .unwrap()
        .is_none());
    assert!(fixture
        .owner
        .inspect_key(&RuntimeWorldRetentionKey::signal(&bases[declared[1]]))
        .unwrap()
        .is_some());
}
