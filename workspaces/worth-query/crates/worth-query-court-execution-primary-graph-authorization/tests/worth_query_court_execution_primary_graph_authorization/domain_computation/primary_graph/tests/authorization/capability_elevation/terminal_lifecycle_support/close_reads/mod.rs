//! close reads.
use crate::domain_computation::primary_graph::tests::authorization::capability_elevation::terminal_lifecycle_support::*;

pub fn close_reads(
    phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    world: &World,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    approved: WorthQueryApprovedElevation,
) -> CloseReads {
    let closer =
        crate::domain_computation::primary_graph::tests::authorization::capability_elevation::approval_transition::authenticated(phase_custody, world, "bob", request);
    let access = close_access(phase_custody, world, &closer, request).unwrap();
    let operation = world
        .application
        .installed_schema()
        .installed_operation(RevokeCapabilityElevationOperation::reference())
        .unwrap();
    let admission = world
        .application
        .authorize_elevation_close(
            &phase_custody.phase_for_application_for_test(&world.application),
            approved,
            access,
            &operation,
            Default::default(),
        )
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(
            &phase_custody.phase_for_application_for_test(&world.application),
            &admission,
            |reader, _| {
                seal_lifecycle_facts(reader);
                let elevation = reader
                    .resolve_entity(
                        CapabilityElevationIdentity::reference(),
                        "elevation-2".to_owned(),
                    )
                    .unwrap();
                reader
                    .require_decision_field(&elevation, CapabilityElevationClosedAt::reference())
                    .unwrap();
            },
            AllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .into_parts();
    world
        .application
        .begin_projected_application_read_attempt(
            &phase_custody.phase_for_application_for_test(&world.application),
            admission,
            projection,
            AllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .complete_projected_dependencies(
            &phase_custody.phase_for_application_for_test(&world.application),
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
}
