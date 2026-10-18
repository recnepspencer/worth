//! Named owning action installation retains an independent native read authority.
use super::super::programs::WorkflowOwnerProgram;
use super::*;
#[allow(
    dead_code,
    reason = "Shared fixture factory is used by the compiled authored_workflow example"
)]
pub fn publish_owner_workflow() -> DocumentWorkflowRuntime<WorkflowOwnerProgram> {
    publish_owner_workflow_with_projection().0
}
pub fn publish_owner_workflow_with_projection() -> (
    DocumentWorkflowRuntime<WorkflowOwnerProgram>,
    primary_graph::WorthQueryApplicationInvariantProjectionAuthority<DocumentRetentionSchema>,
) {
    let program =
        worth_query_host::facade::declaration::application_program::ApplicationProgramAuthoring::<
            DocumentRetentionSchema,
            WorkflowOwnerProgram,
        >::begin()
        .validated_program()
        .unwrap();
    let mut projection = None;
    let application = in_memory_rostered_program(
        program,
        // The contribution installs both V1 and V2 rule contracts. Keep V2
        // honestly declared in the existing roster; this branch still runs owner V1.
        WorthQueryApplicationProgramRoster::new()
            .support(validated_second_program())
            .support(super::super::actor_generation::validated_actor_program()),
        DocumentRetentionSchema::declaration().unwrap(),
        ((),),
        host_limits(),
        |phase, graph, schema| {
            projection = Some(graph.retain_invariant_projection_authority());
            seed_host(phase, graph, schema)
        },
    )
    .unwrap();
    (
        super::super::workflow::retain_workflow(application),
        projection.expect("real installation issues observation authority"),
    )
}
pub fn read_owner_approval_grant(
    runtime: &primary_graph::WorthQueryPrimaryGraphApplicationRuntime<DocumentRetentionSchema>,
    projection: &primary_graph::WorthQueryApplicationInvariantProjectionAuthority<
        DocumentRetentionSchema,
    >,
) -> String {
    use super::super::workflow::declaration::{
        WorkflowGrantIdentityField, WorkflowGrantStatusField,
    };
    runtime
        .with_host_advancement(|phase| {
            let read = projection
                .project(
                    &phase,
                    |reader| {
                        let grant = reader
                            .resolve_entity(
                                WorkflowGrantIdentityField::reference(),
                                "workflow-approval-grant".to_owned(),
                            )
                            .unwrap();
                        reader
                            .field(&grant, WorkflowGrantStatusField::reference())
                            .unwrap()
                    },
                    runtime::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap();
            read.output().clone()
        })
        .unwrap()
}
