use im::OrdMap;
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
    back_edge_iterations: &OrdMap<(EntityId, EntityId), u64>,
    navigation_back: bool,
) -> Result<(String, [u8; 32]), WorthQueryApplicationAttemptDenial> {
    let mut fields = vec![
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
    ];
    if !back_edge_iterations.is_empty() {
        let mut vector = String::new();
        for ((source, target), iterations) in back_edge_iterations {
            use std::fmt::Write;
            write!(
                &mut vector,
                "{}:{}:{}>{}:{}:{}={iterations};",
                source.partition_value(),
                source.local_slot_value(),
                source.generation_value(),
                target.partition_value(),
                target.local_slot_value(),
                target.generation_value(),
            )
            .expect("writing a workflow iteration vector to String cannot fail");
        }
        fields.push(("workflow.node.path", node.path().to_owned()));
        fields.push(("workflow.back-edge-iterations", vector));
    }
    if navigation_back {
        fields.push(("workflow.navigation", "back".to_owned()));
    }
    let material = canonical_operation_material(fields);
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
