//! Real Report admission refuses before allocation on its original funded read.
use super::*;
use crate::domain_computation::primary_graph::{
    application_query::FreshQueryPermissionStop,
    causal_inspection::application_inspection::{
        stop::{InspectionCause, WorthQueryApplicationInspectionResourceRefusal},
        tests::observe,
        WorthQueryApplicationInspectionDenial,
    },
    tests::fixture::{installed_authorization_world, live_scope, TestAccountSourceBinding},
};
use std::{num::NonZeroUsize, time::Duration};
use worth_execution::{
    ExecutionAllocationPolicy, ExecutionAuthority, ExecutionWorkCeiling, LeaseRequest,
    MemoryLimitDenial,
};

#[test]
fn real_report_refusal_preserves_original_payer_until_owned_denial_closes() {
    let world = installed_authorization_world(true);
    let request = live_scope();
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
    let mut held_before_report = None;
    let handoff = world.application.with_funded_application_advancement(
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
            let result = world.application.with_fresh_retained_query_identities(
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
                    .unwrap_or_else(|_| panic!("genuine fresh preparation must succeed"));
                    read.check(&execution)
                        .unwrap_or_else(|_| panic!("real currentness must precede filling"));
                    let held = read.memory.as_ref().unwrap().bytes();
                    assert!(held > 0);
                    held_before_report = Some(held);
                    let ExecutionAllocationPolicy::Execution(actual) =
                        execution.allocation_policy()
                    else {
                        panic!("the genuine phase must retain the original accounted caller")
                    };
                    let frame =
                        ExecutionWorkCeiling::scope_memory_bound_for_test(Some(actual), None)
                            .unwrap();
                    let available = actual.available_memory_bytes().unwrap();
                    let filled = actual
                        .reserve_memory(available.checked_sub(frame).unwrap())
                        .unwrap();
                    // This is the actual production report owner immediately after
                    // its real currentness check, with its real target and session.
                    let refusal =
                        report::materialize(&execution, &read.target, &read.session, read.memory)
                            .err()
                            .expect("report backing must refuse after its real frame admits");
                    drop(filled);
                    Err::<(), _>(FreshQueryPermissionStop::Inspect(
                        InspectionAdmissionDenial::Report(refusal),
                    ))
                },
            );
            let Err(FreshQueryPermissionStop::Inspect(cause)) = result else {
                panic!("a real Report refusal must survive its issued-guard continuation")
            };
            failed = Some(cause);
        },
    );
    assert!(matches!(handoff, Ok(()) | Err(crate::domain_computation::primary_graph::application_contribution::WorthQueryAdvancementDenial::NestedStopped)));
    let denial = WorthQueryApplicationInspectionDenial::new(
        InspectionCause::Admission(failed.expect("the real report owner returned its refusal")),
        memory,
    );
    assert!(
        matches!(denial.resource_refusal(), Some(WorthQueryApplicationInspectionResourceRefusal::Memory(
        MemoryLimitDenial::Capacity { requested, admitted, .. }
    )) if requested > admitted && *admitted == held_before_report.unwrap())
    );
    assert!(
        parent.charged_memory_bytes() > before_memory,
        "owned denial keeps its actual original ticket"
    );
    assert_eq!(
        readers.observe().active(),
        before_readers,
        "failed report retains no current reader"
    );
    drop(denial);
    assert_eq!(parent.charged_memory_bytes(), before_memory);
}
