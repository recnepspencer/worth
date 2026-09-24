use sha2::{Digest, Sha256};

use crate::domain_computation::canonical_operation_material;

pub(in crate::domain_computation::primary_graph) struct WorkflowProposalMeaning {
    pub(super) identity: String,
    pub(super) operation: String,
    pub(super) input_type: String,
    pub(super) input_identity: String,
    pub(super) source_identity: Option<String>,
    pub(super) node_path: String,
}

impl WorkflowProposalMeaning {
    pub(in crate::domain_computation::primary_graph) fn identity(&self) -> &str {
        &self.identity
    }
}

pub(in crate::domain_computation::primary_graph) fn derive_workflow_proposal(
    transition_identity: &str,
    operation: &str,
    input_type: &str,
    input_identity: [u8; 32],
    source_identity: Option<[u8; 32]>,
    node_path: &str,
) -> WorkflowProposalMeaning {
    let input_identity = encode(input_identity);
    let source_identity = source_identity.map(encode);
    let material = canonical_operation_material(vec![
        ("workflow.transition", transition_identity.to_owned()),
        ("workflow.operation", operation.to_owned()),
        ("workflow.input-type", input_type.to_owned()),
        ("workflow.input", input_identity.clone()),
        (
            "workflow.source",
            source_identity.clone().unwrap_or_else(|| "none".to_owned()),
        ),
    ]);
    let identity_bytes: [u8; 32] = Sha256::digest(material.as_bytes()).into();
    WorkflowProposalMeaning {
        identity: encode(identity_bytes),
        operation: operation.to_owned(),
        input_type: input_type.to_owned(),
        input_identity,
        source_identity,
        node_path: node_path.to_owned(),
    }
}

pub(in crate::domain_computation::primary_graph) fn derive_workflow_proposal_context_identity(
    instance: worth_relational::facade::identity::EntityId,
) -> [u8; 32] {
    Sha256::digest(
        canonical_operation_material(vec![
            (
                "workflow.instance.partition",
                instance.partition_value().to_string(),
            ),
            (
                "workflow.instance.slot",
                instance.local_slot_value().to_string(),
            ),
            (
                "workflow.instance.generation",
                instance.generation_value().to_string(),
            ),
        ])
        .as_bytes(),
    )
    .into()
}

fn encode(identity: [u8; 32]) -> String {
    let mut text = String::with_capacity(64);
    for byte in identity {
        use std::fmt::Write;
        write!(&mut text, "{byte:02x}").expect("writing a workflow identity to String cannot fail");
    }
    text
}
