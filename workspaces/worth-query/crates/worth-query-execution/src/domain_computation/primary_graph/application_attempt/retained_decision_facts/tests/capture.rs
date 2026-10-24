//! Real installed projection exercises the retained-key owner. No commit or
//! downstream heap-closure claim is made by this proof.
use super::custody::{authority, isolated, SERIAL};
use crate::domain_computation::primary_graph::{
    tests::{
        application_attempt::{authenticated_principal, resolved_account},
        fixture::{
            installed_authorization_world, live_scope, AccountStatus, TouchAccountOperation,
        },
    },
    WorthQueryInvariantProjectionDenialKind, WorthQueryValueDecodeDenial,
};
use worth_execution::{
    ExecutionAllocationDenialKind, ExecutionAllocationPolicy as Policy, LeaseDenial, LeaseRequest,
};

/// A caller lease funds the whole host call it opens, including its work.
fn funding(bytes: u64) -> LeaseRequest {
    LeaseRequest {
        policy: worth_foundational::ExecutionRequestPolicy::new(
            worth_foundational::ExecutionPosture::Serial,
            worth_foundational::DeterminismContract::CanonicalBitwise,
            worth_foundational::ExecutionBudget::new(
                std::num::NonZeroUsize::MIN,
                Some(bytes),
                Some(16_000_000),
            ),
        ),
        deadline: None,
        cancellation: worth_execution::CancellationToken::new(),
    }
}

#[test]
fn installed_absence_projection_refuses_before_retention_and_releases_snapshot() {
    if !isolated(
        "installed_absence_projection_refuses_before_retention_and_releases_snapshot",
        module_path!(),
    ) {
        return;
    }
    let _serial = SERIAL.lock().unwrap();
    let world = installed_authorization_world(true);
    let request_scope = live_scope();
    let actor = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request_scope)
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(&(phase).test_host_request(), &world, "open", &request_scope)
        })
        .expect("the fixture read owns its installed host call");
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let admission = world
        .application
        .with_host_advancement(|phase| {
            world
                .selected_product(&(phase).test_host_request())
                .authorize_operation(
                    &phase,
                    &actor,
                    &account,
                    &operation,
                    Default::default(),
                    &request_scope,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    const PAYLOAD: u64 = 4 * 1024 * 1024;
    let exhausted = authority().request_lease(funding(PAYLOAD)).unwrap();
    // The read's decode scope also charges the lease before any key is
    // retained; its own refusal quotes that share (MENTALITY §7).
    let mut keep = 0;
    let denial = (0..8)
        .find_map(|_| {
            let denial = world
                .application
                .with_funded_host_advancement(Policy::Execution(&exhausted), |phase| {
                    // The call is open; its lease keeps only the decode scope's quote.
                    let room = exhausted.available_memory_bytes().unwrap();
                    let _held = exhausted.reserve_memory(room - keep).unwrap();
                    match world.invariant.project_admitted_operation(
                        &phase,
                        &admission,
                        |reader, _| {
                            // Ignoring the local error must not seal a partial/empty projection.
                            let _ = reader.decision_select_entities(
                                AccountStatus::reference(),
                                "unseen-retained-key".to_owned(),
                                2,
                            );
                        },
                        Policy::Execution(&exhausted),
                    ) {
                        Err(denial) => denial,
                        Ok(_) => panic!("nonempty canonical key requires backing"),
                    }
                })
                .expect("the fixture read owns its installed host call");
            match denial.invariant_denial().map(|denial| denial.kind()) {
                Some(WorthQueryInvariantProjectionDenialKind::ValueDecode(
                    WorthQueryValueDecodeDenial::Scope(worth_execution::WorkCeilingDenial::Admission(
                        LeaseDenial::MemoryExhausted(worth_execution::MemoryLimitDenial::Capacity {
                            requested,
                            admitted,
                            ..
                        }),
                    )),
                )) => {
                    keep += requested - admitted;
                    None
                }
                _ => Some(denial),
            }
        })
        .expect("the read reaches key retention under a learned keep");
    let retained = denial.invariant_denial().unwrap();
    assert_eq!(
        retained.kind(),
        WorthQueryInvariantProjectionDenialKind::SourceRetentionDenied
    );
    assert_eq!(retained.projection_work().unwrap().equality_lookups(), 1);
    let allocation = retained.allocation_denial().unwrap();
    assert!(allocation.requested_payload_bytes().is_some());
    assert_eq!(
        allocation.kind(),
        ExecutionAllocationDenialKind::Lease(LeaseDenial::MemoryExhausted(
            worth_execution::MemoryLimitDenial::Capacity {
                requested: allocation.requested_payload_bytes().unwrap(),
                admitted: 0,
                // The attempt's control child sits under the call's child of
                // the lease; the lease's held room refuses it.
                level: worth_execution::MemoryLimitLevel::Policy { ancestor: 2 }
            }
        ))
    );
    assert!(allocation.requested_payload_bytes().unwrap() > 0);
    assert_eq!(exhausted.charged_memory_bytes(), 0);
    drop(exhausted);
    let parent = authority().request_lease(funding(PAYLOAD)).unwrap();
    let child = parent.child(funding(PAYLOAD)).unwrap();
    let (_, snapshot, work) = world
        .application
        .with_funded_host_advancement(Policy::Execution(&child), |phase| {
            world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &admission,
                    |reader, _| {
                        assert!(reader
                            .decision_select_entities(
                                AccountStatus::reference(),
                                "unseen-retained-key".to_owned(),
                                2
                            )
                            .unwrap()
                            .is_empty());
                    },
                    Policy::Execution(&child),
                )
                .unwrap()
                .into_parts()
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(work.equality_lookups(), 1);
    drop(child);
    assert!(
        matches!(
            parent.reserve_memory(PAYLOAD),
            Err(worth_execution::MemoryLimitDenial::Capacity { requested, admitted, level: worth_execution::MemoryLimitLevel::Policy { ancestor: 0 } }) if requested == PAYLOAD && admitted < PAYLOAD
        ),
        "sealed snapshot retains its actual inline/key backings"
    );
    drop(snapshot);
    assert_eq!(parent.reserve_memory(PAYLOAD).unwrap().bytes(), PAYLOAD);
}
