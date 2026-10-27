//! A retained name cannot bypass installation affinity or the original execution stop.
use super::*;
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};
use worth_query_declaration::facade::application_schema::{
    ApplicationRelationIntegrity, ApplicationRelationRef,
};

#[test]
fn privileged_reader_refuses_foreign_names_and_caller_forged_orbit_meaning() {
    let _isolation = super::super::super::super::fixture::isolated_request_owner();
    let foreign = install(&[(0, 1), (1, 0)]);
    let world = install(&[(0, 1), (1, 0)]);
    let host = crate::test_request_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let (_, foreign_name) = name(&foreign, &execution, &selected(&foreign, &execution), 0);
    let (_, local_name) = name(&world, &execution, &selected(&world, &execution), 0);
    world.application.with_host_advancement(|phase| {
        world.invariant.project(&phase, |reader| {
            let before = reader.work();
            assert_eq!(reader.lineage(Next::reference(), &foreign_name).unwrap_err().kind(),
                WorthQueryInvariantProjectionTraversalDenialKind::ForeignIdentity);
            assert_eq!(reader.work(), before);
            for (relation_name, expected) in [
                ("MappingTarget", WorthQueryInvariantProjectionTraversalDenialKind::NotOrbitRelation),
                ("Absent", WorthQueryInvariantProjectionTraversalDenialKind::RelationNotInstalled),
            ] {
                let forged: ApplicationRelationRef<Schema, Next, Node, Node> = ApplicationRelationRef::from_schema_identifiers(
                    relation_name, "Node", "Node", ApplicationRelationIntegrity::same_context_unbounded_retain_dangling().orbit());
                assert_eq!(reader.lineage(forged, &local_name).unwrap_err().kind(), expected);
            }
            let actual = reader.lineage(Next::reference(), &local_name).unwrap();
            assert_eq!(actual.latest_identity(), Some(local_name));
        }, Allocation::SystemAllocation).unwrap()
    }).unwrap();
}

#[test]
fn swallowed_lineage_cancellation_vetoes_the_original_admitted_projection() {
    let _isolation = super::super::super::super::fixture::isolated_request_owner();
    let world = install(&[(0, 1), (1, 0)]);
    let host = crate::test_request_policy::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let (_, identity) = name(&world, &execution, &selected(&world, &execution), 0);
    let request = super::super::super::super::fixture::live_scope();
    let admission = world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request();
            let execution = host
                .execution_request_for(world.application.product_runtime())
                .unwrap();
            program::authorize(
                &world,
                &phase,
                &execution,
                &request,
                0,
                ReplaceNext::reference(),
            )
            .0
        })
        .unwrap();
    let cancellation = CancellationSource::new();
    let original =
        SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
    let execution = ExecutionRequest::serial(&original);
    let contacted = std::cell::Cell::new(false);
    let denial = world
        .invariant
        .project_admitted_operation_with_request(
            &execution,
            &admission,
            |reader, _| {
                cancellation.cancel();
                let stopped = reader.lineage(Next::reference(), &identity).unwrap_err();
                assert!(matches!(
                    stopped.kind(),
                    WorthQueryInvariantProjectionTraversalDenialKind::ReadExecution(_)
                ));
                contacted.set(true);
            },
            Allocation::SystemAllocation,
        )
        .err()
        .expect("the swallowed original read stop must refuse projection completion");
    assert!(contacted.get());
    assert_eq!(
        denial.kind(),
        WorthQueryOperationProjectionDenialKind::InvariantAdmission(
            WorthQueryInvariantProjectionDenialKind::ExecutionRequest(
                worth_relational::facade::transactions::RelationalExecutionDenialCause::Cancelled
            )
        )
    );
}
