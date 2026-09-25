//! Scheduled public-path qualification for a finite, sparse 10k-node definition.

use worth_query_host::facade::{
    application_entry::{
        WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
        WorkflowInstanceStartOutcome, WorthQueryApplicationRequestExt,
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
        retain_workflow_with_resources, start_instance, ReviewedGeometryWorkflow,
        WorkflowDefinitionAuthoringInput, WorkflowDefinitionAuthoringIntent,
        WorkflowDefinitionAuthoringOperation,
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
    nodes: u16,
) -> worth_query_host::facade::declaration::application_program::AuthoredWorkflowDefinition<
    ReviewedGeometryWorkflow,
> {
    let connections = nodes - 1;
    let limits = ApplicationWorkflowDefinitionLimits::new(
        nodes,
        connections,
        connections,
        COMPONENTS,
        16 * 1024 * 1024,
    )
    .expect("finite geometry definition limits");
    let mut builder = ApplicationWorkflowDefinitionBuilder::<ReviewedGeometryWorkflow>::new(
        "sparse-geometry-scale",
        limits,
    )
    .expect("geometry definition identity");
    let mut operations = Vec::with_capacity(usize::from(connections));
    for index in 0..connections {
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
    let authored = sparse_definition(NODES);
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
    let WorkflowDefinitionPublicationOutcome::Published(published) = publication else {
        panic!("the 10k definition must publish: {publication:?}");
    };
    let compilation = runtime.workflow_compilation_reuse_counters();
    let cold_started_at = std::time::Instant::now();
    assert!(matches!(
        start_instance(&application, published.definition().clone(), 918_402)
            .expect("the 10k cold start must fit its own operation budget"),
        WorkflowInstanceStartOutcome::Started(_)
    ));
    let cold = runtime.workflow_compilation_reuse_counters();
    assert_eq!(cold.cold_misses(), compilation.cold_misses() + 1);
    println!(
        "geometry cold start elapsed_ms={}",
        cold_started_at.elapsed().as_millis()
    );
    assert!(matches!(
        start_instance(&application, published.definition().clone(), 918_403)
            .expect("the 10k warm start must reuse compiled meaning"),
        WorkflowInstanceStartOutcome::Started(_)
    ));
    let warm = runtime.workflow_compilation_reuse_counters();
    assert_eq!(warm.warm_hits(), cold.warm_hits() + 1);
    assert_eq!(warm.cold_misses(), cold.cold_misses());
}

#[test]
fn cold_compilation_of_large_definition_uses_bounded_start_facts() {
    const NODES: u16 = 512;
    const CONNECTIONS: u16 = NODES - 1;
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
    .expect("finite workflow resources");
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
    let publication = runtime
        .request(&principal, &scope)
        .mutate(WorkflowDefinitionAuthoringIntent {
            input: WorkflowDefinitionAuthoringInput {
                identity: PART_IDENTITY.to_owned(),
                dimension: 8,
            },
        })
        .workflow(&application, sparse_definition(NODES))
        .expect("large definition validates")
        .publish(WorkflowDefinitionExpectedPredecessor::Absent)
        .idempotency(&918_401)
        .execute()
        .expect("large definition publishes");
    let WorkflowDefinitionPublicationOutcome::Published(published) = publication else {
        panic!("expected publication, got {publication:?}");
    };
    let before = runtime.workflow_compilation_reuse_counters();
    assert!(matches!(
        start_instance(&application, published.definition().clone(), 918_402)
            .expect("cold start must fit its admission budget"),
        WorkflowInstanceStartOutcome::Started(_)
    ));
    let cold = runtime.workflow_compilation_reuse_counters();
    assert_eq!(cold.cold_misses(), before.cold_misses() + 1);
    assert!(matches!(
        start_instance(&application, published.definition().clone(), 918_403)
            .expect("warm start must reuse compiled meaning"),
        WorkflowInstanceStartOutcome::Started(_)
    ));
    let warm = runtime.workflow_compilation_reuse_counters();
    assert_eq!(warm.warm_hits(), cold.warm_hits() + 1);
    assert_eq!(warm.cold_misses(), cold.cold_misses());
}
