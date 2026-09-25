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
        ApplicationWorkflowConnectionKind, ApplicationWorkflowControlOutcome,
        ApplicationWorkflowDefinitionBuilder, ApplicationWorkflowDefinitionLimits,
        ApplicationWorkflowEvidenceJoinPolicy, ApplicationWorkflowNodeKind,
        ApplicationWorkflowRetry, ApplicationWorkflowSubjectSelector, AuthoredWorkflowDefinition,
    },
};
use worth_query_installation::facade::WorthQueryApplicationWorkflowResourceCeiling;
use worth_query_replay::facade::WorthQueryCertificationCostRuntimeExt;

use super::bounded_dimension_model::{
    dimension_entry::PART_IDENTITY,
    host::publish_on_first_program_for_workflow_scale,
    operator_identity::authenticate_operator,
    schema::{PartDimensionConditionQuery, PartDimensionQuery},
    workflow::{
        retain_workflow_with_resources, start_instance, ReviewedGeometryWorkflow,
        WorkflowDefinitionAuthoringInput, WorkflowDefinitionAuthoringIntent,
        WorkflowDefinitionAuthoringOperation,
    },
};

const FRAGMENT_NODES: u16 = 9;
const FRAGMENT_CONNECTIONS: u16 = 16;

fn control_definition(occurrences: u16) -> AuthoredWorkflowDefinition<ReviewedGeometryWorkflow> {
    let nodes = occurrences * FRAGMENT_NODES + 1;
    let connections = occurrences * (FRAGMENT_CONNECTIONS + 1);
    let limits = ApplicationWorkflowDefinitionLimits::new(
        nodes,
        connections,
        connections,
        component_limits(occurrences),
        16 * 1024 * 1024,
    )
    .expect("finite control definition limits");
    let mut fragment =
        ApplicationWorkflowComponentBuilder::<ReviewedGeometryWorkflow>::new("control-fragment")
            .expect("component identity");
    let proposal = fragment
        .operation::<WorkflowDefinitionAuthoringOperation>("proposal", false)
        .expect("proposal operation");
    let condition = fragment
        .condition::<PartDimensionConditionQuery>("condition")
        .expect("typed condition");
    let geometry = fragment
        .assessment::<PartDimensionQuery>("geometry")
        .expect("resource assessment");
    let independent = fragment
        .assessment_for::<PartDimensionQuery>(
            "independent",
            ApplicationWorkflowSubjectSelector::related(),
        )
        .expect("related assessment");
    let join = fragment
        .evidence_join(
            "evidence",
            ApplicationWorkflowEvidenceJoinPolicy::AllRequiredPassing,
        )
        .expect("evidence join");
    let revise = fragment
        .operation::<WorkflowDefinitionAuthoringOperation>("revise", false)
        .expect("revision operation");
    let exit = fragment
        .operation::<WorkflowDefinitionAuthoringOperation>("exit", false)
        .expect("fragment exit");
    let checkpoint = fragment
        .operation::<WorkflowDefinitionAuthoringOperation>("checkpoint", false)
        .expect("fragment checkpoint");
    let last = fragment
        .operation::<WorkflowDefinitionAuthoringOperation>("last", false)
        .expect("fragment last operation");
    let retry = ApplicationWorkflowRetry::new(
        ApplicationWorkflowControlOutcome::Completed,
        "control-revision",
        2,
    )
    .expect("finite retry");
    fragment
        .control(
            &proposal,
            ApplicationWorkflowControlOutcome::Completed,
            &condition,
        )
        .unwrap()
        .control(
            &condition,
            ApplicationWorkflowControlOutcome::ConditionSatisfied,
            &geometry,
        )
        .unwrap()
        .control(
            &condition,
            ApplicationWorkflowControlOutcome::ConditionUnsatisfied,
            &exit,
        )
        .unwrap()
        .control(
            &geometry,
            ApplicationWorkflowControlOutcome::Completed,
            &independent,
        )
        .unwrap()
        .control(
            &independent,
            ApplicationWorkflowControlOutcome::Completed,
            &join,
        )
        .unwrap()
        .control(
            &join,
            ApplicationWorkflowControlOutcome::EvidenceSatisfied,
            &exit,
        )
        .unwrap()
        .control(
            &join,
            ApplicationWorkflowControlOutcome::EvidenceFailed,
            &revise,
        )
        .unwrap()
        .retry(&revise, retry, &proposal)
        .unwrap()
        .control(
            &revise,
            ApplicationWorkflowControlOutcome::RetryExhausted,
            &exit,
        )
        .unwrap()
        .control(
            &exit,
            ApplicationWorkflowControlOutcome::Completed,
            &checkpoint,
        )
        .unwrap()
        .control(
            &checkpoint,
            ApplicationWorkflowControlOutcome::Completed,
            &last,
        )
        .unwrap()
        .condition_subject(&proposal, &condition)
        .unwrap()
        .proposal_for_assessment(&proposal, &geometry)
        .unwrap()
        .proposal_for_assessment(&proposal, &independent)
        .unwrap()
        .assessment_evidence(&geometry, &join)
        .unwrap()
        .assessment_evidence(&independent, &join)
        .unwrap();
    let input = fragment.input_port("input", &proposal).unwrap();
    let output = fragment.output_port("output", &last).unwrap();
    let fragment = fragment.finish().expect("control fragment closes");
    let mut definition = ApplicationWorkflowDefinitionBuilder::<ReviewedGeometryWorkflow>::new(
        "repeated-control-fragment",
        limits,
    )
    .expect("definition identity");
    let mut previous = None;
    for index in 0..occurrences {
        let expanded = definition
            .expand_component(&format!("control-{index:04}"), &fragment)
            .expect("bounded control expansion");
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
        &previous.expect("nonempty control sequence"),
        ApplicationWorkflowControlOutcome::Completed,
        &terminal,
    );
    definition.finish().expect("control definition closes")
}

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

