//! Real installed projection exercises the retained-key owner. No commit or
//! downstream heap-closure claim is made by this proof.
use crate::process::domain_computation::primary_graph::application_attempt::retained_decision_facts::tests::custody::isolated;
use crate::process::domain_computation::primary_graph::application_attempt::retained_decision_facts::tests::custody::SERIAL;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountStatus;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::TouchAccountOperation;
use worth_query_execution::facade::domain_computation::WorthQueryInvariantProjectionDenialKind;
use worth_execution::ExecutionAllocationDenialClass;
use worth_execution::ExecutionAllocationPolicy as Policy;
use worth_execution::LeaseDenial;

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
            authenticated_principal(
                &(phase).test_host_request_for_test(),
                &world,
                &request_scope,
            )
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            resolved_account(
                &(phase).test_host_request_for_test(),
                &world,
                "open",
                &request_scope,
            )
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
                .selected_product(&(phase).test_host_request_for_test())
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
    let denial = world
        .application
        .with_host_advancement(|phase| {
            let zero =
                worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::memory_child(phase.borrowed_request_for_test(), 0);
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
                Policy::Execution(&zero),
            ) {
                Err(denial) => denial,
                Ok(_) => panic!("nonempty canonical key requires backing"),
            }
        })
        .expect("the fixture read owns its installed host call");
    let retained = denial.invariant_denial().unwrap();
    assert_eq!(
        retained.kind(),
        WorthQueryInvariantProjectionDenialKind::SourceRetentionDenied
    );
    assert_eq!(retained.projection_work().unwrap().equality_lookups(), 1);
    let allocation = retained.allocation_denial().unwrap();
    assert_eq!(
        allocation.classify(),
        ExecutionAllocationDenialClass::Request(worth_execution::WorkCeilingDenial::Admission(
            LeaseDenial::MemoryExhausted(worth_execution::MemoryLimitDenial::Capacity {
                requested: allocation.requested_payload_bytes().unwrap(),
                admitted: 0,
                level: worth_execution::MemoryLimitLevel::Policy { ancestor: 0 }
            })
        ))
    );
    assert!(allocation.requested_payload_bytes().unwrap() > 0);
    const PAYLOAD: u64 = 4 * 1024 * 1024;
    let (snapshot, work, mut probe) = world
        .application
        .with_host_advancement(|phase| {
            let parent = worth_query_testkit_execution_primary_graph_application_attempt_2::test_request_policy::memory_child(
                phase.borrowed_request_for_test(),
                PAYLOAD,
            );
            let child = parent.controlled_child(worth_execution::CancellationToken::new(), None);
            let (_, snapshot, work) = world
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
                .into_parts();
            (snapshot, work, parent.reserve_memory(0).unwrap())
        })
        .expect("the fixture read owns its installed host call");
    assert_eq!(work.equality_lookups(), 1);
    assert!(
        matches!(
            probe.resize(PAYLOAD),
            Err(worth_execution::MemoryLimitDenial::Capacity { requested, admitted, level: worth_execution::MemoryLimitLevel::Policy { ancestor: 0 } }) if requested == PAYLOAD && admitted < PAYLOAD
        ),
        "sealed snapshot retains its actual inline/key backings"
    );
    drop(snapshot);
    probe.resize(PAYLOAD).unwrap();
    assert_eq!(probe.bytes(), PAYLOAD);
}
