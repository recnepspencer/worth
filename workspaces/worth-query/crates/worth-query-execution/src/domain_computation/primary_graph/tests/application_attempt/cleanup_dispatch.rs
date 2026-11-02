//! An original effect keeps its first-send and recovery custody after failed cleanup.
use super::super::fixture::AuthorizationWorld;
use super::preimage_evidence::{retained_status_program, RetentionMutationBreadth};
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationManagedCleanupReceipt as Cleanup,
    WorthQueryCleanupDispatchDenial as DispatchDenial, WorthQueryCleanupDispatchPosture as Posture,
    WorthQueryExternalDispatchRequest, WorthQueryExternalEffectTransport,
    WorthQueryExternalTransportOutcome, WorthQueryRecoveredApplicationCommitReceipt,
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

#[derive(Clone, Copy)]
enum Response {
    Completed,
    Lost,
    Panic,
}
struct Transport {
    contacts: AtomicUsize,
    response: Response,
}
impl WorthQueryExternalEffectTransport for Transport {
    fn dispatch(
        &self,
        _: WorthQueryExternalDispatchRequest<'_>,
    ) -> WorthQueryExternalTransportOutcome {
        self.contacts.fetch_add(1, Ordering::SeqCst);
        match self.response {
            Response::Completed => WorthQueryExternalTransportOutcome::Completed,
            Response::Lost => WorthQueryExternalTransportOutcome::LostResponse,
            Response::Panic => panic!("transport unwound after accepting the original bytes"),
        }
    }
}
fn committed_with_failed_cleanup(world: &AuthorizationWorld) -> Cleanup {
    let request = live_scope();
    let program = world
        .application
        .with_host_advancement(|phase| {
            let custody = phase.test_host_request();
            let principal = authenticated_principal(&custody, world, &request);
            let account = resolved_account(&custody, world, "open", &request);
            retained_status_program(
                &custody,
                world,
                &principal,
                &account,
                &request,
                "cleanup-first-send",
                RetentionMutationBreadth::Narrow,
            )
        })
        .unwrap();
    let worker = world
        .application
        .schedule_next_managed_cleanup_on_worker_for_test()
        .unwrap();
    let outcome = world.application.compare_and_commit_application(
        program,
        idempotency(207, 208),
        crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
    );
    drop(worker);
    let WorthQueryApplicationCommitOutcome::CleanupRequired(required) = outcome else {
        panic!("actual commitment followed by failed managed cleanup must preserve both owners: {outcome:?}");
    };
    let (business, cleanup) = required.into_parts();
    assert!(matches!(
        business,
        WorthQueryApplicationCommitOutcome::Indeterminate(_)
    ));
    let scope = live_scope();
    let receipt = world
        .application
        .with_application_advancement(&scope, |phase| cleanup.retry_in_advancement(&phase))
        .unwrap()
        .unwrap();
    assert!(receipt.inspection().resources_released());
    assert_eq!(receipt.dispatch_posture(), Posture::Pending);
    receipt
}
fn original(
    world: &AuthorizationWorld,
    cleanup: &Cleanup,
) -> WorthQueryRecoveredApplicationCommitReceipt {
    let binding = cleanup
        .original
        .as_ref()
        .unwrap()
        .original_authority()
        .idempotency_binding();
    world.application.with_host_advancement(|phase| {
        let selected = world.selected_product(&phase.test_host_request());
        match world.application.primary_provider.resolve_idempotency_binding_at_product(
            &phase.borrowed_request_for_test(), binding, &selected.product().publication_binding()).unwrap() {
            crate::domain_computation::primary_graph::provider::WorthQueryProviderIdempotencyResolution::Readmitted(receipt) => receipt,
            other => panic!("the actual durable owner reads the original commit: {other:?}"),
        }
    }).unwrap()
}
fn setup(response: Response) -> (AuthorizationWorld, Arc<Transport>, Cleanup) {
    let world = installed_authorization_world(true);
    let transport = Arc::new(Transport {
        contacts: AtomicUsize::new(0),
        response,
    });
    world
        .application
        .install_external_effect_transport(transport.clone())
        .unwrap();
    let cleanup = committed_with_failed_cleanup(&world);
    assert_eq!(transport.contacts.load(Ordering::SeqCst), 0);
    (world, transport, cleanup)
}
#[test]
fn failed_cleanup_keeps_the_original_first_send_after_derived_windows_are_discarded() {
    let (world, transport, mut cleanup) = setup(Response::Completed);
    let recorded = original(&world, &cleanup);
    let mut copy = cleanup.clone();
    let request = live_scope();
    world
        .application
        .with_application_advancement(&request, |phase| {
            assert!(matches!(
                world
                    .application
                    .resume_cleanup_original_dispatch(&phase, &mut copy, &request),
                Err(DispatchDenial::OriginalNotOwned)
            ));
        })
        .unwrap();
    assert!(world
        .application
        .mint_cleanup_recovery_handle(&copy)
        .is_err());
    assert_eq!(transport.contacts.load(Ordering::SeqCst), 0);
    world
        .application
        .primary_provider
        .discard_completed_evidence_for_test();
    world
        .application
        .primary_provider
        .discard_receipt_basis_window_for_test();
    world
        .application
        .with_application_advancement(&request, |phase| {
            assert!(world
                .application
                .resume_cleanup_original_dispatch(&phase, &mut cleanup, &request)
                .unwrap()
                .is_some());
            assert!(matches!(
                world
                    .application
                    .resume_cleanup_original_dispatch(&phase, &mut cleanup, &request),
                Err(DispatchDenial::AlreadyAttempted)
            ));
        })
        .unwrap();
    assert_eq!(transport.contacts.load(Ordering::SeqCst), 1);
    assert_eq!(cleanup.dispatch_posture(), Posture::Observed);
    assert_eq!(original(&world, &cleanup), recorded);
    assert_eq!(transport.contacts.load(Ordering::SeqCst), 1);
}
#[test]
fn lost_original_dispatch_retains_its_exact_recovery_binding_without_another_first_send() {
    let (world, transport, mut cleanup) = setup(Response::Lost);
    let recorded = original(&world, &cleanup);
    let request = live_scope();
    world
        .application
        .with_application_advancement(&request, |phase| {
            world
                .application
                .resume_cleanup_original_dispatch(&phase, &mut cleanup, &request)
                .unwrap();
            assert!(matches!(
                world
                    .application
                    .resume_cleanup_original_dispatch(&phase, &mut cleanup, &request),
                Err(DispatchDenial::AlreadyAttempted)
            ));
        })
        .unwrap();
    let handle = world
        .application
        .mint_cleanup_recovery_handle(&cleanup)
        .unwrap();
    assert_eq!(
        handle.binding().commit_reference(),
        recorded.commit_reference()
    );
    assert!(handle.binding().provider_posture().is_some());
    assert_eq!(transport.contacts.load(Ordering::SeqCst), 1);
}
#[test]
fn transport_unwind_cannot_restore_original_first_send_eligibility() {
    let (world, transport, mut cleanup) = setup(Response::Panic);
    let request = live_scope();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        world
            .application
            .with_application_advancement(&request, |phase| {
                world
                    .application
                    .resume_cleanup_original_dispatch(&phase, &mut cleanup, &request)
                    .map(|_| ())
            })
    }));
    assert!(result.is_err() || result.unwrap().is_err());
    assert_eq!(cleanup.dispatch_posture(), Posture::AttemptedUnresolved);
    assert_eq!(transport.contacts.load(Ordering::SeqCst), 1);
    let request = live_scope();
    world
        .application
        .with_application_advancement(&request, |phase| {
            assert!(matches!(
                world
                    .application
                    .resume_cleanup_original_dispatch(&phase, &mut cleanup, &request),
                Err(DispatchDenial::AlreadyAttempted)
            ));
        })
        .unwrap();
    let handle = world
        .application
        .mint_cleanup_recovery_handle(&cleanup)
        .unwrap();
    assert_eq!(
        handle.binding().commit_reference(),
        original(&world, &cleanup).commit_reference()
    );
    assert_eq!(transport.contacts.load(Ordering::SeqCst), 1);
}
