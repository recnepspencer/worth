//! close reads.
use super::*;

pub(in super::super) fn close_reads(
    phase_custody: &crate::domain_computation::primary_graph::TestHostRequest<'_>,
    world: &World,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    approved: WorthQueryApprovedElevation,
) -> CloseReads {
    let closer =
        super::super::approval_transition::authenticated(phase_custody, world, "bob", request);
    let access = close_access(phase_custody, world, &closer, request).unwrap();
    let operation = world
        .application
        .installed_schema()
        .installed_operation(RevokeCapabilityElevationOperation::reference())
        .unwrap();
    let admission = world
        .application
        .authorize_elevation_close(
            &phase_custody.phase_for_application(&world.application),
            approved,
            access,
            &operation,
            Default::default(),
        )
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(
            &phase_custody.phase_for_application(&world.application),
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
            &phase_custody.phase_for_application(&world.application),
            admission,
            projection,
            AllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .complete_projected_dependencies(
            &phase_custody.phase_for_application(&world.application),
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
}
