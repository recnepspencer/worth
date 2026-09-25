//! Repeated public components at exact 100/1k/10k expanded-node sizes.

use std::time::{Duration, Instant};

use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_entry::{
        WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
        WorkflowInstanceStartOutcome, WorthQueryApplicationRequestExt,
    },
    declaration::application_program::{
        ApplicationWorkflowComponentBuilder, ApplicationWorkflowComponentLimits,
        ApplicationWorkflowControlOutcome, ApplicationWorkflowDefinitionBuilder,
        ApplicationWorkflowDefinitionLimits, AuthoredWorkflowDefinition,
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

const FRAGMENT_NODES: u16 = 9;
const FRAGMENT_CONNECTIONS: u16 = FRAGMENT_NODES - 1;

fn component_definition(occurrences: u16) -> AuthoredWorkflowDefinition<ReviewedGeometryWorkflow> {
    let nodes = occurrences * FRAGMENT_NODES + 1;
    let connections = nodes - 1;
    let component_limits = component_limits(occurrences);
    let limits = ApplicationWorkflowDefinitionLimits::new(
        nodes,
        connections,
        connections,
        component_limits,
        16 * 1024 * 1024,
    )
    .expect("finite repeated component definition limits");
    let mut fragment =
        ApplicationWorkflowComponentBuilder::<ReviewedGeometryWorkflow>::new("solver-fragment")
            .expect("component identity");
    let operations = (0..FRAGMENT_NODES)
        .map(|index| {
            fragment
                .operation::<WorkflowDefinitionAuthoringOperation>(
                    format!("operation-{index}"),
                    false,
                )
                .expect("component operation")
        })
        .collect::<Vec<_>>();
    for pair in operations.windows(2) {
        fragment
            .control(
                &pair[0],
                ApplicationWorkflowControlOutcome::Completed,
                &pair[1],
            )
            .expect("internal control");
    }
    let input = fragment
        .input_port("input", &operations[0])
        .expect("component input");
    let output = fragment
        .output_port("output", operations.last().expect("nonempty fragment"))
        .expect("component output");
    let fragment = fragment.finish().expect("component closes");
    let mut definition = ApplicationWorkflowDefinitionBuilder::<ReviewedGeometryWorkflow>::new(
        "repeated-solver-fragment",
        limits,
    )
    .expect("definition identity");
    let mut previous = None;
    for index in 0..occurrences {
        let expanded = definition
            .expand_component(&format!("fragment-{index:04}"), &fragment)
            .expect("bounded component expansion");
        let entry = expanded.input(&input).expect("exported input binds");
        let exit = expanded.output(&output).expect("exported output binds");
        if let Some(previous) = previous {
            definition.control(
                &previous,
                ApplicationWorkflowControlOutcome::Completed,
                &entry,
            );
        } else {
            definition.start(&entry);
        }
        previous = Some(exit);
    }
    let terminal = definition.terminal("done").expect("terminal node");
    definition.control(
        &previous.expect("nonempty component sequence"),
        ApplicationWorkflowControlOutcome::Completed,
        &terminal,
    );
    definition.finish().expect("repeated definition closes")
}

fn component_limits(occurrences: u16) -> ApplicationWorkflowComponentLimits {
    ApplicationWorkflowComponentLimits::new(
        occurrences,
        1,
        u32::from(occurrences) * u32::from(FRAGMENT_NODES),
        u32::from(occurrences) * u32::from(FRAGMENT_CONNECTIONS),
        u32::from(occurrences) * 2,
    )
    .expect("independent occurrence and provenance budgets")
}

fn qualify(occurrences: u16, key: u64) {
    let nodes = occurrences * FRAGMENT_NODES + 1;
    let connections = nodes - 1;
    let resources = WorthQueryApplicationWorkflowResourceCeiling::new(
        nodes,
        connections,
        connections,
        component_limits(occurrences),
        16 * 1024 * 1024,
        2,
        u32::from(nodes),
        256 * 1024,
    )
    .expect("finite workflow resources");
    let application =
        retain_workflow_with_resources(publish_on_first_program_for_geometry_scale(), resources);
    let runtime = application.runtime();
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(300),
        cancellation.token(),
    );
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let started_at = Instant::now();
    let authored = component_definition(occurrences);
    let authored_ms = started_at.elapsed().as_millis();
    let validated = authored
        .validate()
        .expect("the repeated component graph validates");
    assert_eq!(validated.nodes().len(), usize::from(nodes));
    assert_eq!(validated.connections().len(), usize::from(connections));
    assert_eq!(
        validated.component_expansions().len(),
        usize::from(occurrences)
    );
    let work = validated.validation_work();
    assert_eq!(work.provenance_records(), u64::from(occurrences));
    assert_eq!(work.indexed_nodes(), u64::from(nodes));
    assert_eq!(work.indexed_connections(), u64::from(connections));
    assert!(
        work.total_visits() <= u64::from(nodes) * 64,
        "repeated components must not cause broad validation scans: {work:?}"
    );
    let validated_ms = started_at.elapsed().as_millis();
    println!("repeated component nodes={nodes} authored_ms={authored_ms} validated_ms={validated_ms} validation_work={work:?}");
    let cost_scope = runtime
        .capture_certification_cost_scope(application.current_world())
        .expect("geometry branch remains measurable");
    let draft = runtime
        .request(&principal, &scope)
        .mutate(WorkflowDefinitionAuthoringIntent {
            input: WorkflowDefinitionAuthoringInput {
                identity: PART_IDENTITY.to_owned(),
                dimension: 8,
            },
        })
        .workflow(&application, component_definition(occurrences))
        .expect("the component binds to installed vocabulary");
    println!(
        "repeated component nodes={nodes} ordinary_draft_ms={}",
        started_at.elapsed().as_millis()
    );
    let publication_started_at = Instant::now();
    let publication = draft
        .publish(WorkflowDefinitionExpectedPredecessor::Absent)
        .idempotency(&key)
        .execute()
        .expect("the repeated definition prepares for publication");
    println!(
        "repeated component nodes={nodes} through_publication_ms={} publication_execute_ms={}",
        started_at.elapsed().as_millis(),
        publication_started_at.elapsed().as_millis()
    );
    let WorkflowDefinitionPublicationOutcome::Published(published) = publication else {
        panic!("expected repeated component publication: {publication:?}");
    };
    assert_eq!(
        published.definition().content_identity(),
        validated.content_identity()
    );
    let cost = runtime
        .observe_certification_cost(&cost_scope)
        .expect("publication cost remains measurable");
    let sharing = cost.relational().sharing_cost_delta();
    assert_eq!(sharing.copied_truth_bytes, 0);
    assert!(sharing.publication_new_authoritative_bytes > 0);
    assert!(
        sharing.publication_new_authoritative_bytes <= u64::from(nodes) * 32 * 1024,
        "authoritative publication bytes must remain proportional to expanded nodes"
    );
    assert!(
        sharing.publication_content_values_hashed <= u64::from(nodes) * 64,
        "native content hashing must remain proportional to expanded nodes"
    );
    let through_publication_ms = started_at.elapsed().as_millis();
    let before = runtime.workflow_compilation_reuse_counters();
    let cold_started_at = Instant::now();
    assert!(matches!(
        start_instance(&application, published.definition().clone(), key + 1)
            .expect("cold start fits its own operation budget"),
        WorkflowInstanceStartOutcome::Started(_)
    ));
    let cold_ms = cold_started_at.elapsed().as_millis();
    let cold = runtime.workflow_compilation_reuse_counters();
    assert_eq!(cold.cold_misses(), before.cold_misses() + 1);
    assert!(matches!(
        start_instance(&application, published.definition().clone(), key + 2)
            .expect("warm start reuses compiled meaning"),
        WorkflowInstanceStartOutcome::Started(_)
    ));
    let warm = runtime.workflow_compilation_reuse_counters();
    assert_eq!(warm.warm_hits(), cold.warm_hits() + 1);
    assert_eq!(warm.cold_misses(), cold.cold_misses());
    println!(
        "repeated component nodes={nodes} occurrences={occurrences} validated_ms={validated_ms} through_publication_ms={through_publication_ms} cold_ms={cold_ms} copied_truth_bytes={} new_authoritative_bytes={}",
        sharing.copied_truth_bytes,
        sharing.publication_new_authoritative_bytes,
    );
}

#[test]
fn repeated_component_100_nodes_publishes_and_starts() {
    qualify(11, 918_500);
}

#[test]
fn repeated_component_1000_nodes_publishes_and_starts() {
    qualify(111, 918_510);
}

#[test]
#[ignore = "scheduled public 10k-node repeated-component qualification"]
fn repeated_component_10000_nodes_publishes_and_starts() {
    qualify(1111, 918_520);
}
