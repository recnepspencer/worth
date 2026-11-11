//! Portable operation identity and exact conditional-node location.
use super::*;
use worth_query_installation::facade::WorthQueryConditionalNodeLocation;

pub(in crate::domain_computation::primary_graph::conditional_operation) fn prepare_temporal_node_definition(
    operation: CanonicalDigestId,
    domain: &str,
    slot: &str,
    location: &WorthQueryConditionalNodeLocation,
) -> Result<(CanonicalDigestId, WorthQueryCanonicalWorkEvidence), CanonicalDigestDerivationDenial> {
    let mut meaning = CanonicalIdentityMaterial::new(
        "worth-query.temporal-node-definition",
        "worth-query-temporal-node-definition-v1",
    );
    meaning.digest("operation-definition", operation);
    // This is the installed owner's complete, canonical domain definition,
    // already covering its conditional declarations. No authority seal or
    // independent declaration parser participates in the durable key.
    meaning.text("domain-operation", domain);
    meaning.text("domain-slot", slot);
    match location {
        WorthQueryConditionalNodeLocation::Operation { node_identity } => {
            meaning.text("location", "operation");
            meaning.text("node", node_identity);
        }
        WorthQueryConditionalNodeLocation::WorkflowStage {
            stage_identity,
            node_identity,
        } => {
            meaning.text("location", "workflow-stage");
            meaning.text("stage", stage_identity);
            meaning.text("node", node_identity);
        }
    }
    meaning.derive()
}

#[cfg(test)]
mod tests;
