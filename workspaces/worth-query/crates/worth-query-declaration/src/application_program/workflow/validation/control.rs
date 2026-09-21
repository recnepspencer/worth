use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::application_program::workflow::{
    ApplicationWorkflowConnection, ApplicationWorkflowConnectionKind,
    ApplicationWorkflowControlOutcome, ApplicationWorkflowNode, ApplicationWorkflowNodeIdentity,
    ApplicationWorkflowNodeKind,
};

use super::{denial, ApplicationWorkflowValidationDenial, ApplicationWorkflowValidationDenialKind};

pub(super) fn validate(
    start: &ApplicationWorkflowNodeIdentity,
    nodes: &BTreeMap<&ApplicationWorkflowNodeIdentity, &ApplicationWorkflowNode>,
    connections: &[ApplicationWorkflowConnection],
) -> Result<(), ApplicationWorkflowValidationDenial> {
    if !nodes
        .values()
        .any(|node| matches!(node.kind(), ApplicationWorkflowNodeKind::Terminal))
    {
        return Err(denial(
            ApplicationWorkflowValidationDenialKind::MissingTerminal,
            start.as_str(),
        ));
    }
    require_complete_outcomes(nodes, connections)?;
    let mut indegree = nodes
        .keys()
        .map(|identity| ((*identity).clone(), 0_u16))
        .collect::<BTreeMap<_, _>>();
    let mut outgoing =
        BTreeMap::<ApplicationWorkflowNodeIdentity, Vec<ApplicationWorkflowNodeIdentity>>::new();
    for connection in connections {
        if matches!(
            connection.kind(),
            ApplicationWorkflowConnectionKind::Control(_)
        ) {
            outgoing
                .entry(connection.source().clone())
                .or_default()
                .push(connection.target().clone());
            *indegree
                .get_mut(connection.target())
                .expect("endpoint validation ran") += 1;
        }
    }
    require_reachable(start, nodes, &outgoing)?;
    require_acyclic(start, nodes.len(), indegree, &outgoing)
}

fn require_complete_outcomes(
    nodes: &BTreeMap<&ApplicationWorkflowNodeIdentity, &ApplicationWorkflowNode>,
    connections: &[ApplicationWorkflowConnection],
) -> Result<(), ApplicationWorkflowValidationDenial> {
    for (identity, node) in nodes {
        let outcomes = connections
            .iter()
            .filter_map(|connection| {
                (connection.source() == *identity)
                    .then(|| match connection.kind() {
                        ApplicationWorkflowConnectionKind::Control(outcome) => Some(outcome),
                        ApplicationWorkflowConnectionKind::Data(_) => None,
                    })
                    .flatten()
            })
            .collect::<Vec<_>>();
        let expected: &[ApplicationWorkflowControlOutcome] = match node.kind() {
            ApplicationWorkflowNodeKind::Approval(_) => &[
                ApplicationWorkflowControlOutcome::Approved,
                ApplicationWorkflowControlOutcome::Rejected,
            ],
            ApplicationWorkflowNodeKind::Terminal => {
                if outcomes.is_empty() {
                    continue;
                }
                return Err(denial(
                    ApplicationWorkflowValidationDenialKind::TerminalHasSuccessor,
                    identity.as_str(),
                ));
            }
            _ => &[ApplicationWorkflowControlOutcome::Completed],
        };
        for expected_outcome in expected {
            match outcomes
                .iter()
                .filter(|outcome| *outcome == expected_outcome)
                .count()
            {
                0 => {
                    return Err(denial(
                        ApplicationWorkflowValidationDenialKind::MissingControlOutcome,
                        identity.as_str(),
                    ));
                }
                1 => {}
                _ => {
                    return Err(denial(
                        ApplicationWorkflowValidationDenialKind::AmbiguousControlOutcome,
                        identity.as_str(),
                    ));
                }
            }
        }
        if outcomes.iter().any(|outcome| !expected.contains(outcome)) {
            return Err(denial(
                ApplicationWorkflowValidationDenialKind::UnexpectedControlOutcome,
                identity.as_str(),
            ));
        }
    }
    Ok(())
}

fn require_reachable(
    start: &ApplicationWorkflowNodeIdentity,
    nodes: &BTreeMap<&ApplicationWorkflowNodeIdentity, &ApplicationWorkflowNode>,
    outgoing: &BTreeMap<ApplicationWorkflowNodeIdentity, Vec<ApplicationWorkflowNodeIdentity>>,
) -> Result<(), ApplicationWorkflowValidationDenial> {
    let mut reachable = BTreeSet::new();
    let mut queue = VecDeque::from([start.clone()]);
    while let Some(current) = queue.pop_front() {
        if reachable.insert(current.clone()) {
            queue.extend(outgoing.get(&current).into_iter().flatten().cloned());
        }
    }
    if let Some(unreachable) = nodes.keys().find(|identity| !reachable.contains(*identity)) {
        return Err(denial(
            ApplicationWorkflowValidationDenialKind::UnreachableNode,
            unreachable.as_str(),
        ));
    }
    Ok(())
}

fn require_acyclic(
    start: &ApplicationWorkflowNodeIdentity,
    node_count: usize,
    mut indegree: BTreeMap<ApplicationWorkflowNodeIdentity, u16>,
    outgoing: &BTreeMap<ApplicationWorkflowNodeIdentity, Vec<ApplicationWorkflowNodeIdentity>>,
) -> Result<(), ApplicationWorkflowValidationDenial> {
    let mut roots = indegree
        .iter()
        .filter_map(|(identity, degree)| (*degree == 0).then_some(identity.clone()))
        .collect::<VecDeque<_>>();
    let mut visited = 0_usize;
    while let Some(current) = roots.pop_front() {
        visited += 1;
        for target in outgoing.get(&current).into_iter().flatten() {
            let degree = indegree.get_mut(target).expect("endpoint validation ran");
            *degree -= 1;
            if *degree == 0 {
                roots.push_back(target.clone());
            }
        }
    }
    if visited != node_count {
        return Err(denial(
            ApplicationWorkflowValidationDenialKind::ControlCycle,
            start.as_str(),
        ));
    }
    Ok(())
}
