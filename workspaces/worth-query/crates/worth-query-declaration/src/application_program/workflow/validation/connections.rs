use crate::application_program::workflow::{
    ApplicationWorkflowConnectionKind, ApplicationWorkflowDataFlow,
    ApplicationWorkflowNodeIdentity, ApplicationWorkflowNodeKind,
};
use std::mem::size_of;

use super::{
    denial, index::ValidationGraph, ApplicationWorkflowValidationDenial,
    ApplicationWorkflowValidationDenialKind, ValidationWorkMeter,
};

pub(super) fn validate_requirements(
    graph: &ValidationGraph<'_>,
    work: &mut ValidationWorkMeter,
) -> Result<(), ApplicationWorkflowValidationDenial> {
    for (index, node) in graph.nodes() {
        work.visit_requirement_node();
        let incoming = |flow| graph.summary(index).incoming(flow);
        match node.kind() {
            ApplicationWorkflowNodeKind::Condition(_)
                if incoming(ApplicationWorkflowDataFlow::ConditionSubject) != 1 =>
            {
                return Err(denial(
                    ApplicationWorkflowValidationDenialKind::MissingConditionSubject,
                    node.identity().as_str(),
                ));
            }
            ApplicationWorkflowNodeKind::Assessment(_)
                if incoming(ApplicationWorkflowDataFlow::AssessmentSubject) != 1 =>
            {
                return Err(denial(
                    ApplicationWorkflowValidationDenialKind::MissingAssessmentSubject,
                    node.identity().as_str(),
                ));
            }
            ApplicationWorkflowNodeKind::EvidenceJoin(_)
                if incoming(ApplicationWorkflowDataFlow::AssessmentEvidence) == 0 =>
            {
                return Err(denial(
                    ApplicationWorkflowValidationDenialKind::IncompleteEvidenceJoin,
                    node.identity().as_str(),
                ));
            }
            ApplicationWorkflowNodeKind::Approval(_)
                if incoming(ApplicationWorkflowDataFlow::ProposalSubject) != 1
                    || incoming(ApplicationWorkflowDataFlow::JoinedEvidence) != 1 =>
            {
                return Err(denial(
                    ApplicationWorkflowValidationDenialKind::IncompleteApproval,
                    node.identity().as_str(),
                ));
            }
            ApplicationWorkflowNodeKind::Operation {
                operation,
                requires_workflow_authority,
            } => {
                validate_authority(
                    node.identity(),
                    *requires_workflow_authority,
                    operation.workflow_execution_requirements(),
                    incoming(ApplicationWorkflowDataFlow::ApprovalAuthority),
                )?;
                if *requires_workflow_authority
                    && incoming(ApplicationWorkflowDataFlow::OperationInput)
                        + incoming(ApplicationWorkflowDataFlow::CapturedOperationInput)
                        != 1
                {
                    return Err(denial(
                        ApplicationWorkflowValidationDenialKind::MissingOperationInput,
                        node.identity().as_str(),
                    ));
                }
            }
            _ => {}
        }
    }
    Ok(())
}

pub(super) fn validate(
    graph: &ValidationGraph<'_>,
    effects: usize,
    maximum_effects: u16,
    work: &mut ValidationWorkMeter,
) -> Result<(), ApplicationWorkflowValidationDenial> {
    validate_flows(graph, effects, maximum_effects, work)?;
    require_unique_connections(graph, work)
}

fn validate_flows(
    graph: &ValidationGraph<'_>,
    effects: usize,
    maximum_effects: u16,
    work: &mut ValidationWorkMeter,
) -> Result<(), ApplicationWorkflowValidationDenial> {
    let mut back = super::back_actions::BackActionValidation::new(effects, maximum_effects);
    for indexed in graph.connections() {
        work.visit_connection_semantics();
        if let ApplicationWorkflowConnectionKind::BackAction(operation) =
            indexed.connection.kind_ref()
        {
            back.validate(graph, indexed, operation, work)?;
        }
        if let ApplicationWorkflowConnectionKind::Data(flow) = indexed.connection.kind_ref() {
            validate_flow(
                graph.node(indexed.source).kind(),
                graph.node(indexed.target).kind(),
                *flow,
                indexed.connection.target(),
            )?;
        }
    }
    Ok(())
}

