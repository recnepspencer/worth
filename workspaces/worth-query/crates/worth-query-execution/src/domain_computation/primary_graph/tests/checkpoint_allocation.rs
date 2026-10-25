//! Actual Query capture, native-region custody and ordinary fresh readmission.
use worth_execution::{ExecutionAllocationDenialKind, LeaseDenial};
use worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatus;

use super::{
    fixture::{installed_world, restored_world},
    retired_index_checkpoint::resolve_alice,
};
use crate::domain_computation::primary_graph::{
    WorthQueryCheckpointCaptureDenial, WorthQueryCheckpointCapturePolicy as Policy,
};

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
    let world = installed_world(&[("alice", WorthQueryPrincipalMappingStatus::Enabled)]);
    let original_principal = world
        .application
        .with_host_advancement(|phase| resolve_alice(&(phase).test_host_request(), &world, 1))
        .expect("fixture host admits the identity read");
    let refusal = world
        .application
        .with_host_advancement(|phase| {
            let zero =
                crate::test_request_policy::memory_child(phase.borrowed_request_for_test(), 0);
            world
                .application
                .capture_application_checkpoint(&phase, Policy::Execution(&zero))
        })
        .expect("installed host call opens")
        .unwrap_err();
    let WorthQueryCheckpointCaptureDenial::Allocation(refusal) = refusal else {
        panic!("genuine lease refusal must preserve its exact typed cause");
    };
    assert_eq!(
        refusal.kind(),
        ExecutionAllocationDenialKind::Lease(LeaseDenial::MemoryExhausted(
            worth_execution::MemoryLimitDenial::Capacity {
                requested: refusal.requested_payload_bytes().unwrap(),
                admitted: 0,
                level: worth_execution::MemoryLimitLevel::Policy { ancestor: 0 }
            }
        ))
    );
    assert!(refusal.requested_payload_bytes().unwrap() > 0);

    const LIMIT: u64 = 8 * 1024 * 1024;
    let (checkpoint, sections, mut probe) = world
        .application
        .with_host_advancement(|phase| {
            let parent =
                crate::test_request_policy::memory_child(phase.borrowed_request_for_test(), LIMIT);
            let lease = parent.controlled_child(worth_execution::CancellationToken::new(), None);
            let (checkpoint, sections) = world
                .application
                .capture_application_checkpoint_with_sections(&phase, Policy::Execution(&lease))
                .unwrap();
            (checkpoint, sections, parent.reserve_memory(0).unwrap())
        })
        .expect("installed host call opens");
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
    // Policy changes physical custody, never the authenticated current grammar.
    assert_eq!(system.bytes(), checkpoint.bytes());
    assert_eq!(system.charged_payload_bytes(), None);
    assert_eq!(system_sections.total_bytes(), sections.total_bytes());
    drop(system);
    let native = decoded.native;
    // Independent current wire layout: magic, checksum, Query/facts/instances
    // versions (u16 each), bootstrap commit, native length and accepted count.
    const NATIVE_OFFSET: usize =
        8 + 32 + 3 * std::mem::size_of::<u16>() + 3 * std::mem::size_of::<u64>();
    assert_eq!(native.bytes().as_ptr(), pointer.wrapping_add(NATIVE_OFFSET));
    let native_clone = native.clone();
    assert_eq!(native_clone.bytes().as_ptr(), native.bytes().as_ptr());
    drop(world);

    let restored = restored_world(checkpoint.clone()).unwrap();
    assert_eq!(
        restored
            .application
            .with_host_advancement(|phase| resolve_alice(
                &(phase).test_host_request(),
                &restored,
                1
            ))
            .expect("fixture host admits the restored identity read"),
        original_principal
    );
    drop(restored);
    drop(checkpoint);
    drop(native);
    // A native-region clone alone retains the ENTIRE enclosing frame charge,
    // after the originating lease, Query frame and original World have gone.
    assert!(matches!(
        probe.resize(LIMIT - charge + 1),
        Err(memory) if memory == worth_execution::MemoryLimitDenial::Capacity { requested: LIMIT - charge + 1, admitted: LIMIT - charge, level: worth_execution::MemoryLimitLevel::Policy { ancestor: 0 } }
    ));
    probe.resize(LIMIT - charge).unwrap();
    assert_eq!(probe.bytes(), LIMIT - charge);
    probe.resize(0).unwrap();
    drop(native_clone);
    probe.resize(LIMIT).unwrap();
    assert_eq!(probe.bytes(), LIMIT);
}
