//! One node or connection record. Nodes name vocabulary by identifier with
//! the portable types they were authored against; connections carry their
//! control, data or retry meaning directly.

use worth_query_declaration::facade::application_program::{
    ApplicationWorkflowConnection, ApplicationWorkflowConnectionKind,
    ApplicationWorkflowControlOutcome, ApplicationWorkflowDataFlow, ApplicationWorkflowNode,
    ApplicationWorkflowNodeKind, ApplicationWorkflowRetry,
};

use super::{DraftConnection, DraftMember, DraftNode};

mod condition;
use crate::binary_input::BinaryInput;
use crate::binary_output::BinaryOutput;
use crate::denial::{
    WorthQueryPackageArchiveDenial as Denial, WorthQueryPackageArchiveDenialKind as Kind,
};

/// A tag and an empty identity: the smallest node record.
pub(super) const MINIMUM_NODE_BYTES: usize = 1 + 4;
/// A kind tag, two empty identities and an outcome tag.
pub(super) const MINIMUM_CONNECTION_BYTES: usize = 1 + 4 + 4 + 1;

pub(super) fn encode_node(
    output: &mut BinaryOutput,
    node: &ApplicationWorkflowNode,
) -> Result<(), Denial> {
    let (tag, fields): (u8, Vec<&str>) = match node.kind() {
        ApplicationWorkflowNodeKind::Operation { operation, .. } => (
            1,
            vec![
                operation.identifier(),
                operation.input_type().as_str(),
                operation.binding().map_or("", |(identity, _, _)| identity),
            ],
        ),
        ApplicationWorkflowNodeKind::Assessment(assessment) => (
            2,
            vec![
                assessment.identifier(),
                assessment.parameter_type().as_str(),
                assessment.result_type().as_str(),
            ],
        ),
        ApplicationWorkflowNodeKind::Condition(_) => (3, Vec::new()),
        ApplicationWorkflowNodeKind::Approval(approval) => (
            4,
            vec![approval.identifier(), approval.capability_type().as_str()],
        ),
        ApplicationWorkflowNodeKind::EvidenceJoin(policy) => (5, vec![policy.identity()]),
        ApplicationWorkflowNodeKind::Terminal => (6, Vec::new()),
    };
    output.raw_bytes(&[tag]);
    text(output, node.identity().as_str())?;
    for field in fields {
        text(output, field)?;
    }
    match node.kind() {
        ApplicationWorkflowNodeKind::Operation {
            requires_workflow_authority,
            ..
        } => output.raw_bytes(&[u8::from(*requires_workflow_authority)]),
        ApplicationWorkflowNodeKind::Assessment(assessment) => {
            text(output, &assessment.subject().persistence_identity())?;
            let relation = assessment.applicability().relation();
            output.raw_bytes(&[u8::from(relation.is_some())]);
            for field in relation
                .into_iter()
                .flat_map(|(name, from, to)| [name, from, to])
            {
                text(output, field)?;
            }
        }
        ApplicationWorkflowNodeKind::Condition(condition) => {
            condition::encode_condition(output, condition)?;
        }
        _ => {}
    }
    Ok(())
}

