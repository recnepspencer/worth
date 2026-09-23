//! Scheduled public-path qualification for a finite, sparse 10k-node definition.

use worth_query_host::facade::{
    application_entry::{
        WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
        WorthQueryApplicationRequestExt,
    },
    declaration::application_program::{
        ApplicationWorkflowComponentLimits, ApplicationWorkflowControlOutcome,
        ApplicationWorkflowDefinitionBuilder, ApplicationWorkflowDefinitionLimits,
    },
};
use worth_query_installation::facade::WorthQueryApplicationWorkflowResourceCeiling;
use worth_query_replay::facade::WorthQueryCertificationCostRuntimeExt;

use super::bounded_dimension_model::{
    dimension_entry::PART_IDENTITY,
    host::publish_on_first_program_for_geometry_scale,
    operator_identity::authenticate_operator,
    workflow::{
        retain_workflow_with_resources, ReviewedGeometryWorkflow, WorkflowDefinitionAuthoringInput,
        WorkflowDefinitionAuthoringIntent, WorkflowDefinitionAuthoringOperation,
    },
};

const NODES: u16 = 10_000;
const CONNECTIONS: u16 = NODES - 1;
const COMPONENTS: ApplicationWorkflowComponentLimits =
    match ApplicationWorkflowComponentLimits::new(1, 1, 1, 1, 1) {
        Some(limits) => limits,
        None => panic!("nonzero component limits"),
    };

fn sparse_definition(
) -> worth_query_host::facade::declaration::application_program::AuthoredWorkflowDefinition<
    ReviewedGeometryWorkflow,
> {
    let limits = ApplicationWorkflowDefinitionLimits::new(
        NODES,
        CONNECTIONS,
        CONNECTIONS,
        COMPONENTS,
        16 * 1024 * 1024,
    )
    .expect("finite geometry definition limits");
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedGeometryWorkflow>::new(
        "sparse-geometry-scale",
        limits,
    )
    .expect("geometry definition identity");
    let mut operations = Vec::with_capacity(usize::from(CONNECTIONS));
    for index in 0..CONNECTIONS {
        operations.push(
            builder
                .operation::<WorkflowDefinitionAuthoringOperation>(
                    format!("operation-{index:05}"),
                    false,
                )
                .expect("sparse operation"),
        );
    }
    let terminal = builder.terminal("done").expect("terminal node");
    builder.start(&operations[0]);
    builder.sequence(&operations).expect("sparse sequence");
    builder.control(
        operations.last().expect("nonempty sequence"),
        ApplicationWorkflowControlOutcome::Completed,
        &terminal,
    );
    builder.finish().expect("sparse definition closes")
}

#[test]
#[ignore = "scheduled public 10k-node publication qualification"]
fn public_ten_thousand_node_definition_publishes() {
    let resources = WorthQueryApplicationWorkflowResourceCeiling::new(
        NODES,
        CONNECTIONS,
        CONNECTIONS,
        COMPONENTS,
        16 * 1024 * 1024,
        2,
        u32::from(NODES),
        256 * 1024,
    )
    .expect("finite geometry workflow resources");
    let application =
        retain_workflow_with_resources(publish_on_first_program_for_geometry_scale(), resources);
    let runtime = application.runtime();
    let cancellation = worth_query_host::facade::admission::authenticated_principal::WorthQueryCancellationSource::new();
    let scope =
        worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope::new(
            std::time::Instant::now() + std::time::Duration::from_secs(300),
            cancellation.token(),
        );
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let cost_scope = runtime
        .capture_certification_cost_scope(application.current_world())
        .expect("geometry branch remains measurable");
    let started_at = std::time::Instant::now();
    let authored = sparse_definition();
    let authored_ms = started_at.elapsed().as_millis();
    let draft = runtime
        .request(&principal, &scope)
        .mutate(WorkflowDefinitionAuthoringIntent {
            input: WorkflowDefinitionAuthoringInput {
                identity: PART_IDENTITY.to_owned(),
                dimension: 8,
            },
        })
        .workflow(&application, authored)
        .expect("the 10k definition validates against installed vocabulary");
    let validated_ms = started_at.elapsed().as_millis();
    let publication = draft
        .publish(WorkflowDefinitionExpectedPredecessor::Absent)
        .idempotency(&918_400)
        .execute()
        .expect("the 10k definition prepares for publication");
    let cost = runtime
        .observe_certification_cost(&cost_scope)
        .expect("geometry publication cost remains measurable");
    let sharing = cost.relational().sharing_cost_delta();
    assert_eq!(sharing.copied_truth_bytes, 0);
    assert!(sharing.publication_new_authoritative_bytes > 0);
    println!(
        "geometry publication authored_ms={authored_ms} validated_ms={validated_ms} elapsed_ms={} application_work={:?} copied_truth_bytes={} new_authoritative_bytes={} content_values_hashed={} touched_regions={} reused_regions={}",
        started_at.elapsed().as_millis(),
        cost.application_work(),
        sharing.copied_truth_bytes,
        sharing.publication_new_authoritative_bytes,
        sharing.publication_content_values_hashed,
        sharing.publication_touched_region_count,
        sharing.publication_reused_region_count,
    );
    match publication {
        WorkflowDefinitionPublicationOutcome::Published(_) => {}
        other => panic!("the 10k definition must publish: {other:?}"),
    }
}
