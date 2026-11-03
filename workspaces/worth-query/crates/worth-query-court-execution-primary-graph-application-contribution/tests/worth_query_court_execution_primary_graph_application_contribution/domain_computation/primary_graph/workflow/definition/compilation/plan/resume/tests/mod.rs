use std::num::NonZeroU64;
use std::sync::Arc;

use worth_foundational::facade::BoundaryProtocolIdentity;
use worth_foundational::facade::BoundaryProtocolVersion;
use worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome;
use worth_query_declaration::facade::application_program::ApplicationWorkflowDefinitionContentIdentity;
use worth_query_declaration::facade::application_program::ApplicationWorkflowInboundWait;
use worth_query_declaration::facade::application_schema::ApplicationInboundOccurrenceLimits;
use worth_query_declaration::facade::application_schema::ApplicationInboundOccurrenceProtocol;
use worth_relational::facade::identity::PartitionId;

use worth_query_execution::domain_computation::primary_graph::workflow::definition::compilation::plan::CompiledWorkflowConnectionKind;
    use worth_query_execution::domain_computation::primary_graph::workflow::definition::compilation::plan::CompiledWorkflowDefinition;
    use worth_query_execution::domain_computation::primary_graph::workflow::definition::compilation::plan::CompiledWorkflowNode;
    use worth_query_execution::domain_computation::primary_graph::workflow::definition::compilation::plan::CompiledWorkflowNodeKind;
    use worth_relational::facade::identity::EntityId;
    use crate::domain_computation::primary_graph::workflow::definition::compilation::plan::resume::tests;
    use worth_query_execution::domain_computation::primary_graph::application_attempt::workflow_instance_observation::step_budget::WorkflowStepAllowance;
    use worth_query_execution::facade::domain_computation::WorthQueryApplicationAttemptDenialKind;
    use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::program_roster::rostered_program_revision;
    use worth_query_execution::domain_computation::primary_graph::workflow::definition::compilation::plan::CompiledWorkflowConnection;
    use worth_query_execution::domain_computation::primary_graph::workflow::definition::compilation::plan::dispatch::CompiledWorkflowDispatch;
    use worth_query_execution::domain_computation::primary_graph::workflow::definition::compilation::plan::CompiledWorkflowNodeKind;
    use worth_query_execution::domain_computation::primary_graph::workflow::definition::compilation::plan::CompiledWorkflowNodeMeaning;
    use worth_query_execution::domain_computation::primary_graph::workflow::definition::compilation::plan::CompiledWorkflowPublicationPlan;
    use worth_query_execution::domain_computation::primary_graph::workflow::definition::compilation::plan::CompiledWorkflowSemanticConnection;
    use worth_query_execution::domain_computation::primary_graph::workflow::instance::migration::admit_workflow_migration;
    use worth_query_execution::domain_computation::primary_graph::workflow::instance::migration::WorkflowSuccession;

#[test]
fn inbound_origin_blocks_migration_and_fork_resume_at_wait_but_allows_prior_operation() {
    let source = definition(100);
    for (succession, target) in [
        (WorkflowSuccession::Migration, definition(200)),
        (WorkflowSuccession::ForkContinuation, source.clone()),
    ] {
        let skipped = target.resumed_at("await").unwrap();
        let denial = admit_workflow_migration(
            succession,
            &source,
            &skipped,
            &[],
            &[],
            WorkflowStepAllowance::for_test(),
        )
        .expect_err("a successor cannot consume an origin it never ran");
        assert_eq!(
            denial.kind(),
            WorthQueryApplicationAttemptDenialKind::WorkflowInstanceMigrationUnmapped,
        );

        let prior = target.resumed_at("apply").unwrap();
        admit_workflow_migration(
            succession,
            &source,
            &prior,
            &[],
            &[],
            WorkflowStepAllowance::for_test(),
        )
        .expect("a successor that runs the origin before the wait remains lawful");
    }
}

pub fn definition(definition_slot: u64) -> CompiledWorkflowDefinition {
    let node_slot = definition_slot + 10;
    let entity = |slot| EntityId::new(PartitionId::new(1), slot, 1);
    let one = NonZeroU64::new(1).unwrap();
    let limits = ApplicationInboundOccurrenceLimits {
        maximum_envelope_bytes: one,
        maximum_verifier_work: one,
        maximum_payload_bytes: one,
        maximum_outstanding_dispatch_provenance: one,
        maximum_accepted_occurrences: one,
        maximum_accepted_bytes: one,
        maximum_concurrent_publications: one,
        maximum_discovery_work: one,
        replay_window_milliseconds: one,
        maximum_cleanup_work: one,
    };
    let kinds = [
            CompiledWorkflowNodeKind::Operation {
                operation: "apply".to_owned(),
                input_type: "payment".to_owned(),
                binding: None,
                requires_workflow_authority: true,
                workflow_execution_requirements: worth_query_declaration::facade::application_operation::ApplicationWorkflowExecutionRequirements::not_applicable(),
            },
            CompiledWorkflowNodeKind::AwaitInbound {
                origin: "apply".to_owned(),
                effect: "payment-settlement".to_owned(),
                protocol: ApplicationInboundOccurrenceProtocol::new(
                    BoundaryProtocolIdentity::new("worth.query.workflow.payment"),
                    BoundaryProtocolVersion::new(1),
                ),
                source_identity: "rail".to_owned(),
                limits,
                wait: ApplicationWorkflowInboundWait::UntilInstanceDeadline,
            },
            CompiledWorkflowNodeKind::Terminal,
        ];
    let paths = ["apply", "await", "done"];
    let nodes = paths
        .into_iter()
        .zip(kinds)
        .enumerate()
        .map(|(ordinal, (path, kind))| CompiledWorkflowNode {
            entity: entity(node_slot + ordinal as u64),
            meaning: Arc::new(CompiledWorkflowNodeMeaning {
                path: path.to_owned(),
                kind,
            }),
        })
        .collect::<Vec<_>>();
    let semantic_connections = (0..2)
        .map(|source| CompiledWorkflowSemanticConnection {
            source,
            target: source + 1,
            kind: Arc::new(CompiledWorkflowConnectionKind::Control(
                ApplicationWorkflowControlOutcome::Completed,
            )),
        })
        .collect::<Vec<_>>();
    let connections = semantic_connections
        .iter()
        .enumerate()
        .map(|(ordinal, semantic)| CompiledWorkflowConnection {
            entity: entity(definition_slot + ordinal as u64 + 1),
            source: nodes[semantic.source].entity(),
            target: nodes[semantic.target].entity(),
            kind: Arc::clone(&semantic.kind),
        })
        .collect::<Vec<_>>();
    CompiledWorkflowDefinition {
        publication: Arc::new(CompiledWorkflowPublicationPlan {
            lineage: entity(1),
            definition: entity(definition_slot),
            start: 0,
            node_ordinals: Arc::from(
                nodes
                    .iter()
                    .enumerate()
                    .map(|(ordinal, node)| (node.entity(), ordinal))
                    .collect::<Vec<_>>()
                    .into_boxed_slice(),
            ),
            nodes: nodes.into_boxed_slice(),
            connections: connections.into_boxed_slice(),
            global_actions: Arc::from([]),
            dispatch: Arc::new(CompiledWorkflowDispatch::build(3, &semantic_connections)),
        }),
        content_identity: ApplicationWorkflowDefinitionContentIdentity::from_recorded(
            &"00".repeat(32),
        )
        .unwrap(),
        program_revision: rostered_program_revision(),
        resume: None,
    }
}
