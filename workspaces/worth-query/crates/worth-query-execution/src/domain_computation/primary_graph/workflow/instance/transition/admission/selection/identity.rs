use sha2::{Digest, Sha256};
use worth_relational::facade::identity::EntityId;

use super::{CompiledWorkflowDefinition, CompiledWorkflowNode};
use crate::domain_computation::canonical_operation_material;
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
};

pub(super) fn transition_identity(
    compiled: &CompiledWorkflowDefinition,
    instance: EntityId,
    node: &CompiledWorkflowNode,
    occurrence: u64,
) -> Result<(String, [u8; 32]), WorthQueryApplicationAttemptDenial> {
    let material = canonical_operation_material(vec![
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
        (
            "workflow.definition.partition",
            compiled.definition().partition_value().to_string(),
        ),
        (
            "workflow.definition.slot",
            compiled.definition().local_slot_value().to_string(),
        ),
        (
            "workflow.definition.generation",
            compiled.definition().generation_value().to_string(),
        ),
        (
            "workflow.node.partition",
            node.entity().partition_value().to_string(),
        ),
        (
            "workflow.node.slot",
            node.entity().local_slot_value().to_string(),
        ),
        (
            "workflow.node.generation",
            node.entity().generation_value().to_string(),
        ),
        ("workflow.occurrence", occurrence.to_string()),
    ]);
    let identity_bytes: [u8; 32] = Sha256::digest(material.as_bytes()).into();
    let identity = encode(identity_bytes)?;
    Ok((identity, identity_bytes))
}

fn encode(identity: [u8; 32]) -> Result<String, WorthQueryApplicationAttemptDenial> {
    let mut text = String::with_capacity(64);
    for byte in identity {
        use std::fmt::Write;
        write!(&mut text, "{byte:02x}").map_err(|_| {
            super::denial(
                WorthQueryApplicationAttemptDenialKind::WorkflowTransitionIdentityUnavailable,
                "workflow transition identity encoding failed",
            )
        })?;
    }
    Ok(text)
}