/// Decodes one node record written under draft protocol `version`.
pub(super) fn decode_node(input: &mut BinaryInput<'_>, version: u16) -> Result<DraftNode, Denial> {
    let tag = input.u8()?;
    let identity = nonempty_text(input)?;
    let member = match tag {
        1 => {
            let identifier = nonempty_text(input)?;
            let input_type = nonempty_text(input)?;
            let binding = Some(owned_text(input)?).filter(|binding| !binding.is_empty());
            DraftMember::Operation {
                identifier,
                input_type,
                binding,
                requires_workflow_authority: boolean(input)?,
            }
        }
        2 => {
            let identifier = nonempty_text(input)?;
            let parameter_type = nonempty_text(input)?;
            let result_type = nonempty_text(input)?;
            let subject = nonempty_text(input)?;
            let related_relation = if boolean(input)? {
                Some([
                    nonempty_text(input)?,
                    nonempty_text(input)?,
                    nonempty_text(input)?,
                ])
            } else {
                None
            };
            DraftMember::Assessment {
                identifier,
                parameter_type,
                result_type,
                subject,
                related_relation,
            }
        }
        3 if version == 1 => DraftMember::Condition(condition::decode_version_1_condition(input)?),
        3 => DraftMember::Condition(condition::decode_condition(input)?),
        4 => DraftMember::Approval {
            identifier: nonempty_text(input)?,
            capability_type: nonempty_text(input)?,
        },
        5 => DraftMember::EvidenceJoin {
            policy: nonempty_text(input)?,
        },
        6 => DraftMember::Terminal,
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    Ok(DraftNode { identity, member })
}

pub(super) fn encode_connection(
    output: &mut BinaryOutput,
    connection: &ApplicationWorkflowConnection,
) -> Result<(), Denial> {
    let kind = connection.kind();
    output.raw_bytes(&[match kind {
        ApplicationWorkflowConnectionKind::Control(_) => 1,
        ApplicationWorkflowConnectionKind::Data(_) => 2,
        ApplicationWorkflowConnectionKind::Retry(_) => 3,
    }]);
    text(output, connection.source().as_str())?;
    text(output, connection.target().as_str())?;
    match kind {
        ApplicationWorkflowConnectionKind::Control(outcome) => {
            output.raw_bytes(&[control_tag(outcome)]);
        }
        ApplicationWorkflowConnectionKind::Data(flow) => output.raw_bytes(&[data_tag(flow)]),
        ApplicationWorkflowConnectionKind::Retry(retry) => {
            output.raw_bytes(&[control_tag(retry.trigger())]);
            text(output, retry.reason())?;
            output.u16(retry.maximum_attempts());
        }
    }
    Ok(())
}

pub(super) fn decode_connection(input: &mut BinaryInput<'_>) -> Result<DraftConnection, Denial> {
    let tag = input.u8()?;
    let source = nonempty_text(input)?;
    let target = nonempty_text(input)?;
    let kind = match tag {
        1 => ApplicationWorkflowConnectionKind::Control(control_outcome(input.u8()?)?),
        2 => ApplicationWorkflowConnectionKind::Data(data_flow(input.u8()?)?),
        3 => {
            let trigger = control_outcome(input.u8()?)?;
            let reason = input.text()?;
            let retry = ApplicationWorkflowRetry::new(trigger, reason, input.u16()?)
                .ok_or_else(|| Denial::new(Kind::InvalidRecordShape))?;
            ApplicationWorkflowConnectionKind::Retry(retry)
        }
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    };
    Ok(DraftConnection {
        source,
        target,
        kind,
    })
}

pub(super) fn text(output: &mut BinaryOutput, value: &str) -> Result<(), Denial> {
    u32::try_from(value.len()).map_err(|_| Denial::new(Kind::NumericWidthExceeded))?;
    output.text(value);
    Ok(())
}

pub(super) fn nonempty_text(input: &mut BinaryInput<'_>) -> Result<String, Denial> {
    let value = owned_text(input)?;
    if value.is_empty() {
        Err(Denial::new(Kind::InvalidRecordShape))
    } else {
        Ok(value)
    }
}

fn owned_text(input: &mut BinaryInput<'_>) -> Result<String, Denial> {
    input.text().map(str::to_owned)
}

fn boolean(input: &mut BinaryInput<'_>) -> Result<bool, Denial> {
    match input.u8()? {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(Denial::new(Kind::InvalidBooleanEncoding)),
    }
}

// Wire tags are part of the draft format since v1: each is written out by hand so
// no reordering can move one, and a new variant fails to compile here.

pub(super) const fn control_tag(outcome: ApplicationWorkflowControlOutcome) -> u8 {
    match outcome {
        ApplicationWorkflowControlOutcome::Completed => 0,
        ApplicationWorkflowControlOutcome::Approved => 1,
        ApplicationWorkflowControlOutcome::Rejected => 2,
        ApplicationWorkflowControlOutcome::EvidenceSatisfied => 3,
        ApplicationWorkflowControlOutcome::EvidenceFailed => 4,
        ApplicationWorkflowControlOutcome::RetryExhausted => 5,
        ApplicationWorkflowControlOutcome::ConditionSatisfied => 6,
        ApplicationWorkflowControlOutcome::ConditionUnsatisfied => 7,
        ApplicationWorkflowControlOutcome::NavigatedBack => 8,
    }
}

pub(super) const fn data_tag(flow: ApplicationWorkflowDataFlow) -> u8 {
    match flow {
        ApplicationWorkflowDataFlow::ProposalSubject => 0,
        ApplicationWorkflowDataFlow::AssessmentSubject => 1,
        ApplicationWorkflowDataFlow::AssessmentEvidence => 2,
        ApplicationWorkflowDataFlow::JoinedEvidence => 3,
        ApplicationWorkflowDataFlow::ApprovalAuthority => 4,
        ApplicationWorkflowDataFlow::OperationInput => 5,
        ApplicationWorkflowDataFlow::ConditionSubject => 6,
    }
}

pub(super) fn control_outcome(tag: u8) -> Result<ApplicationWorkflowControlOutcome, Denial> {
    Ok(match tag {
        0 => ApplicationWorkflowControlOutcome::Completed,
        1 => ApplicationWorkflowControlOutcome::Approved,
        2 => ApplicationWorkflowControlOutcome::Rejected,
        3 => ApplicationWorkflowControlOutcome::EvidenceSatisfied,
        4 => ApplicationWorkflowControlOutcome::EvidenceFailed,
        5 => ApplicationWorkflowControlOutcome::RetryExhausted,
        6 => ApplicationWorkflowControlOutcome::ConditionSatisfied,
        7 => ApplicationWorkflowControlOutcome::ConditionUnsatisfied,
        8 => ApplicationWorkflowControlOutcome::NavigatedBack,
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    })
}

pub(super) fn data_flow(tag: u8) -> Result<ApplicationWorkflowDataFlow, Denial> {
    Ok(match tag {
        0 => ApplicationWorkflowDataFlow::ProposalSubject,
        1 => ApplicationWorkflowDataFlow::AssessmentSubject,
        2 => ApplicationWorkflowDataFlow::AssessmentEvidence,
        3 => ApplicationWorkflowDataFlow::JoinedEvidence,
        4 => ApplicationWorkflowDataFlow::ApprovalAuthority,
        5 => ApplicationWorkflowDataFlow::OperationInput,
        6 => ApplicationWorkflowDataFlow::ConditionSubject,
        _ => return Err(Denial::new(Kind::UnsupportedRecordVariant)),
    })
}
