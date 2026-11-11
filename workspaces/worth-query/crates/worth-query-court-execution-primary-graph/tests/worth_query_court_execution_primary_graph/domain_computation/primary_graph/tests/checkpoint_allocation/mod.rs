//! Actual Query capture, native-region custody and ordinary fresh readmission.
use worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatus;

use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::restored_world;
use crate::domain_computation::primary_graph::tests::retired_index_checkpoint::resolve_alice;
use worth_query_execution::facade::application_installation::WorthQueryCheckpointCaptureDenial;
use worth_execution::ExecutionAllocationPolicy as Policy;

#[test]
fn leased_checkpoint_retains_one_frame_charge_through_native_handoff_and_reopen() {
    worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::primary_graph::bootstrap_publication::execution_refusals::isolated(
        concat!(
            module_path!(),
            "::leased_checkpoint_retains_one_frame_charge_through_native_handoff_and_reopen"
        ),
        leased_checkpoint_retains_one_frame_charge_through_native_handoff_and_reopen_isolated,
    );
}

pub fn leased_checkpoint_retains_one_frame_charge_through_native_handoff_and_reopen_isolated() {
    let world = installed_world(&[("alice", WorthQueryPrincipalMappingStatus::Enabled)]);
    let original_principal = world
        .application
        .with_host_advancement(|phase| {
            resolve_alice(&(phase).test_host_request_for_test(), &world, 1)
        })
        .expect("fixture host admits the identity read");
    let refusal = world
        .application
        .with_host_advancement(|phase| {
            let zero =
                worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::memory_child(phase.borrowed_request_for_test(), 0);
            world
                .application
                .capture_application_checkpoint(&phase, Policy::Execution(&zero))
        })
        .expect("installed host call opens")
        .unwrap_err();
    let WorthQueryCheckpointCaptureDenial::Execution(cause) = refusal else {
        panic!("complete capture preserves its original admission cause: {refusal:?}");
    };
    let worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenialKind::ExecutionDenied {
        kind: worth_query_execution::facade::primary_graph::WorthQueryProviderSessionDenialKind::ExecutionResource {
            denial: worth_query_execution::facade::application_contribution::WorthQueryManagedComputationResourceDenial::MemoryLimit {
                requested, admitted: 0,
                level: worth_query_execution::facade::application_contribution::WorthQueryMemoryLimitLevel::Policy,
            }, partition_identity: None, policy_ancestor: Some(0),
        }
    } = cause.kind() else { panic!("exact zero-memory capture cause: {cause:?}"); };
    assert!(requested > 0);

    const LIMIT: u64 = 8 * 1024 * 1024;
    let (checkpoint, sections, mut probe) = world
        .application
        .with_host_advancement(|phase| {
            let parent =
                worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::memory_child(phase.borrowed_request_for_test(), LIMIT);
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
    let retained = held_by_policy(&mut probe);
    assert!(
        retained > charge,
        "the complete origin retains its admitted Native and World custody"
    );
    assert!(checkpoint.capture_origin().is_some());
    let pointer = checkpoint.bytes().as_ptr();
    let clone = checkpoint.clone();
    assert_eq!(clone.bytes().as_ptr(), pointer);
    assert_eq!(clone.charged_payload_bytes(), Some(charge));
    assert_eq!(
        held_by_policy(&mut probe),
        retained,
        "cloning shares every original charge"
    );
    let decoded = clone.decode_for_test().unwrap();
    let (system, system_sections) = world
        .application
        .with_host_advancement(|phase| {
            let (system, sections) = world
                .application
                .capture_application_checkpoint_with_sections(&phase, Policy::SystemAllocation)
                .unwrap();
            let funded = world
                .application
                .inspect_application_checkpoint(&phase, checkpoint.bytes())
                .unwrap();
            let ordinary = world
                .application
                .inspect_application_checkpoint(&phase, system.bytes())
                .unwrap();
            assert_eq!(
                funded.model, ordinary.model,
                "both policies capture the same complete authored model"
            );
            assert!(system.capture_origin().is_some());
            (system, sections)
        })
        .unwrap();
    assert_eq!(system.charged_payload_bytes(), None);
    assert_eq!(system_sections.total_bytes(), sections.total_bytes());
    drop(system);
    let native = decoded.native_into_for_test();
    // Independent current wire layout: magic, checksum, Query/facts/instances
    // versions (u16 each), bootstrap commit and four section/count lengths.
    const NATIVE_OFFSET: usize =
        8 + 32 + 3 * std::mem::size_of::<u16>() + 5 * std::mem::size_of::<u64>();
    assert_eq!(native.bytes().as_ptr(), pointer.wrapping_add(NATIVE_OFFSET));
    let native_clone = native.clone();
    assert_eq!(native_clone.bytes().as_ptr(), native.bytes().as_ptr());
    drop(world);

    let restored = restored_world(checkpoint.clone()).unwrap();
    assert_eq!(
        restored
            .application
            .with_host_advancement(|phase| resolve_alice(
                &(phase).test_host_request_for_test(),
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
    let held = held_by_policy(&mut probe);
    assert_eq!(
        held, charge,
        "the native-region alias holds exactly the entire frame, nothing more"
    );
    assert!(matches!(
        probe.resize(LIMIT - held + 1),
        Err(memory) if memory == worth_execution::MemoryLimitDenial::Capacity { requested: LIMIT - held + 1, admitted: LIMIT - held, level: worth_execution::MemoryLimitLevel::Policy { ancestor: 0 } }
    ));
    probe.resize(LIMIT - held).unwrap();
    assert_eq!(probe.bytes(), LIMIT - held);
    probe.resize(0).unwrap();
    drop(native_clone);
    assert_eq!(held_by_policy(&mut probe), 0);
    probe.resize(LIMIT).unwrap();
    assert_eq!(probe.bytes(), LIMIT);
}

/// The empty probe's exact policy room exposes every other charge the policy holds.
fn held_by_policy(probe: &mut worth_execution::ExecutionMemoryReservation) -> u64 {
    const LIMIT: u64 = 8 * 1024 * 1024;
    assert_eq!(
        probe.bytes(),
        0,
        "only an empty probe observes the policy's other charges"
    );
    match probe.resize(LIMIT + 1) {
        Err(worth_execution::MemoryLimitDenial::Capacity {
            admitted,
            level: worth_execution::MemoryLimitLevel::Policy { ancestor: 0 },
            ..
        }) => LIMIT - admitted,
        other => panic!("the policy capacity observes its held charge: {other:?}"),
    }
}
