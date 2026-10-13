//! Repeated Phase 3 control fragments at exact 100/1k/10k expanded-node sizes.

use std::{
    any::TypeId,
    time::{Duration, Instant},
};

use worth_query_host::facade::{
    admission::authenticated_principal::{WorthQueryCancellationSource, WorthQueryRequestScope},
    application_entry::{
        WorkflowDefinitionExpectedPredecessor, WorkflowDefinitionPublicationOutcome,
        WorkflowInstanceStartOutcome, WorthQueryApplicationRequestExt,
    },
    declaration::application_program::{
        ApplicationWorkflowComponentBuilder, ApplicationWorkflowComponentLimits,
        ApplicationWorkflowConditionOperands, ApplicationWorkflowConnectionKind,
        ApplicationWorkflowControlOutcome, ApplicationWorkflowDefinitionBuilder,
        ApplicationWorkflowDefinitionLimits, ApplicationWorkflowEvidenceJoinPolicy,
        ApplicationWorkflowNodeKind, ApplicationWorkflowRetry, ApplicationWorkflowSubjectSelector,
        AuthoredWorkflowDefinition,
    },
};
use worth_query_installation::facade::WorthQueryApplicationWorkflowResourceCeiling;
use worth_query_replay::facade::WorthQueryCertificationCostRuntimeExt;

use super::document_retention_model::{
    host::publish_on_first_program_for_workflow_scale,
    operator_identity::authenticate_operator,
    retention_entry::DOCUMENT_IDENTITY,
    schema::{DocumentRetentionConditionQuery, DocumentRetentionQuery},
    workflow::{
        retain_workflow_with_resources, start_instance, ReviewedDocumentWorkflow,
        WorkflowDefinitionAuthoringInput, WorkflowDefinitionAuthoringIntent,
        WorkflowDefinitionAuthoringOperation,
    },
};

const FRAGMENT_NODES: u16 = 9;
const FRAGMENT_CONNECTIONS: u16 = 16;

#[path = "workflow_control_scale/control_definition.rs"]
mod control_definition;
use control_definition::control_definition;

fn component_limits(occurrences: u16) -> ApplicationWorkflowComponentLimits {
    ApplicationWorkflowComponentLimits::new(
        occurrences,
        1,
        u32::from(occurrences) * u32::from(FRAGMENT_NODES),
        u32::from(occurrences) * u32::from(FRAGMENT_CONNECTIONS),
        u32::from(occurrences) * 2,
    )
    .expect("independent occurrence and provenance bounds")
}

