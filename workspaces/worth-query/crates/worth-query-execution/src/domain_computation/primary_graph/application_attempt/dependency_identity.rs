use super::WorthQueryApplicationEffectProgram;

mod canonical_encoding;
mod output_postcondition;

use canonical_encoding::{dependency_identity, lineage_identity};
use output_postcondition::{complete_output_currentness_facts, normalized_output_facts};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum WorthQueryProducerIdentityDenial {
    DependencyByteCapacityUnsupported,
    DependencyCanonicalizationRejected,
    MissingSourcePartition,
    LineageLookupBudgetExceeded,
    LineageCanonicalizationRejected,
}

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
        request_admission: &mut crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission,
    ) -> Result<([u8; 32], [u8; 32]), WorthQueryProducerIdentityDenial>
    where
        OutputBinding: 'static,
    {
        let dependency_facts = normalized_output_facts(&self.read_set.facts, &self.effects);
        let maximum_dependency_bytes = usize::try_from(
            runtime
                .runtime
                .application_candidate_resource_profile()
                .maximum_producer_dependency_bytes(),
        )
        .map_err(|_| WorthQueryProducerIdentityDenial::DependencyByteCapacityUnsupported)?;
        let (dependency, work) =
            dependency_identity(declared_key, &dependency_facts, maximum_dependency_bytes)
                .map_err(|_| {
                    WorthQueryProducerIdentityDenial::DependencyCanonicalizationRejected
                })?;
        self.output_currentness_facts = Some(complete_output_currentness_facts(dependency_facts));
        self.read_set
            .admission
            .retain_execution_canonical_work(work);
        // The lineage head lookup is framework preparation on the request meter.
        let (head, lookup_work) = runtime
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
                    .ok_or(WorthQueryProducerIdentityDenial::MissingSourcePartition)?,
                request_admission.remaining_work(),
            )
            .map_err(|()| WorthQueryProducerIdentityDenial::LineageLookupBudgetExceeded)?;
        request_admission
            .charge_external_work(lookup_work as u64)
            .map_err(|_| WorthQueryProducerIdentityDenial::LineageLookupBudgetExceeded)?;
        let force_successor = successor_of.is_some_and(|stale_key| {
            head.as_ref()
                .is_some_and(|head| head.idempotency_key_identity == stale_key)
        });
        let occurrence = self
            .read_set
            .lease
            .product()
            .observation()
            .lifecycle_incarnation();
        if let Some(head) = head.as_ref().filter(|head| head.occurrence == occurrence) {
            // A head that consumed upstream outputs is replayed only while a
            // row posts it: that row holds its claims, and a replay publishes
            // none. A head that consumed none is replayed whenever its
            // dependencies still match, and its row posts it again.
            let replays = !force_successor && head.dependency_identity == Some(dependency);
            let unposted = (head.claims_upstream || !replays)
                && !runtime
                    .output_demands
                    .posts_settlement(&head.settlement, request_admission)
                    .map_err(|_| WorthQueryProducerIdentityDenial::LineageLookupBudgetExceeded)?;
            if replays && !unposted {
                return Ok((head.idempotency_key_identity, dependency));
            }
            // This execution commits as the head's successor. Its record
            // displaces the head, whose settlement the publication retires.
        }
        let prior_head = head
            .map(|head| head.idempotency_key_identity)
            .or(successor_of);
        let (key, work) = lineage_identity(dependency, prior_head)
            .map_err(|_| WorthQueryProducerIdentityDenial::LineageCanonicalizationRejected)?;
        self.read_set
            .admission
            .retain_execution_canonical_work(work);
        Ok((key, dependency))
    }
}

#[cfg(test)]
mod tests;
