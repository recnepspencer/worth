use std::collections::{BTreeMap, BTreeSet};

use crate::application_program::workflow::{
    ApplicationWorkflowConnection, ApplicationWorkflowConnectionKind, ApplicationWorkflowDataFlow,
    ApplicationWorkflowNode, ApplicationWorkflowNodeIdentity, ApplicationWorkflowNodeKind,
};

use super::{denial, ApplicationWorkflowValidationDenial, ApplicationWorkflowValidationDenialKind};

pub(super) fn validate(
    nodes: &BTreeMap<&ApplicationWorkflowNodeIdentity, &ApplicationWorkflowNode>,
    connections: &[ApplicationWorkflowConnection],
) -> Result<(), ApplicationWorkflowValidationDenial> {
    let mut unique = BTreeSet::new();
    for connection in connections {
        if !nodes.contains_key(connection.source()) || !nodes.contains_key(connection.target()) {
            return Err(denial(
                ApplicationWorkflowValidationDenialKind::UnknownConnectionEndpoint,
                format!(
                    "{} -> {}",
                    connection.source().as_str(),
                    connection.target().as_str()
                ),
            ));
        }
        let key = (connection.source(), connection.target(), connection.kind());
        if !unique.insert(key) {
            return Err(denial(
                ApplicationWorkflowValidationDenialKind::DuplicateConnection,
                format!(
                    "{} -> {}",
                    connection.source().as_str(),
                    connection.target().as_str()
                ),
            ));
        }
        if let ApplicationWorkflowConnectionKind::Data(flow) = connection.kind() {
            validate_flow(
                nodes[connection.source()].kind(),
                nodes[connection.target()].kind(),
                flow,
                connection.target(),
            )?;
        }
    }
    Ok(())
}

fn validate_flow(
    source: &ApplicationWorkflowNodeKind,
    target: &ApplicationWorkflowNodeKind,
    flow: ApplicationWorkflowDataFlow,
    subject: &ApplicationWorkflowNodeIdentity,
) -> Result<(), ApplicationWorkflowValidationDenial> {
    let valid = match flow {
        ApplicationWorkflowDataFlow::ProposalSubject => matches!(
            (source, target),
            (
                ApplicationWorkflowNodeKind::Operation { .. },
                ApplicationWorkflowNodeKind::Approval(_)
            )
        ),
        ApplicationWorkflowDataFlow::AssessmentSubject => matches!(
            (source, target),
            (
                ApplicationWorkflowNodeKind::Operation { .. },
                ApplicationWorkflowNodeKind::Assessment(_)
            )
        ),
        ApplicationWorkflowDataFlow::AssessmentEvidence => matches!(
            (source, target),
            (
                ApplicationWorkflowNodeKind::Assessment(_),
                ApplicationWorkflowNodeKind::EvidenceJoin
            )
        ),
        ApplicationWorkflowDataFlow::JoinedEvidence => matches!(
            (source, target),
            (
                ApplicationWorkflowNodeKind::EvidenceJoin,
                ApplicationWorkflowNodeKind::Approval(_)
            )
        ),
        ApplicationWorkflowDataFlow::ApprovalAuthority => matches!(
            (source, target),
            (
                ApplicationWorkflowNodeKind::Approval(_),
                ApplicationWorkflowNodeKind::Operation { .. }
            )
        ),
        ApplicationWorkflowDataFlow::OperationInput => matches!(
            (source, target),
            (
                ApplicationWorkflowNodeKind::Operation { .. },
                ApplicationWorkflowNodeKind::Operation { .. }
            )
        ),
    };
    if valid {
        Ok(())
    } else {
        Err(denial(
            ApplicationWorkflowValidationDenialKind::InvalidDataFlow,
            subject.as_str(),
        ))
    }
}