fn qualify(occurrences: u16, key: u64) {
    let nodes = occurrences * FRAGMENT_NODES + 1;
    let connections = occurrences * (FRAGMENT_CONNECTIONS + 1);
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
    .expect("finite control workflow resources");
    let application =
        retain_workflow_with_resources(publish_on_first_program_for_workflow_scale(), resources);
    let runtime = application.runtime();
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(
        Instant::now() + Duration::from_secs(300),
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
            .filter(|node| matches!(node.kind(), ApplicationWorkflowNodeKind::Condition(condition) if condition.query_type() == TypeId::of::<PartDimensionConditionQuery>()))
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
                .filter(|node| matches!(node.kind(), ApplicationWorkflowNodeKind::Assessment(assessment) if assessment.query_type() == TypeId::of::<PartDimensionQuery>() && assessment.subject() == &subject))
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
    let cost_scope = runtime
        .capture_certification_cost_scope(application.current_world())
        .expect("the branch remains measurable");
    let publication_at = Instant::now();
    let publication = runtime
        .request(&principal, &scope)
        .mutate(WorkflowDefinitionAuthoringIntent {
            input: WorkflowDefinitionAuthoringInput {
                identity: PART_IDENTITY.to_owned(),
                dimension: 8,
            },
        })
        .workflow(&application, control_definition(occurrences))
        .expect("control graph binds to installed vocabulary")
        .publish(WorkflowDefinitionExpectedPredecessor::Absent)
        .idempotency(&key)
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
    let cost = runtime
        .observe_certification_cost(&cost_scope)
        .expect("publication cost remains measurable");
    let sharing = cost.relational().sharing_cost_delta();
    assert_eq!(sharing.copied_truth_bytes, 0);
    assert!(
        sharing.publication_new_authoritative_bytes <= u64::from(nodes) * 40 * 1024,
        "control publication bytes={} nodes={nodes}",
        sharing.publication_new_authoritative_bytes
    );
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
}

#[test]
fn repeated_control_fragment_100_nodes_publishes_and_starts() {
    qualify(11, 918_600);
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
