use worth_foundational::facade::{
    canonicalization, prepare_canonical_basis_sequence, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisValue, CanonicalDigestAlgorithmId,
    CanonicalDigestDerivationDenial, CanonicalDigestWorkBudget, CanonicalIntegerWidth,
    CanonicalizationRuleVersion,
};

use super::{
    ApplicationWorkflowConnection, ApplicationWorkflowConnectionKind,
    ApplicationWorkflowControlOutcome, ApplicationWorkflowDataFlow,
    ApplicationWorkflowDefinitionContentIdentity, ApplicationWorkflowDefinitionLimits,
    ApplicationWorkflowNode, ApplicationWorkflowNodeKind, ApplicationWorkflowSpec,
};

const RULE_VERSION: &str = "worth-query-authored-workflow-definition-v1";
const BASIS_DOMAIN: CanonicalBasisDomain =
    CanonicalBasisDomain::Future("worth-query.authored-workflow-definition");

pub(super) fn canonicalize<Spec>(
    limits: ApplicationWorkflowDefinitionLimits,
    start: &str,
    mut nodes: Vec<ApplicationWorkflowNode>,
    mut connections: Vec<ApplicationWorkflowConnection>,
) -> Result<
    (
        ApplicationWorkflowDefinitionContentIdentity,
        Box<[ApplicationWorkflowNode]>,
        Box<[ApplicationWorkflowConnection]>,
    ),
    CanonicalDigestDerivationDenial,
>
where
    Spec: ApplicationWorkflowSpec,
{
    nodes.sort_by(|left, right| left.identity().cmp(right.identity()));
    connections.sort_by(|left, right| connection_key(left).cmp(&connection_key(right)));
    let mut entries = Vec::with_capacity(nodes.len() + connections.len() + 5);
    entries.push(text_entry("spec", Spec::IDENTITY.as_str()));
    entries.push(text_entry("start", start));
    entries.push(text_entry("limits", &limits_record(limits)));
    entries.push(unsigned_entry("node-count", nodes.len() as u128));
    entries.extend(
        nodes
            .iter()
            .enumerate()
            .map(|(index, node)| text_entry(&format!("node[{index}]"), &node_record(node))),
    );
    entries.push(unsigned_entry(
        "connection-count",
        connections.len() as u128,
    ));
    entries.extend(connections.iter().enumerate().map(|(index, connection)| {
        text_entry(
            &format!("connection[{index}]"),
            &connection_record(connection),
        )
    }));
    let basis = prepare_canonical_basis_sequence(
        CanonicalizationRuleVersion::new(RULE_VERSION)
            .expect("the workflow canonical rule version is valid"),
        BASIS_DOMAIN,
        entries,
    )
    .into_result()
    .expect("workflow canonical loci are unique and the basis is nonempty");
    let maximum_entries = u32::from(limits.maximum_nodes())
        .saturating_add(u32::from(limits.maximum_connections()))
        .saturating_add(8);
    let budget =
        CanonicalDigestWorkBudget::new(maximum_entries, limits.maximum_canonical_bytes() as usize)
            .expect("validated workflow limits are nonzero");
    let ready = canonicalization()
        .digest()
        .for_sequence_with_budget(basis, CanonicalDigestAlgorithmId::sha256(), budget)
        .into_result()?;
    let digest = canonicalization().digest().derive(ready);
    Ok((
        ApplicationWorkflowDefinitionContentIdentity(*digest.value().bytes()),
        nodes.into_boxed_slice(),
        connections.into_boxed_slice(),
    ))
}

fn limits_record(limits: ApplicationWorkflowDefinitionLimits) -> String {
    format!(
        "nodes={};connections={};effects={};depth={};bytes={}",
        limits.maximum_nodes(),
        limits.maximum_connections(),
        limits.maximum_effects(),
        limits.maximum_component_depth(),
        limits.maximum_canonical_bytes()
    )
}

