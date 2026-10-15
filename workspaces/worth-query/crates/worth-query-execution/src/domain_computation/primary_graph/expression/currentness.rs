use sha2::{Digest, Sha256};

use crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowConditionExpression;

const DOMAIN: &[u8] = b"worth-query/workflow-condition-sources/v1";

/// Binds every current expression operand source by name.
pub(in crate::domain_computation::primary_graph) fn supporting_identity<'operand>(
    _expression: &CompiledWorkflowConditionExpression,
    sources: impl IntoIterator<Item = (&'operand str, [u8; 32])>,
) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(DOMAIN);
    for (name, identity) in sources {
        hasher.update((name.len() as u64).to_be_bytes());
        hasher.update(name.as_bytes());
        hasher.update(identity);
    }
    hasher.finalize().into()
}
