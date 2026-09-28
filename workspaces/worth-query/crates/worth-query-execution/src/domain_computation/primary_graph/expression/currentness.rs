use sha2::{Digest, Sha256};

use crate::domain_computation::primary_graph::workflow::definition::CompiledWorkflowConditionExpression;

const DOMAIN: &[u8] = b"worth-query/workflow-condition-sources/v1";

/// The identity a condition transition binds its idempotency to. A migrated
/// condition keeps its single source's identity, so a retry of a version-1
/// transition replays; an expression binds every operand source by name, in
/// name order, so a changed operand is a different transition intent.
pub(in crate::domain_computation::primary_graph) fn supporting_identity<'operand>(
    expression: &CompiledWorkflowConditionExpression,
    sources: impl IntoIterator<Item = (&'operand str, [u8; 32])>,
) -> [u8; 32] {
    let mut sources = sources.into_iter();
    if let CompiledWorkflowConditionExpression::Migrated = expression {
        let (_, identity) = sources
            .next()
            .expect("a migrated condition reads exactly one source");
        return identity;
    }
    let mut hasher = Sha256::new();
    hasher.update(DOMAIN);
    for (name, identity) in sources {
        hasher.update((name.len() as u64).to_be_bytes());
        hasher.update(name.as_bytes());
        hasher.update(identity);
    }
    hasher.finalize().into()
}
