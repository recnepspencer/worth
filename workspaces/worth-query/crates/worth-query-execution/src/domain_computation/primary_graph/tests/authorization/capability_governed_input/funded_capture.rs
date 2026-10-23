//! Real native capture and capability carry share one final-owner payload reservation.
use super::*;
use crate::domain_computation::primary_graph::tests::fixture::{
    CapabilityTouchInput, TouchAccountCapability,
};
use crate::publication_boundary::program_publication_access;
use worth_query_declaration::facade::application_operation::{
    ApplicationEncodedInput, ApplicationMutationBinding,
};

#[test]
fn native_capture_allocation_preserves_actual_cancellation_and_deadline_stops() {
    use crate::domain_computation::primary_graph::{
        WorthQueryAdvancementDenial as Denial, WorthQueryManagedComputationInterruption as Stop,
    };
    use worth_query_admission::facade::authenticated_principal::{
        WorthQueryCancellationSource, WorthQueryRequestScope,
    };
    let (world, _) = super::capability_world();
    let opening = live_scope();
    let cancellation = WorthQueryCancellationSource::new();
    cancellation.cancel();
    let expired = WorthQueryCancellationSource::new();
    let stopped = [
        (
            WorthQueryRequestScope::new(None, cancellation.token()),
            Stop::Cancelled,
        ),
        (
            WorthQueryRequestScope::new(Some(std::time::Instant::now()), expired.token()),
            Stop::DeadlineExceeded,
        ),
    ];
    type InputBinding = <CapabilityTouchMutationBinding as ApplicationMutationBinding<
        IdentityExecutionSchema,
    >>::InputBinding;
    world.application.with_application_advancement(&opening, |phase| {
        for (request, expected) in &stopped {
            let cause = world.application.capture_operation_input_in_advancement::<InputBinding>(
                &phase, &program_publication_access(), request, &capability_input(100),
            ).unwrap_err();
            assert!(matches!(cause,
                crate::publication_boundary::operation_input_capture::WorthQueryOperationInputCaptureDenial::Allocation(_)),
                "reach the actual payload allocation owner after the live opening");
            assert!(matches!(cause.execution_stop(), Some(Denial::Interrupted(actual)) if actual == *expected));
        }
    }).unwrap();
}
#[test]
fn funded_capture_capability_input_holds_payload_until_its_last_native_owner() {
    let (world, principal) = super::capability_world();
    world.authorization_time.hold(time(100));
    let request = live_scope();
    let original = capability_input(100);
    type InputBinding = <CapabilityTouchMutationBinding as ApplicationMutationBinding<
        IdentityExecutionSchema,
    >>::InputBinding;
    let canonical = ApplicationEncodedInput::<InputBinding>::encode(original.clone()).unwrap();
    world
        .application
        .with_application_advancement(&request, |phase| {
            let execution = phase
                .checked_execution_request_for(world.application.product_runtime())
                .unwrap();
            let worth_execution::ExecutionAllocationPolicy::Execution(parent) =
                execution.allocation_policy()
            else {
                panic!("capture must be governed by actual native execution resources")
            };
            let before = parent.charged_memory_bytes();
            let captured = world
                .application
                .capture_operation_input_in_advancement::<InputBinding>(
                    &phase,
                    &program_publication_access(),
                    &request,
                    &original,
                )
                .unwrap();
            let quote = u64::try_from(
                std::alloc::Layout::array::<CapabilityTouchInput>(1)
                    .unwrap()
                    .size(),
            )
            .unwrap();
            assert_eq!(
                parent.charged_memory_bytes() - before,
                quote,
                "actual typed payload, not estimated nested heap"
            );
            assert_eq!(
                captured.identity(),
                canonical.identity(),
                "canonical identity bytes unchanged"
            );
            let sibling = captured.clone();
            assert_eq!(
                parent.charged_memory_bytes() - before,
                quote,
                "sharing issues no second ticket/copy"
            );
            let capability = world
                .application
                .installed_schema()
                .capability(
                    TouchAccountCapability::reference(),
                    CapabilityTouchOperation::reference(),
                )
                .unwrap();
            let admitted = world
                .selected_product(&phase.test_host_request())
                .admit_encoded_capability_access(
                    &phase,
                    &program_publication_access(),
                    &principal,
                    &capability,
                    sibling,
                    &request,
                    None,
                )
                .unwrap();
            let held = parent.charged_memory_bytes();
            drop(captured);
            assert_eq!(
                parent.charged_memory_bytes(),
                held,
                "admitted capability retains SAME payload after initiating encoded owner drops"
            );
            let operation = world
                .application
                .installed_schema()
                .installed_operation(CapabilityTouchOperation::reference())
                .unwrap();
            let admission = world
                .application
                .authorize_capability_operation(&phase, admitted, &operation, Default::default())
                .unwrap();
            let retained = crate::domain_computation::primary_graph::WorthQueryApplicationCommitAuthorityBinding::from_admission(
                &admission,
                idempotency(98, 98),
            );
            drop(admission);
            assert!(
                parent.charged_memory_bytes() >= before + quote,
                "retained original meaning keeps the native payload's final-owner custody"
            );
            assert_eq!(
                retained.retained_governed_input::<CapabilityTouchInput>().unwrap().amount,
                50,
            );
            drop(retained);
            assert!(
                parent.charged_memory_bytes() + quote <= held,
                "last actual native input owner releases the reservation"
            );
        })
        .unwrap();
}
