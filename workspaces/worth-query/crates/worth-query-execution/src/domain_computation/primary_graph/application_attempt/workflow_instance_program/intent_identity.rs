use sha2::{Digest, Sha256};
use worth_relational::facade::identity::EntityId;

use crate::domain_computation::canonical_operation_material;
use crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowDefinition;

pub(super) fn instance_identity(
    compiled: &CompiledWorkflowDefinition,
    subject: EntityId,
    start_key_identity: [u8; 32],
) -> Result<(String, [u8; 32]), ()> {
    let material = canonical_operation_material(vec![
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
            "workflow.definition",
            compiled.content_identity().to_string(),
        ),
        ("workflow.program", compiled.program_revision().to_string()),
        ("workflow.start", compiled.start_path().to_owned()),
        ("workflow.start.node", compiled.start().identity_material()),
        ("workflow.structure", compiled.structure_identity_material()),
        ("workflow.nodes", compiled.node_count().to_string()),
        ("subject.partition", subject.partition_value().to_string()),
        ("subject.slot", subject.local_slot_value().to_string()),
        ("subject.generation", subject.generation_value().to_string()),
        ("start.key", encode(start_key_identity)),
    ]);
    let identity: [u8; 32] = Sha256::digest(material.as_bytes()).into();
    let mut text = String::with_capacity(64);
    for byte in identity {
        use std::fmt::Write;
        write!(&mut text, "{byte:02x}").map_err(|_| ())?;
    }
    Ok((text, identity))
}

fn encode(identity: [u8; 32]) -> String {
    let mut text = String::with_capacity(64);
    for byte in identity {
        use std::fmt::Write;
        write!(&mut text, "{byte:02x}").expect("writing to a String cannot fail");
    }
    text
}
