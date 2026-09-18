use super::WorthQueryApplicationEffectProgram;

mod canonical_encoding;
mod output_postcondition;

use canonical_encoding::{dependency_identity, lineage_identity};
use output_postcondition::{is_output_currentness_fact, normalized_output_facts};

impl<Schema, Operation, Input, Scope>
    WorthQueryApplicationEffectProgram<Schema, Operation, Input, Scope>
{
    pub(in crate::domain_computation::primary_graph) fn producer_idempotency_identities<
        OutputBinding,
    >(
        &mut self,
        runtime: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<
            Schema,
        >,
        declared_key: [u8; 32],
        successor_of: Option<[u8; 32]>,
    ) -> Result<([u8; 32], [u8; 32]), ()>
    where
        OutputBinding: 'static,
    {
        let dependency_facts = normalized_output_facts(&self.read_set.facts, &self.effects);
        let (dependency, work) = dependency_identity(declared_key, &dependency_facts)?;
        self.output_currentness_facts = Some(
            dependency_facts
                .into_iter()
                .filter(is_output_currentness_fact)
                .collect::<Vec<_>>()
                .into(),
        );
        self.read_set
            .admission
            .retain_execution_canonical_work(work);
        let head = runtime
            .primary_provider
            .graph
            .output_lineage
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .producer_head::<OutputBinding>(
                self.read_set.admission.operation_scope_binding(),
                self.read_set.lease.product().observation(),
                self.read_set
                    .admission
                    .source_partition_identity()
                    .ok_or(())?,
            );
        let force_successor = successor_of.is_some_and(|stale_key| {
            head.is_some_and(|head| head.idempotency_key_identity == stale_key)
        });
        if let Some(head) = head {
            if !force_successor
                && head.occurrence
                    == self
                        .read_set
                        .lease
                        .product()
                        .observation()
                        .lifecycle_incarnation()
                && head.dependency_identity == Some(dependency)
            {
                return Ok((head.idempotency_key_identity, dependency));
            }
        }
        let prior_head = head
            .map(|head| head.idempotency_key_identity)
            .or(successor_of);
        let (key, work) = lineage_identity(dependency, prior_head)?;
        self.read_set
            .admission
            .retain_execution_canonical_work(work);
        Ok((key, dependency))
    }
}

#[cfg(test)]
mod tests;