fn require_unique_connections(
    graph: &ValidationGraph<'_>,
    work: &mut ValidationWorkMeter,
) -> Result<(), ApplicationWorkflowValidationDenial> {
    let connections = graph.connections();
    let mut order = (0..connections.len()).collect::<Vec<_>>();
    order.sort_unstable_by(|left, right| {
        let left = &connections[*left];
        let right = &connections[*right];
        (left.source, left.target, left.connection.kind_ref()).cmp(&(
            right.source,
            right.target,
            right.connection.kind_ref(),
        ))
    });
    work.observe_index_bytes(
        graph
            .retained_bytes()
            .saturating_add(order.capacity() * size_of::<usize>()),
    );
    if let Some(duplicate) = order.windows(2).find_map(|pair| {
        let left = &connections[pair[0]];
        let right = &connections[pair[1]];
        (left.source == right.source
            && left.target == right.target
            && left.connection.kind_ref() == right.connection.kind_ref())
        .then_some(right.connection)
    }) {
        return Err(denial(
            ApplicationWorkflowValidationDenialKind::DuplicateConnection,
            format!(
                "{} -> {}",
                duplicate.source().as_str(),
                duplicate.target().as_str()
            ),
        ));
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
        ApplicationWorkflowDataFlow::ConditionSubject => matches!(
            (source, target),
            (
                ApplicationWorkflowNodeKind::Operation { .. },
                ApplicationWorkflowNodeKind::Condition(_)
            )
        ),
        ApplicationWorkflowDataFlow::AssessmentEvidence => matches!(
            (source, target),
            (
                ApplicationWorkflowNodeKind::Assessment(_),
                ApplicationWorkflowNodeKind::EvidenceJoin(_)
            )
        ),
        ApplicationWorkflowDataFlow::JoinedEvidence => matches!(
            (source, target),
            (
                ApplicationWorkflowNodeKind::EvidenceJoin(_),
                ApplicationWorkflowNodeKind::Approval(_)
            )
        ),
        ApplicationWorkflowDataFlow::ApprovalAuthority => match (source, target) {
            (
                ApplicationWorkflowNodeKind::Approval(approval),
                ApplicationWorkflowNodeKind::Operation { operation, .. },
            ) => operation
                .workflow_execution_requirements()
                .approval()
                .is_none_or(|(identifier, identity)| {
                    approval.identifier() == identifier
                        && approval.capability_type().as_str() == identity
                }),
            _ => false,
        },
        ApplicationWorkflowDataFlow::CapturedOperationInput => matches!(
            (source, target),
            (
                ApplicationWorkflowNodeKind::Operation { operation: source, requires_workflow_authority: false },
                ApplicationWorkflowNodeKind::Operation { operation: target, requires_workflow_authority: true },
            ) if source.binding().is_some()
                && source.is_workflow_control()
                && target.binding().is_some()
                && !target.is_workflow_control()
        ),
        ApplicationWorkflowDataFlow::OperationInput => match (source, target) {
            (
                ApplicationWorkflowNodeKind::Operation {
                    operation: source, ..
                },
                ApplicationWorkflowNodeKind::Operation {
                    operation: target, ..
                },
            ) => {
                if source.input_type() != target.input_type() {
                    return Err(denial(
                        ApplicationWorkflowValidationDenialKind::IncompatibleOperationInput,
                        subject.as_str(),
                    ));
                }
                true
            }
            _ => false,
        },
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

fn validate_authority(
    identity: &ApplicationWorkflowNodeIdentity,
    required: bool,
    requirements: &crate::application_operation::ApplicationWorkflowExecutionRequirements,
    authorities: usize,
) -> Result<(), ApplicationWorkflowValidationDenial> {
    let approval_required = requirements.approval().is_some();
    if required
        && (!requirements.is_applicable()
            || authorities > 1
            || (approval_required && authorities != 1))
    {
        return Err(denial(
            ApplicationWorkflowValidationDenialKind::MissingWorkflowAuthority,
            identity.as_str(),
        ));
    }
    if !required && (authorities != 0 || requirements.is_applicable()) {
        return Err(denial(
            ApplicationWorkflowValidationDenialKind::UnexpectedWorkflowAuthority,
            identity.as_str(),
        ));
    }
    Ok(())
}
