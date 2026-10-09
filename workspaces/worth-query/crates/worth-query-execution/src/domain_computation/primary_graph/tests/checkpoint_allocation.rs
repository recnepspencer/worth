//! Actual Query capture, native-region custody and ordinary fresh readmission.
use std::{num::NonZeroUsize, sync::OnceLock};
use worth_execution::{
    CancellationToken, ExecutionAllocationDenialKind, ExecutionAuthority, ExecutionAuthorityConfig,
    LeaseDenial, LeaseRequest,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatus;

use super::{
    fixture::{installed_world, restored_world},
    retired_index_checkpoint::resolve_alice,
};
use crate::domain_computation::primary_graph::{
    WorthQueryCheckpointCaptureDenial, WorthQueryCheckpointCapturePolicy as Policy,
};

static AUTHORITY: OnceLock<ExecutionAuthority> = OnceLock::new();

fn request(bytes: u64) -> LeaseRequest {
    LeaseRequest {
        policy: ExecutionRequestPolicy::new(
            ExecutionPosture::Serial,
            DeterminismContract::CanonicalBitwise,
            ExecutionBudget::new(NonZeroUsize::new(1).unwrap(), bytes, 1),
        ),
        deadline: None,
        cancellation: CancellationToken::new(),
    }
}

#[test]
fn leased_checkpoint_retains_one_frame_charge_through_native_handoff_and_reopen() {
    crate::domain_computation::primary_graph::bootstrap_publication::execution_refusals::isolated(
        concat!(
            module_path!(),
            "::leased_checkpoint_retains_one_frame_charge_through_native_handoff_and_reopen"
        ),
        leased_checkpoint_retains_one_frame_charge_through_native_handoff_and_reopen_isolated,
    );
}

fn leased_checkpoint_retains_one_frame_charge_through_native_handoff_and_reopen_isolated() {
    let authority = AUTHORITY.get_or_init(|| {
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::new(1).unwrap(),
            charged_memory_bytes: None,
        })
        .unwrap()
    });
    let world = installed_world(&[("alice", WorthQueryPrincipalMappingStatus::Enabled)]);
    let original_principal = resolve_alice(&world, 1);
    let zero = authority.request_lease(request(0)).unwrap();
    let refusal = world
        .application
        .capture_application_checkpoint(Policy::Execution(&zero))
        .unwrap_err();
    let WorthQueryCheckpointCaptureDenial::Allocation(refusal) = refusal else {
        panic!("genuine lease refusal must preserve its exact typed cause");
    };
    assert_eq!(
        refusal.kind(),
        ExecutionAllocationDenialKind::Lease(LeaseDenial::MemoryExhausted(
            worth_execution::MemoryLimitDenial {
                requested: refusal.requested_payload_bytes().unwrap(),
                admitted: 0,
                level: worth_execution::MemoryLimitLevel::Policy { ancestor: 0 }
            }
        ))
    );
    assert!(refusal.requested_payload_bytes().unwrap() > 0);
    drop(zero);

    const LIMIT: u64 = 8 * 1024 * 1024;
    let parent = authority.request_lease(request(LIMIT)).unwrap();
    let lease = parent.child(request(LIMIT)).unwrap();
    let (checkpoint, sections) = world
        .application
        .capture_application_checkpoint_with_sections(Policy::Execution(&lease))
        .unwrap();
    let charge = u64::try_from(checkpoint.bytes().len()).unwrap();
    assert_eq!(checkpoint.charged_payload_bytes(), Some(charge));
    assert_eq!(sections.total_bytes(), checkpoint.bytes().len());
    assert!(charge < LIMIT);
    let pointer = checkpoint.bytes().as_ptr();
    let clone = checkpoint.clone();
    assert_eq!(clone.bytes().as_ptr(), pointer);
    assert_eq!(clone.charged_payload_bytes(), Some(charge));
    let decoded = clone.decode().unwrap();
    let (system, system_sections) = super::super::WorthQueryApplicationCheckpoint::encode(
        decoded.native.clone(),
        world.application.publication(),
        &decoded.accepted_outputs,
        Policy::SystemAllocation,
    )
    .unwrap();
    // Both policies encode the SAME actual native payload and accepted rows.
    // Policy changes physical custody, never the authenticated v9 grammar.
    assert_eq!(system.bytes(), checkpoint.bytes());
    assert_eq!(system.charged_payload_bytes(), None);
    assert_eq!(system_sections.total_bytes(), sections.total_bytes());
    drop(system);
    let native = decoded.native;
    assert_eq!(native.bytes().as_ptr(), pointer.wrapping_add(66));
    let native_clone = native.clone();
    assert_eq!(native_clone.bytes().as_ptr(), native.bytes().as_ptr());
    drop(lease);
    drop(world);

    let restored = restored_world(checkpoint.clone()).unwrap();
    assert_eq!(resolve_alice(&restored, 1), original_principal);
    drop(restored);
    drop(checkpoint);
    drop(native);
    // A native-region clone alone retains the ENTIRE enclosing frame charge,
    // after the originating lease, Query frame and original World have gone.
    assert!(matches!(
        parent.reserve_memory(LIMIT - charge + 1),
        Err(memory) if memory == worth_execution::MemoryLimitDenial { requested: LIMIT - charge + 1, admitted: LIMIT - charge, level: worth_execution::MemoryLimitLevel::Policy { ancestor: 0 } }
    ));
    assert_eq!(
        parent.reserve_memory(LIMIT - charge).unwrap().bytes(),
        LIMIT - charge
    );
    drop(native_clone);
    assert_eq!(parent.reserve_memory(LIMIT).unwrap().bytes(), LIMIT);
}
