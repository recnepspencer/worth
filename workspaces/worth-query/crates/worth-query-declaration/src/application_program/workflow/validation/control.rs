use std::{collections::VecDeque, mem::size_of};

use crate::application_program::workflow::{
    ApplicationWorkflowConnectionKind, ApplicationWorkflowControlOutcome, ApplicationWorkflowNode,
    ApplicationWorkflowNodeKind,
};

use super::{
    denial,
    index::{NodeIndex, ValidationGraph},
    ApplicationWorkflowValidationDenial, ApplicationWorkflowValidationDenialKind,
    ValidationWorkMeter,
};

pub(super) struct ControlProof {
    pub(super) topological_order: Vec<NodeIndex>,
}

pub(super) fn validate(
    start: NodeIndex,
    graph: &ValidationGraph<'_>,
    work: &mut ValidationWorkMeter,
) -> Result<ControlProof, ApplicationWorkflowValidationDenial> {
    if !graph
        .nodes()
        .any(|(_, node)| matches!(node.kind(), ApplicationWorkflowNodeKind::Terminal))
    {
        return Err(denial(
            ApplicationWorkflowValidationDenialKind::MissingTerminal,
            graph.node(start).identity().as_str(),
        ));
    }
    require_complete_outcomes(graph, work)?;
    require_bounded_back_edges(graph, work)?;
    require_reachable(start, graph, work)?;
    let topological_order = require_acyclic(start, graph, work)?;
    Ok(ControlProof { topological_order })
}

fn require_complete_outcomes(
    graph: &ValidationGraph<'_>,
    work: &mut ValidationWorkMeter,
) -> Result<(), ApplicationWorkflowValidationDenial> {
    for (index, node) in graph.nodes() {
        work.visit_control_node();
        validate_node_outcomes(node, graph.summary(index))?;
    }
    Ok(())
}

fn validate_node_outcomes(
    node: &ApplicationWorkflowNode,
    summary: &super::index::NodeConnectionSummary,
) -> Result<(), ApplicationWorkflowValidationDenial> {
    let Some(expected) = expected_outcomes(node.kind()) else {
        if summary.has_any_control_successor() {
            return Err(denial(
                ApplicationWorkflowValidationDenialKind::TerminalHasSuccessor,
                node.identity().as_str(),
            ));
        }
        return Ok(());
    };
    if summary.retry_count() > 1 {
        return Err(outcome_denial(
            ApplicationWorkflowValidationDenialKind::AmbiguousControlOutcome,
            node,
        ));
    }
    let retry_trigger = summary.retry_trigger();
    if retry_trigger.is_some_and(|trigger| !expected.contains(&trigger)) {
        return Err(outcome_denial(
            ApplicationWorkflowValidationDenialKind::UnexpectedControlOutcome,
            node,
        ));
    }
    for expected_outcome in expected {
        let count = summary.outcome(*expected_outcome)
            + usize::from(retry_trigger == Some(*expected_outcome));
        if count != 1 {
            return Err(outcome_denial(
                if count == 0 {
                    ApplicationWorkflowValidationDenialKind::MissingControlOutcome
                } else {
                    ApplicationWorkflowValidationDenialKind::AmbiguousControlOutcome
                },
                node,
            ));
        }
    }
    if retry_trigger.is_some()
        && summary.outcome(ApplicationWorkflowControlOutcome::RetryExhausted) != 1
    {
        let count = summary.outcome(ApplicationWorkflowControlOutcome::RetryExhausted);
        return Err(outcome_denial(
            if count == 0 {
                ApplicationWorkflowValidationDenialKind::MissingControlOutcome
            } else {
                ApplicationWorkflowValidationDenialKind::AmbiguousControlOutcome
            },
            node,
        ));
    }
    if ALL_OUTCOMES.iter().any(|outcome| {
        summary.outcome(*outcome) != 0
            && !expected.contains(outcome)
            && !(retry_trigger.is_some()
                && *outcome == ApplicationWorkflowControlOutcome::RetryExhausted)
    }) {
        return Err(outcome_denial(
            ApplicationWorkflowValidationDenialKind::UnexpectedControlOutcome,
            node,
        ));
    }
    Ok(())
}

fn expected_outcomes(
    kind: &ApplicationWorkflowNodeKind,
) -> Option<&'static [ApplicationWorkflowControlOutcome]> {
    match kind {
        ApplicationWorkflowNodeKind::Approval(_) => Some(&[
            ApplicationWorkflowControlOutcome::Approved,
            ApplicationWorkflowControlOutcome::Rejected,
        ]),
        ApplicationWorkflowNodeKind::EvidenceJoin(_) => Some(&[
            ApplicationWorkflowControlOutcome::EvidenceSatisfied,
            ApplicationWorkflowControlOutcome::EvidenceFailed,
        ]),
        ApplicationWorkflowNodeKind::Condition(_) => Some(&[
            ApplicationWorkflowControlOutcome::ConditionSatisfied,
            ApplicationWorkflowControlOutcome::ConditionUnsatisfied,
        ]),
        ApplicationWorkflowNodeKind::Operation { .. }
        | ApplicationWorkflowNodeKind::Assessment(_) => {
            Some(&[ApplicationWorkflowControlOutcome::Completed])
        }
        ApplicationWorkflowNodeKind::Terminal => None,
    }
}

