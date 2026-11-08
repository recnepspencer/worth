//! An original effect keeps its first-send and recovery custody after failed cleanup.
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::{AuthorizationWorld, installed_authorization_world, live_scope};
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::{authenticated_principal, resolved_account, idempotency};
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::preimage_evidence::{retained_narrow_status_program_with_binding};
use worth_query_execution::facade::primary_graph::*;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationManagedCleanupReceipt as Cleanup,
    WorthQueryCleanupDispatchDenial as DispatchDenial, WorthQueryCleanupDispatchPosture as Posture,
    WorthQueryRecoveredApplicationCommitReceipt,
};
use worth_query_execution::facade::primary_graph::{
    WorthQueryExternalDispatchRequest, WorthQueryExternalEffectTransport,
    WorthQueryExternalTransportOutcome,
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
fn committed_with_failed_cleanup(
    world: &AuthorizationWorld,
) -> (Cleanup, WorthQueryApplicationIdempotencyBinding) {
    let request = live_scope();
    let (program, binding) = world
        .application
        .with_host_advancement(|phase| {
            let custody = phase.test_host_request_for_test();
            let principal = authenticated_principal(&custody, world, &request);
            let account = resolved_account(&custody, world, "open", &request);
            retained_narrow_status_program_with_binding(
                &custody,
                world,
                &principal,
                &account,
                &request,
                "cleanup-first-send",
                idempotency(207, 208),
            )
        })
        .unwrap();
    let worker = world
        .application
        .schedule_next_managed_cleanup_on_worker_for_test()
        .unwrap();
    let outcome = world.application.compare_and_commit_application(
        program,
        binding,
        worth_query_execution::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
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
    (receipt, binding)
}
fn original(
    world: &AuthorizationWorld,
    binding: WorthQueryApplicationIdempotencyBinding,
) -> WorthQueryRecoveredApplicationCommitReceipt {
    world.application.with_host_advancement(|phase| {
        let selected = world.selected_product(&phase.test_host_request_for_test());
        match world.application.runtime_for_test().primary_provider_for_test().resolve_idempotency_binding_at_product_for_test(
            &phase.borrowed_request_for_test(), binding, &selected.product().publication_binding_for_test()).unwrap() {
            worth_query_execution::court_doors_for_test::domain_computation::primary_graph::provider::idempotency::WorthQueryProviderIdempotencyResolution::Readmitted(receipt) => receipt,
            other => panic!("the actual durable owner reads the original commit: {other:?}"),
        }
    }).unwrap()
}
fn setup(
    response: Response,
) -> (
    AuthorizationWorld,
    Arc<Transport>,
    Cleanup,
    WorthQueryApplicationIdempotencyBinding,
) {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let transport = Arc::new(Transport {
        contacts: AtomicUsize::new(0),
        response,
    });
    world
        .application
        .install_external_effect_transport(transport.clone())
        .unwrap();
    let (cleanup, binding) = committed_with_failed_cleanup(&world);
    assert_eq!(transport.contacts.load(Ordering::SeqCst), 0);
    (world, transport, cleanup, binding)
}
#[test]
fn failed_cleanup_keeps_the_original_first_send_after_derived_windows_are_discarded() {
    let (world, transport, mut cleanup, binding) = setup(Response::Completed);
    let recorded = original(&world, binding);
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
        .runtime_for_test()
        .primary_provider_for_test()
        .discard_completed_evidence_for_test();
    world
        .application
        .runtime_for_test()
        .primary_provider_for_test()
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
    assert_eq!(original(&world, binding), recorded);
    assert_eq!(transport.contacts.load(Ordering::SeqCst), 1);
}
#[test]
fn lost_original_dispatch_retains_its_exact_recovery_binding_without_another_first_send() {
    let (world, transport, mut cleanup, binding) = setup(Response::Lost);
    let recorded = original(&world, binding);
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
    let (world, transport, mut cleanup, binding) = setup(Response::Panic);
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
        original(&world, binding).commit_reference()
    );
    assert_eq!(transport.contacts.load(Ordering::SeqCst), 1);
}
