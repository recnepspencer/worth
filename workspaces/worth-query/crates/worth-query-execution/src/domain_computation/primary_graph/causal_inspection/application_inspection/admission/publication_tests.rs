//! Cancellation after the actual consuming terminal prevents final publication.
use super::*;
use crate::domain_computation::primary_graph::{
    application_query::FreshQueryPermissionStop,
    causal_inspection::application_inspection::{
        stop::InspectionCause, tests::observe, WorthQueryApplicationInspectionDenial,
    },
    tests::fixture::{installed_authorization_world, TestAccountSourceBinding},
};
use std::{
    num::NonZeroUsize,
    time::{Duration, Instant},
};
use worth_execution::{ExecutionAllocationPolicy, ExecutionAuthority, LeaseRequest};
use worth_query_admission::facade::authenticated_principal::WorthQueryCancellationSource;

#[test]
fn cancelled_after_consuming_completion_cannot_publish_and_keeps_original_payer() {
    let world = installed_authorization_world(true);
    let cancellation = WorthQueryCancellationSource::new();
    let request = WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(60)),
        cancellation.token(),
    );
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let original = observe(&world, &external, &request, false);
    let installed = world
        .application
        .installed_schema()
        .installed_query_binding::<TestAccountSourceBinding>()
        .unwrap();
    let policy = worth_foundational::ExecutionRequestPolicy::new(
        worth_foundational::ExecutionPosture::Serial,
        worth_foundational::DeterminismContract::CanonicalBitwise,
        worth_foundational::ExecutionBudget::new(
            NonZeroUsize::MIN,
            Some(32 * 1024 * 1024),
            Some(100_000_000),
        ),
    );
    let authority = ExecutionAuthority::serial();
    let parent = authority
        .request_lease(LeaseRequest {
            policy,
            deadline: None,
            cancellation: worth_execution::CancellationToken::new(),
        })
        .unwrap();
    let readers = world.application.application_query_basis_observer();
    let before_readers = readers.observe().active();
    let before_memory = parent.charged_memory_bytes();
    let mut memory = None;
    let mut failed = None;
    let mut completed_actual_terminal = false;
    world
        .application
        .with_funded_application_advancement(
            &request,
            ExecutionAllocationPolicy::Execution(&parent),
            |phase| {
                let execution = phase
                    .checked_execution_request_for(world.application.product_runtime())
                    .unwrap();
                let selected = world
                    .application
                    .on_branch(world.application.current_world())
                    .select_with_request(&execution)
                    .unwrap();
                let (report, terminal) = world
                    .application
                    .with_fresh_retained_query_identities(
                        &execution,
                        &installed,
                        None,
                        &external,
                        &original.source,
                        &selected,
                        &request,
                        |_, principal, scope| {
                            let mut read = prepare(
                                &world.application,
                                &execution,
                                &installed,
                                &selected,
                                principal,
                                scope,
                                &request,
                                &original,
                                None,
                                &mut memory,
                            )
                            .unwrap_or_else(|_| {
                                panic!("actual fresh inspection preparation succeeds")
                            });
                            let report = read
                                .materialize(&execution)
                                .unwrap_or_else(|_| panic!("actual report materializes"));
                            let terminal = read
                                .terminal(&execution)
                                .unwrap_or_else(|_| panic!("actual final read check succeeds"));
                            Ok::<_, FreshQueryPermissionStop<()>>((report, terminal))
                        },
                    )
                    .unwrap_or_else(|_| panic!("issued identity continuation succeeds"));
                let (_, product, basis) = selected.into_parts();
                drop(product);
                let completed = terminal
                    .complete(&execution, basis.release())
                    .unwrap_or_else(|_| panic!("actual session completes after release"));
                completed_actual_terminal = true;
                assert!(memory.as_ref().unwrap().bytes() > 0);
                cancellation.cancel();
                failed = Some(
                    completed
                        .publish(&execution, &original, report, &mut memory)
                        .err()
                        .expect("cancelled final publication cannot return prepared success"),
                );
            },
        )
        .unwrap();
    assert!(completed_actual_terminal);
    let denial = WorthQueryApplicationInspectionDenial::new(
        InspectionCause::Admission(failed.unwrap()),
        memory,
    );
    assert!(matches!(denial.cause, InspectionCause::Admission(InspectionAdmissionDenial::RequestControl(
        worth_query_admission::facade::authenticated_principal::WorthQueryRequestInterruption::Cancelled
    ))));
    assert!(parent.charged_memory_bytes() > before_memory);
    assert_eq!(readers.observe().active(), before_readers);
    drop(denial);
    assert_eq!(parent.charged_memory_bytes(), before_memory);
}
