//! The graph questions migration asks of a target definition.

use std::collections::BTreeSet;

use worth_relational::facade::identity::EntityId;

use super::{
    CompiledWorkflowConnectionKind, CompiledWorkflowDefinition, CompiledWorkflowNode,
    CompiledWorkflowNodeKind,
};

impl CompiledWorkflowDefinition {
    /// The same definition read from `path` as its first node. Progress,
    /// replay and history of a migration successor all begin there.
    pub(in crate::domain_computation::primary_graph) fn resumed_at(
        &self,
        path: &str,
    ) -> Option<Self> {
        let [node] = self.nodes_with_path(path) else {
            return None;
        };
        let ordinal = self.node_ordinal(node.entity())?;
        Some(Self {
            resume: Some(ordinal),
            ..self.clone()
        })
    }

    /// The node a migration successor began at, when it did not begin at the
    /// definition's start.
    pub(in crate::domain_computation::primary_graph) fn resumed_path(&self) -> Option<&str> {
        self.resume
            .map(|ordinal| self.publication.nodes[ordinal].path())
    }

    /// The node a fresh instance of this definition begins at, whatever
    /// node this compilation resumes at.
    pub(in crate::domain_computation::primary_graph) fn definition_start(&self) -> EntityId {
        self.publication.nodes[self.publication.start].entity()
    }

    /// Every node control or retry can run from `start`, including `start`,
    /// without ever running `avoiding`. Avoiding `start` itself reaches
    /// nothing.
    pub(in crate::domain_computation::primary_graph) fn reachable_from(
        &self,
        start: EntityId,
        avoiding: Option<EntityId>,
    ) -> BTreeSet<EntityId> {
        if avoiding == Some(start) {
            return BTreeSet::new();
        }
        let mut reached = BTreeSet::from([start]);
        let mut pending = vec![start];
        while let Some(source) = pending.pop() {
            for edge in self.outgoing(source) {
                let runs = matches!(
                    self.publication.connections[edge.connection].kind.as_ref(),
                    CompiledWorkflowConnectionKind::Control(_)
                        | CompiledWorkflowConnectionKind::Retry { .. }
                );
                let peer = self.publication.nodes[edge.peer].entity();
                if runs && avoiding != Some(peer) && reached.insert(peer) {
                    pending.push(peer);
                }
            }
        }
        reached
    }

    /// Every node whose result `target` consumes, whatever the data flow.
    pub(in crate::domain_computation::primary_graph) fn consumed_sources(
        &self,
        target: EntityId,
    ) -> Option<impl Iterator<Item = &CompiledWorkflowNode>> {
        let inbound_origin = match self.node(target)?.kind() {
            CompiledWorkflowNodeKind::AwaitInbound { origin, .. } => {
                let [operation] = self.nodes_with_path(origin) else {
                    return None;
                };
                if !matches!(operation.kind(), CompiledWorkflowNodeKind::Operation { .. }) {
                    return None;
                }
                Some(operation)
            }
            _ => None,
        };
        Some(
            self.incoming(target)
                .filter_map(move |edge| {
                    match self.publication.connections[edge.connection].kind.as_ref() {
                        CompiledWorkflowConnectionKind::Data(_) => {
                            Some(&self.publication.nodes[edge.peer])
                        }
                        _ => None,
                    }
                })
                .chain(inbound_origin),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::{num::NonZeroU64, sync::Arc};

    use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
    use worth_query_declaration::facade::{
        application_program::{
            ApplicationWorkflowControlOutcome, ApplicationWorkflowDefinitionContentIdentity,
            ApplicationWorkflowInboundWait,
        },
        application_schema::{
            ApplicationInboundOccurrenceLimits, ApplicationInboundOccurrenceProtocol,
        },
    };
    use worth_relational::facade::identity::PartitionId;

    use super::*;
    use crate::domain_computation::primary_graph::{
        application_attempt::{WorkflowStepAllowance, WorthQueryApplicationAttemptDenialKind},
        tests::fixture::rostered_program_revision,
        workflow::{
            definition::compilation::plan::{
                CompiledWorkflowConnection, CompiledWorkflowDispatch, CompiledWorkflowNodeKind,
                CompiledWorkflowNodeMeaning, CompiledWorkflowPublicationPlan,
                CompiledWorkflowSemanticConnection,
            },
            instance::{admit_workflow_migration, WorkflowSuccession},
        },
    };

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

    fn definition(definition_slot: u64) -> CompiledWorkflowDefinition {
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
}
