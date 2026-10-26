//! Installed meaning and the original request remain authority at dirty edges.
use super::*;
use worth_execution::{CancellationSource, SerialMemoryBudget, SerialRequest};
use worth_query_declaration::facade::application_schema::{
    ApplicationRelationIntegrity, ApplicationRelationRef,
};

#[test]
fn projection_orbits_refuse_foreign_members_and_forged_orbit_declarations() {
    let _isolation = super::super::super::super::fixture::isolated_request_owner();
    let foreign = world::install_orbit(graph(), probe::Probe::new(|_, _, _| {}));
    let member = foreign
        .application
        .with_host_advancement(|phase| {
            foreign
                .invariant
                .project(
                    &phase,
                    |reader| reader.resolve_entity(NodeKey::reference(), 0_u64).unwrap(),
                    Allocation::SystemAllocation,
                )
                .unwrap()
                .into_parts()
                .0
        })
        .unwrap();
    let world = world::install_orbit(graph(), probe::Probe::new(|_, _, _| {}));
    world
        .application
        .with_host_advancement(|phase| {
            world
                .invariant
                .project(
                    &phase,
                    |reader| {
                        let before = reader.work();
                        let denial = reader.orbit(Next::reference(), &member).unwrap_err();
                        assert_eq!(
                            denial.kind(),
                            WorthQueryInvariantProjectionTraversalDenialKind::ForeignIdentity
                        );
                        assert_eq!(reader.work(), before);
                    },
                    Allocation::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap();
    let plain = world::install_with_probe(graph(), probe::Probe::new(|_, _, _| {}));
    plain.application.with_host_advancement(|phase| {
        plain.invariant.project_operation::<ReplaceNext, _>(&phase, |reader| {
            let local = reader.resolve_entity(NodeKey::reference(), 0_u64).unwrap();
            let forged: ApplicationRelationRef<OrbitSchema<false>, Next, Node, Node> = ApplicationRelationRef::from_schema_identifiers(
                "Next", "Node", "Node",
                ApplicationRelationIntegrity::same_context_unbounded_retain_dangling().orbit(),
            );
            let denial = reader.orbit(forged, &local).unwrap_err();
            assert_eq!(denial.kind(), WorthQueryInvariantProjectionTraversalDenialKind::NotOrbitRelation);
        }, Allocation::SystemAllocation).unwrap()
    }).unwrap();
}

#[test]
fn original_cancellation_cannot_be_swallowed_by_orbit_or_identity_inspection() {
    let _isolation = super::super::super::super::fixture::isolated_request_owner();
    let world = world::install_orbit(graph(), probe::Probe::new(|_, _, _| {}));
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
    for identity_only in [false, true] {
        let cancellation = CancellationSource::new();
        let original =
            SerialRequest::from_memory(SerialMemoryBudget::new(None), cancellation.token(), None);
        let execution = ExecutionRequest::serial(&original);
        let before = std::cell::Cell::new(None);
        let denial = world
            .invariant
            .project_admitted_operation_with_request(
                &execution,
                &admission,
                |reader, local| {
                    cancellation.cancel();
                    let error = if identity_only {
                        reader.orbit_of(Next::reference(), local).map(|_| ())
                    } else {
                        reader.orbit(Next::reference(), local).map(|_| ())
                    }
                    .unwrap_err();
                    assert!(matches!(
                        error.kind(),
                        WorthQueryInvariantProjectionTraversalDenialKind::ReadExecution(_)
                    ));
                    before.set(Some(()));
                    // Intentionally return success after swallowing the original read stop.
                },
                Allocation::SystemAllocation,
            )
            .err()
            .expect("the original stop must veto projection completion");
        assert_eq!(
            before.get(),
            Some(()),
            "the attempted read reached its exact refusal"
        );
        assert_eq!(denial.kind(), WorthQueryOperationProjectionDenialKind::InvariantAdmission(
            WorthQueryInvariantProjectionDenialKind::ExecutionRequest(
                worth_relational::facade::transactions::RelationalExecutionDenialCause::Cancelled,
            ),
        ));
    }
}