fn node_record(node: &ApplicationWorkflowNode) -> String {
    match node.kind() {
        ApplicationWorkflowNodeKind::Operation {
            operation,
            requires_workflow_authority,
        } => framed_record(
            "operation",
            &[
                node.identity().as_str(),
                operation.identifier(),
                operation.input_type().as_str(),
                if *requires_workflow_authority {
                    "1"
                } else {
                    "0"
                },
            ],
        ),
        ApplicationWorkflowNodeKind::Assessment(assessment) => framed_record(
            "assessment",
            &[
                node.identity().as_str(),
                assessment.identifier(),
                assessment.parameter_type().as_str(),
                assessment.result_type().as_str(),
            ],
        ),
        ApplicationWorkflowNodeKind::Approval(approval) => framed_record(
            "approval",
            &[
                node.identity().as_str(),
                approval.identifier(),
                approval.capability_type().as_str(),
            ],
        ),
        ApplicationWorkflowNodeKind::EvidenceJoin => {
            framed_record("evidence-join", &[node.identity().as_str()])
        }
        ApplicationWorkflowNodeKind::Terminal => {
            framed_record("terminal", &[node.identity().as_str()])
        }
    }
}

fn connection_key(connection: &ApplicationWorkflowConnection) -> (&str, &str, u8, u8) {
    let (family, variant) = match connection.kind() {
        ApplicationWorkflowConnectionKind::Control(outcome) => (0, control_tag(outcome)),
        ApplicationWorkflowConnectionKind::Data(flow) => (1, data_tag(flow)),
    };
    (
        connection.source().as_str(),
        connection.target().as_str(),
        family,
        variant,
    )
}

fn connection_record(connection: &ApplicationWorkflowConnection) -> String {
    let (family, variant) = match connection.kind() {
        ApplicationWorkflowConnectionKind::Control(outcome) => ("control", control_tag(outcome)),
        ApplicationWorkflowConnectionKind::Data(flow) => ("data", data_tag(flow)),
    };
    let variant = variant.to_string();
    framed_record(
        family,
        &[
            connection.source().as_str(),
            connection.target().as_str(),
            &variant,
        ],
    )
}

fn framed_record(kind: &str, fields: &[&str]) -> String {
    let mut record = format!("{}:{kind}", kind.len());
    for field in fields {
        record.push('|');
        record.push_str(&field.len().to_string());
        record.push(':');
        record.push_str(field);
    }
    record
}

const fn control_tag(outcome: ApplicationWorkflowControlOutcome) -> u8 {
    match outcome {
        ApplicationWorkflowControlOutcome::Completed => 0,
        ApplicationWorkflowControlOutcome::Approved => 1,
        ApplicationWorkflowControlOutcome::Rejected => 2,
    }
}

const fn data_tag(flow: ApplicationWorkflowDataFlow) -> u8 {
    match flow {
        ApplicationWorkflowDataFlow::ProposalSubject => 0,
        ApplicationWorkflowDataFlow::AssessmentSubject => 1,
        ApplicationWorkflowDataFlow::AssessmentEvidence => 2,
        ApplicationWorkflowDataFlow::JoinedEvidence => 3,
        ApplicationWorkflowDataFlow::ApprovalAuthority => 4,
        ApplicationWorkflowDataFlow::OperationInput => 5,
    }
}

fn text_entry(locus: &str, value: &str) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        BASIS_DOMAIN,
        CanonicalBasisLocus::Named(locus.to_owned().into()),
        CanonicalBasisEntryKind::Field,
        CanonicalBasisValue::ExactText(value.to_owned().into()),
    )
}

fn unsigned_entry(locus: &str, value: u128) -> CanonicalBasisEntry {
    CanonicalBasisEntry::new(
        BASIS_DOMAIN,
        CanonicalBasisLocus::Named(locus.to_owned().into()),
        CanonicalBasisEntryKind::Shape,
        CanonicalBasisValue::UnsignedInteger {
            width: CanonicalIntegerWidth::Bits64,
            value,
        },
    )
}