/// Qualifies one expansion and returns its expanded node count and the new
/// authoritative bytes its definition publication wrote.
fn qualify(occurrences: u16, key: u64) -> (u16, u64) {
    let nodes = occurrences * FRAGMENT_NODES + 1;
    let connections = occurrences * (FRAGMENT_CONNECTIONS + 1);
    let resources = WorthQueryApplicationWorkflowResourceCeiling::new(
        nodes,
        connections,
        connections,
        component_limits(occurrences),
        super::document_retention_model::host::WORKFLOW_DEFINITION_BYTES,
        2,
        u32::from(nodes),
        256 * 1024,
    )
    .expect("finite control workflow resources");
    let application = retain_workflow_with_resources(
        publish_on_first_program_for_workflow_scale(nodes, connections),
        resources,
    );
    let runtime = application.runtime();
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(
        Some(Instant::now() + Duration::from_secs(300)),
        cancellation.token(),
    );
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let authored_at = Instant::now();
    let authored = control_definition(occurrences);
    let authored_ms = authored_at.elapsed().as_millis();
    let validation_at = Instant::now();
    let validated = authored.validate().expect("control expansion validates");
    let validation_ms = validation_at.elapsed().as_millis();
    assert_eq!(validated.nodes().len(), usize::from(nodes));
    assert_eq!(validated.connections().len(), usize::from(connections));
    assert_eq!(
        validated.component_expansions().len(),
        usize::from(occurrences)
    );
    assert_eq!(
        validated
            .nodes()
            .iter()
            .filter(|node| matches!(node.kind(), ApplicationWorkflowNodeKind::Condition(condition) if condition.operands().iter().any(|operand| operand.query().query_type() == TypeId::of::<DocumentRetentionConditionQuery>())))
            .count(),
        usize::from(occurrences)
    );
    assert_eq!(
        validated
            .nodes()
            .iter()
            .filter(|node| matches!(
                node.kind(),
                ApplicationWorkflowNodeKind::EvidenceJoin(
                    ApplicationWorkflowEvidenceJoinPolicy::AllRequiredPassing
                )
            ))
            .count(),
        usize::from(occurrences)
    );
    for subject in [
        ApplicationWorkflowSubjectSelector::Resource,
        ApplicationWorkflowSubjectSelector::Related,
    ] {
        assert_eq!(
            validated
                .nodes()
                .iter()
                .filter(|node| matches!(node.kind(), ApplicationWorkflowNodeKind::Assessment(assessment) if assessment.query_type() == TypeId::of::<DocumentRetentionQuery>() && assessment.subject() == &subject))
                .count(),
            usize::from(occurrences)
        );
    }
    assert_eq!(
        validated
            .connections()
            .iter()
            .filter(|connection| matches!(
                connection.kind(),
                ApplicationWorkflowConnectionKind::Retry(retry)
                    if retry.trigger() == ApplicationWorkflowControlOutcome::Completed
                        && retry.reason() == "control-revision"
                        && retry.maximum_attempts() == 2
            ))
            .count(),
        usize::from(occurrences)
    );
    let work = validated.validation_work();
    assert_eq!(work.provenance_records(), u64::from(occurrences));
    assert_eq!(work.indexed_nodes(), u64::from(nodes));
    assert_eq!(work.indexed_connections(), u64::from(connections));
    assert!(work.total_visits() <= u64::from(nodes) * 64, "{work:?}");
    let cost_scope = application
        .with_host_advancement(|host_phase| {
            runtime
                .capture_certification_cost_scope(&host_phase, application.current_world())
                .expect("the branch remains measurable")
        })
        .expect("installed host call opens");
    let publication_at = Instant::now();
    let publication = runtime
        .with_host_advancement(|host_phase| {
            runtime
                .request(&principal, &scope)
                .mutate(WorkflowDefinitionAuthoringIntent {
                    input: WorkflowDefinitionAuthoringInput {
                        identity: DOCUMENT_IDENTITY.to_owned(),
                        retention_days: 8,
                    },
                })
                .workflow(&host_phase, &application, control_definition(occurrences))
                .expect("control graph binds to installed vocabulary")
                .publish(WorkflowDefinitionExpectedPredecessor::Absent)
                .idempotency(&key)
        })
        .expect("installed host call opens")
        .execute()
        .expect("control definition publishes");
    let publication_ms = publication_at.elapsed().as_millis();
    let WorkflowDefinitionPublicationOutcome::Published(published) = publication else {
        panic!("expected control publication: {publication:?}");
    };
    assert_eq!(
        published.definition().content_identity(),
        validated.content_identity()
    );
    let cost = application
        .with_host_advancement(|host_phase| {
            runtime
                .observe_certification_cost(&host_phase, &cost_scope)
                .expect("publication cost remains measurable")
        })
        .expect("installed host call opens");
    let sharing = cost.relational().sharing_cost_delta();
    assert_eq!(sharing.copied_truth_bytes, 0);
    let before = runtime.workflow_compilation_reuse_counters();
    let cold_at = Instant::now();
    assert!(matches!(
        start_instance(&application, published.definition().clone(), key + 1)
            .expect("cold control start fits admission"),
        WorkflowInstanceStartOutcome::Started(_)
    ));
    let cold_ms = cold_at.elapsed().as_millis();
    let cold = runtime.workflow_compilation_reuse_counters();
    assert_eq!(cold.cold_misses(), before.cold_misses() + 1);
    let warm_at = Instant::now();
    assert!(matches!(
        start_instance(&application, published.definition().clone(), key + 2)
            .expect("warm control start reuses compiled meaning"),
        WorkflowInstanceStartOutcome::Started(_)
    ));
    let warm_ms = warm_at.elapsed().as_millis();
    let warm = runtime.workflow_compilation_reuse_counters();
    assert_eq!(warm.warm_hits(), cold.warm_hits() + 1);
    assert_eq!(warm.cold_misses(), cold.cold_misses());
    println!(
        "control nodes={nodes} connections={connections} authored_ms={authored_ms} validation_ms={validation_ms} publication_ms={publication_ms} cold_ms={cold_ms} warm_ms={warm_ms} validation_visits={} retry_visits={} copied_truth_bytes={} new_authoritative_bytes={}",
        work.total_visits(),
        work.retry_reachability(),
        sharing.copied_truth_bytes,
        sharing.publication_new_authoritative_bytes,
    );
    (nodes, sharing.publication_new_authoritative_bytes)
}

/// Publication bytes grow linearly with the expanded graph: doubling the
/// occurrences keeps bytes per node within a quarter of the smaller run's.
fn assert_bytes_per_node_stay_flat(
    (nodes, bytes): (u16, u64),
    (doubled, doubled_bytes): (u16, u64),
) {
    assert!(
        u128::from(doubled_bytes) * u128::from(nodes) * 4
            <= u128::from(bytes) * u128::from(doubled) * 5,
        "bytes per node grew: {bytes} at {nodes} nodes, {doubled_bytes} at {doubled} nodes"
    );
}

#[test]
fn repeated_control_fragment_100_nodes_publishes_and_starts_with_flat_bytes_per_node() {
    assert_bytes_per_node_stay_flat(qualify(11, 918_600), qualify(22, 918_630));
}

#[test]
fn repeated_control_fragment_1000_nodes_publishes_and_starts() {
    qualify(111, 918_610);
}

#[test]
#[ignore = "scheduled public 10k-node control-flow qualification"]
fn repeated_control_fragment_10000_nodes_publishes_and_starts() {
    qualify(1111, 918_620);
}