fn outcome_denial(
    kind: ApplicationWorkflowValidationDenialKind,
    node: &ApplicationWorkflowNode,
) -> ApplicationWorkflowValidationDenial {
    denial(kind, node.identity().as_str())
}

fn require_bounded_back_edges(
    graph: &ValidationGraph<'_>,
    work: &mut ValidationWorkMeter,
) -> Result<(), ApplicationWorkflowValidationDenial> {
    let mut generations = vec![0_u32; graph.node_count()];
    let mut queue = VecDeque::with_capacity(graph.node_count());
    let mut generation = 0_u32;
    for connection in graph.connections() {
        if !matches!(
            connection.connection.kind_ref(),
            ApplicationWorkflowConnectionKind::Retry(_)
        ) {
            continue;
        }
        generation += 1;
        generations[connection.target] = generation;
        queue.push_back(connection.target);
        while let Some(current) = queue.pop_front() {
            work.visit_retry_reachability();
            for &successor in graph.control_successors(current) {
                work.visit_retry_reachability();
                if generations[successor] != generation {
                    generations[successor] = generation;
                    queue.push_back(successor);
                }
            }
        }
        if generations[connection.source] != generation {
            return Err(denial(
                ApplicationWorkflowValidationDenialKind::ControlCycle,
                connection.connection.source().as_str(),
            ));
        }
    }
    work.observe_index_bytes(
        graph
            .retained_bytes()
            .saturating_add(generations.capacity() * size_of::<u32>())
            .saturating_add(queue.capacity() * size_of::<NodeIndex>()),
    );
    Ok(())
}

fn require_reachable(
    start: NodeIndex,
    graph: &ValidationGraph<'_>,
    work: &mut ValidationWorkMeter,
) -> Result<(), ApplicationWorkflowValidationDenial> {
    let mut reachable = vec![false; graph.node_count()];
    let mut queue = VecDeque::with_capacity(graph.node_count());
    reachable[start] = true;
    queue.push_back(start);
    while let Some(current) = queue.pop_front() {
        work.visit_control_node();
        for &successor in graph.control_successors(current) {
            work.visit_control_edge();
            if !reachable[successor] {
                reachable[successor] = true;
                queue.push_back(successor);
            }
        }
    }
    work.observe_index_bytes(
        graph
            .retained_bytes()
            .saturating_add(reachable.capacity() * size_of::<bool>())
            .saturating_add(queue.capacity() * size_of::<NodeIndex>()),
    );
    if let Some((_, node)) = graph.nodes().find(|(index, _)| !reachable[*index]) {
        return Err(denial(
            ApplicationWorkflowValidationDenialKind::UnreachableNode,
            node.identity().as_str(),
        ));
    }
    Ok(())
}

fn require_acyclic(
    start: NodeIndex,
    graph: &ValidationGraph<'_>,
    work: &mut ValidationWorkMeter,
) -> Result<Vec<NodeIndex>, ApplicationWorkflowValidationDenial> {
    let mut indegree = (0..graph.node_count())
        .map(|index| graph.control_predecessors(index).len())
        .collect::<Vec<_>>();
    let mut roots = indegree
        .iter()
        .enumerate()
        .filter_map(|(index, degree)| (*degree == 0).then_some(index))
        .collect::<VecDeque<_>>();
    let mut order = Vec::with_capacity(graph.node_count());
    while let Some(current) = roots.pop_front() {
        work.visit_control_node();
        order.push(current);
        for &target in graph.control_successors(current) {
            work.visit_control_edge();
            indegree[target] -= 1;
            if indegree[target] == 0 {
                roots.push_back(target);
            }
        }
    }
    work.observe_index_bytes(
        graph
            .retained_bytes()
            .saturating_add(indegree.capacity() * size_of::<usize>())
            .saturating_add(roots.capacity() * size_of::<NodeIndex>())
            .saturating_add(order.capacity() * size_of::<NodeIndex>()),
    );
    if order.len() != graph.node_count() {
        return Err(denial(
            ApplicationWorkflowValidationDenialKind::ControlCycle,
            graph.node(start).identity().as_str(),
        ));
    }
    Ok(order)
}

const ALL_OUTCOMES: [ApplicationWorkflowControlOutcome; 9] = [
    ApplicationWorkflowControlOutcome::Completed,
    ApplicationWorkflowControlOutcome::Approved,
    ApplicationWorkflowControlOutcome::Rejected,
    ApplicationWorkflowControlOutcome::EvidenceSatisfied,
    ApplicationWorkflowControlOutcome::EvidenceFailed,
    ApplicationWorkflowControlOutcome::ConditionSatisfied,
    ApplicationWorkflowControlOutcome::ConditionUnsatisfied,
    ApplicationWorkflowControlOutcome::RetryExhausted,
    ApplicationWorkflowControlOutcome::NavigatedBack,
];
