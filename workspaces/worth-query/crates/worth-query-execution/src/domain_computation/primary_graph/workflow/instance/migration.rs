//! The law an explicit instance migration satisfies before it publishes.
//!
//! A successor never inherits a proposal, assessment evidence, a join result
//! or an approval: it re-establishes each by running the nodes that produce
//! it. Every result a node it can run consumes must be produced before that
//! node wherever a fresh instance guarantees so; a loop back to the producer
//! never counts. Receipted effects are the only results that carry, as
//! history. Each maps by path to the same operation in the target, and the
//! successor can never run that operation, or any other with its meaning,
//! again. An approval that has not yet been consumed by its receipted
//! operation blocks migration; it settles under the source.
//!
//! A fork continuation obeys the same law for a fork's copy of an instance.
//! Its successor may keep the copied definition while the fork still holds
//! it current, since it continues the work on another branch rather than
//! moving it to a newer definition. New work never commits under a
//! definition the fork has superseded or retired.

use worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome;
use worth_relational::facade::identity::EntityId;

use super::super::definition::{
    CompiledWorkflowDefinition, CompiledWorkflowNode, CompiledWorkflowNodeKind,
};
use super::SettledWorkflowTransition;
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
};

/// A receipted transition the successor must carry: the source's own, or one
/// the source itself inherited. Either maps by the path of the node it ran.
#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct WorkflowPerformedEffect {
    pub(in crate::domain_computation::primary_graph) transition: EntityId,
    pub(in crate::domain_computation::primary_graph) path: String,
}

/// How a successor relates to the instance whose work it takes over.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum WorkflowSuccession {
    /// The source's own branch, onto a newer definition of its workflow.
    Migration,
    /// A fork's copy of an instance started on another branch, onto the
    /// fork's current definition of its workflow.
    ForkContinuation,
}

/// Admits `resumed`, the target definition as the successor runs it.
pub(in crate::domain_computation::primary_graph) fn admit_workflow_migration(
    succession: WorkflowSuccession,
    source: &CompiledWorkflowDefinition,
    resumed: &CompiledWorkflowDefinition,
    settled: &[SettledWorkflowTransition],
    performed: &[WorkflowPerformedEffect],
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    if resumed.lineage() != source.lineage() {
        return Err(unmapped("a successor continues the same workflow"));
    }
    if succession == WorkflowSuccession::Migration && resumed.definition() == source.definition() {
        return Err(unmapped(
            "a migration target is another definition of the same workflow",
        ));
    }
    deny_outstanding_approval(source, settled)?;
    let resume = resumed.start().entity();
    let runs = resumed.reachable_from(resume, None);
    for node in &runs {
        for consumed in resumed.consumed_sources(*node) {
            let producer = Some(consumed.entity());
            if resumed.reachable_from(resume, producer).contains(node)
                && !resumed
                    .reachable_from(resumed.definition_start(), producer)
                    .contains(node)
            {
                return Err(unmapped(
                    "the successor could run a node before the result it consumes",
                ));
            }
        }
    }
    let target = resumed;
    for effect in performed {
        let [performed_at] = source.nodes_with_path(&effect.path) else {
            return Err(unmapped("a performed effect is not at a source operation"));
        };
        if !matches!(
            performed_at.kind(),
            CompiledWorkflowNodeKind::Operation { .. }
        ) {
            return Err(unmapped("a performed effect is not at a source operation"));
        }
        let [mapped] = target.nodes_with_path(&effect.path) else {
            return Err(unmapped(
                "a performed effect has no operation at its path in the target",
            ));
        };
        if mapped.kind() != performed_at.kind() {
            return Err(unmapped(
                "a performed effect maps to a different operation in the target",
            ));
        }
        if runs
            .iter()
            .filter_map(|node| target.node(*node))
            .any(|node| node.kind() == performed_at.kind())
        {
            return Err(unmapped("the successor could run a performed effect again"));
        }
    }
    Ok(())
}

/// The same rule adoption custody applies: the latest decision approved and
/// no receipted settlement of an operation it authorizes has followed it.
fn deny_outstanding_approval(
    source: &CompiledWorkflowDefinition,
    settled: &[SettledWorkflowTransition],
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    let decisions = settled.iter().filter(|transition| {
        matches!(
            source
                .node(transition.node())
                .map(CompiledWorkflowNode::kind),
            Some(CompiledWorkflowNodeKind::Approval { .. })
        )
    });
    for decision in decisions {
        let latest = settled
            .iter()
            .filter(|transition| transition.node() == decision.node())
            .all(|transition| transition.occurrence() <= decision.occurrence());
        if !latest || decision.outcome() != ApplicationWorkflowControlOutcome::Approved {
            continue;
        }
        let outstanding = source
            .approval_authority_targets(decision.node())
            .any(|operation| {
                !settled.iter().any(|transition| {
                    transition.node() == operation.entity()
                        && transition.operation_receipt_identity().is_some()
                        && transition.occurrence() > decision.occurrence()
                })
            });
        if outstanding {
            return Err(WorthQueryApplicationAttemptDenial::new(
                WorthQueryApplicationAttemptDenialKind::WorkflowTransitionOperationUnsettled,
                "an approved operation must settle under the source before migration",
            ));
        }
    }
    Ok(())
}

fn unmapped(subject: &'static str) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::WorkflowInstanceMigrationUnmapped,
        subject,
    )
}
