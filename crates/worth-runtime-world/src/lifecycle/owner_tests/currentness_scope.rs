//! Actual observation-port admission, movement, and lifecycle custody.

use std::sync::Arc;

use crate::branch::{ProductBranchCurrentnessFailure, RuntimeWorldBranchAdmissionDenial};
use crate::lifecycle::{
    RuntimeWorldCloseDenial, RuntimeWorldObservationPort, RuntimeWorldObservationService,
    RuntimeWorldServiceDenial,
};
use crate::publication::{CompositeLateCancellationPosture, RuntimeWorldPublicationOutcome};

use super::publication::{ready_relational_publication, setup, TestOwner};

fn port(owner: &Arc<TestOwner>) -> RuntimeWorldObservationPort {
    let service: Arc<dyn RuntimeWorldObservationService + Send + Sync> = owner.clone();
    RuntimeWorldObservationPort::new(Arc::downgrade(&service))
}

#[test]
fn currentness_scope_holds_operation_and_service_through_exact_head_callback() {
    let (_fixture, owner, expected) = setup();
    let port = port(&owner);
    let strong = Arc::strong_count(&owner);
    let result = port
        .while_product_branch_current(&expected, 7, |argument, head| {
            assert_eq!(head.observation(), &expected);
            assert_eq!(owner.state.operation.active(), 1);
            assert_eq!(Arc::strong_count(&owner), strong + 1);
            assert!(matches!(
                owner.close(),
                Err(RuntimeWorldCloseDenial::AlreadyClosing)
            ));
            argument + 1
        })
        .expect("the exact current head enters its section");
    assert_eq!(result, 8);
    assert_eq!(owner.state.operation.active(), 0);
    assert_eq!(Arc::strong_count(&owner), strong);
}

#[test]
fn admitted_currentness_refuses_before_registry_lookup_and_keeps_custody() {
    let (_fixture, owner, expected) = setup();
    let port = port(&owner);
    let custody = Box::new(41);
    let address = &*custody as *const i32;
    let mut claims = Vec::new();
    let stopped = port
        .while_product_branch_current_admitted(
            &expected,
            custody,
            &mut |work| {
                claims.push(work);
                claims.len() == 1
            },
            |_, _| panic!("a refused lookup cannot enter the current-head callback"),
        )
        .unwrap_err();
    let ProductBranchCurrentnessFailure::PreparationDenied(custody) = stopped else {
        panic!("the second before-read claim is the admission refusal")
    };
    assert_eq!(&*custody as *const i32, address);
    assert_eq!(*custody, 41);
    assert_eq!(claims.len(), 2);
    assert_eq!(owner.state.operation.active(), 0);
    assert_eq!(
        port.observe_product_branch(expected.branch_identity())
            .unwrap(),
        expected
    );
}

#[test]
fn currentness_scope_returns_unmodified_custody_after_actual_publication() {
    let (fixture, owner, expected) = setup();
    let port = port(&owner);
    let ready = ready_relational_publication(&fixture, &owner, expected.clone());
    let cell = owner.state.branches.root_cell().unwrap();
    assert!(matches!(
        ready.publish(&cell, CompositeLateCancellationPosture::NotRequested),
        RuntimeWorldPublicationOutcome::Performed(_)
    ));
    let custody = Box::new(42);
    let address = &*custody as *const i32;
    let stopped = port
        .while_product_branch_current(&expected, custody, |_, _| {
            panic!("a displaced head never enters the callback")
        })
        .unwrap_err();
    let ProductBranchCurrentnessFailure::ExpectedHeadUnavailable(custody) = stopped else {
        panic!("the owner remains open; exact head movement is the denial")
    };
    assert_eq!(&*custody as *const i32, address);
    assert_eq!(*custody, 42);
    assert_eq!(owner.state.operation.active(), 0);
}

#[test]
fn currentness_scope_denies_foreign_and_closed_owner_before_callback() {
    let (_fixture, owner, expected) = setup();
    let (_foreign_fixture, foreign, _) = setup();
    let foreign_port = port(&foreign);
    assert!(matches!(
        foreign_port.while_product_branch_current(&expected, 3, |_, _| panic!("foreign head")),
        Err(ProductBranchCurrentnessFailure::AdmissionDenied {
            denial: RuntimeWorldServiceDenial::Denied(
                RuntimeWorldBranchAdmissionDenial::ForeignOwner
            ),
            argument: 3,
        })
    ));
    let own_port = port(&owner);
    let report = owner.close().expect("no scope is active");
    assert!(report.retained_records().is_empty());
    assert!(matches!(
        own_port.while_product_branch_current(&expected, 4, |_, _| panic!("closed owner")),
        Err(ProductBranchCurrentnessFailure::AdmissionDenied {
            denial: RuntimeWorldServiceDenial::OwnerUnavailable(_),
            argument: 4,
        })
    ));
}

#[cfg(feature = "test-operation-control")]
#[test]
fn currentness_control_observes_real_reference_contention_without_head_movement() {
    use std::time::Duration;

    let (_fixture, owner, expected) = setup();
    let port = port(&owner);
    let hold = owner
        .state
        .operation_control
        .pause_product_currentness(expected.lifecycle_incarnation());
    std::thread::scope(|threads| {
        let reader = threads.spawn(|| {
            port.while_product_branch_current(&expected, 7, |value, head| {
                assert_eq!(head.observation(), &expected);
                value + 1
            })
        });
        assert!(hold.wait_until_reader_contended(Duration::from_secs(5)));
        hold.release();
        assert_eq!(reader.join().unwrap().ok(), Some(8));
    });
    drop(hold);
    assert_eq!(owner.state.operation.active(), 0);
    assert_eq!(
        port.observe_product_branch(expected.branch_identity())
            .unwrap(),
        expected
    );
}