pub(super) fn validate_requirements(
    nodes: &BTreeMap<&ApplicationWorkflowNodeIdentity, &ApplicationWorkflowNode>,
    connections: &[ApplicationWorkflowConnection],
) -> Result<(), ApplicationWorkflowValidationDenial> {
    for (identity, node) in nodes {
        let incoming = |flow| {
            connections
                .iter()
                .filter(|connection| {
                    connection.target() == *identity
                        && connection.kind() == ApplicationWorkflowConnectionKind::Data(flow)
                })
                .count()
        };
        match node.kind() {
            ApplicationWorkflowNodeKind::Assessment(_)
                if incoming(ApplicationWorkflowDataFlow::AssessmentSubject) != 1 =>
            {
                return Err(denial(
                    ApplicationWorkflowValidationDenialKind::MissingAssessmentSubject,
                    identity.as_str(),
                ));
            }
            ApplicationWorkflowNodeKind::EvidenceJoin
                if incoming(ApplicationWorkflowDataFlow::AssessmentEvidence) < 2 =>
            {
                return Err(denial(
                    ApplicationWorkflowValidationDenialKind::IncompleteEvidenceJoin,
                    identity.as_str(),
                ));
            }
            ApplicationWorkflowNodeKind::Approval(_)
                if incoming(ApplicationWorkflowDataFlow::ProposalSubject) != 1
                    || incoming(ApplicationWorkflowDataFlow::JoinedEvidence) != 1 =>
            {
                return Err(denial(
                    ApplicationWorkflowValidationDenialKind::IncompleteApproval,
                    identity.as_str(),
                ));
            }
            ApplicationWorkflowNodeKind::Operation {
                requires_workflow_authority,
                ..
            } => validate_authority(*identity, *requires_workflow_authority, incoming)?,
            _ => {}
        }
    }
    Ok(())
}

pub(super) fn validate_availability(
    start: &ApplicationWorkflowNodeIdentity,
    nodes: &BTreeMap<&ApplicationWorkflowNodeIdentity, &ApplicationWorkflowNode>,
    connections: &[ApplicationWorkflowConnection],
) -> Result<(), ApplicationWorkflowValidationDenial> {
    let identities = nodes
        .keys()
        .map(|identity| (*identity).clone())
        .collect::<BTreeSet<_>>();
    let mut predecessors = identities
        .iter()
        .cloned()
        .map(|identity| (identity, BTreeSet::new()))
        .collect::<BTreeMap<_, _>>();
    for connection in connections {
        if matches!(
            connection.kind(),
            ApplicationWorkflowConnectionKind::Control(_)
        ) {
            predecessors
                .get_mut(connection.target())
                .expect("endpoint validation ran")
                .insert(connection.source().clone());
        }
    }

    let mut dominators = identities
        .iter()
        .cloned()
        .map(|identity| {
            let initial = if &identity == start {
                BTreeSet::from([identity.clone()])
            } else {
                identities.clone()
            };
            (identity, initial)
        })
        .collect::<BTreeMap<_, _>>();
    loop {
        let mut changed = false;
        for identity in identities.iter().filter(|identity| *identity != start) {
            let incoming = predecessors
                .get(identity)
                .expect("every validated node has a predecessor set");
            let mut next = incoming
                .iter()
                .map(|predecessor| {
                    dominators
                        .get(predecessor)
                        .expect("every predecessor is a validated node")
                        .clone()
                })
                .reduce(|left, right| left.intersection(&right).cloned().collect())
                .unwrap_or_default();
            next.insert(identity.clone());
            let current = dominators
                .get_mut(identity)
                .expect("every validated node has a dominator set");
            if *current != next {
                *current = next;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    for connection in connections {
        if matches!(
            connection.kind(),
            ApplicationWorkflowConnectionKind::Data(_)
        ) && (connection.source() == connection.target()
            || !dominators
                .get(connection.target())
                .expect("endpoint validation ran")
                .contains(connection.source()))
        {
            return Err(denial(
                ApplicationWorkflowValidationDenialKind::UnavailableDataFlow,
                format!(
                    "{} -> {}",
                    connection.source().as_str(),
                    connection.target().as_str()
                ),
            ));
        }
    }
    Ok(())
}

fn validate_authority(
    identity: &ApplicationWorkflowNodeIdentity,
    required: bool,
    incoming: impl Fn(ApplicationWorkflowDataFlow) -> usize,
) -> Result<(), ApplicationWorkflowValidationDenial> {
    let authorities = incoming(ApplicationWorkflowDataFlow::ApprovalAuthority);
    if required && authorities != 1 {
        return Err(denial(
            ApplicationWorkflowValidationDenialKind::MissingWorkflowAuthority,
            identity.as_str(),
        ));
    }
    if !required && authorities != 0 {
        return Err(denial(
            ApplicationWorkflowValidationDenialKind::UnexpectedWorkflowAuthority,
            identity.as_str(),
        ));
    }
    Ok(())
}
