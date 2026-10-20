use super::WorthQueryApplicationEffectProgram;

mod canonical_encoding;
mod content_rule;
mod content_tree;
mod output_postcondition;

use super::retained_decision_facts::{StorageControl, StoreDenial};
pub(super) use canonical_encoding::authentication_observation_identity;
use canonical_encoding::lineage_identity;
#[cfg(test)]
use content_tree::dependency_identity;
pub(in crate::domain_computation::primary_graph) use content_tree::{
    CompletedFactRoutes, FactContent,
};
use output_postcondition::complete_content_currentness_facts;
#[cfg(test)]
use output_postcondition::normalized_output_facts;
pub(in crate::domain_computation::primary_graph) use output_postcondition::OutputCurrentnessFacts;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum WorthQueryProducerIdentityDenial {
    DependencyByteCapacityUnsupported,
    DependencyStorage(StoreDenial),
    DependencyCanonicalizationRejected,
    MissingSourcePartition,
    LineageLookupBudgetExceeded,
    Execution(worth_relational::facade::transactions::RelationalExecutionDenialCause),
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
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        declared_key: [u8; 32],
        successor_of: Option<[u8; 32]>,
        request_admission: &mut crate::domain_computation::primary_graph::output_lineage::invalidation::InvalidationEditAdmission,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<([u8; 32], [u8; 32]), WorthQueryProducerIdentityDenial>
    where
        OutputBinding: 'static,
    {
        let control =
            StorageControl::new(policy, Some(self.read_set.admission.publication_request()));
        control
            .check_live()
            .map_err(WorthQueryProducerIdentityDenial::DependencyStorage)?;
        let (original_content, cold_work) = match &self.read_set.content {
            Some(content) if content.same_body(&self.read_set.facts) => (
                content.clone(),
                worth_query_installation::facade::WorthQueryCanonicalWorkEvidence::zero(),
            ),
            _ => FactContent::build(self.read_set.facts.clone(), control)
                .map_err(WorthQueryProducerIdentityDenial::DependencyStorage)?,
        };
        let (dependency_facts, content, moved, normalization_work) = original_content
            .postcondition(
                &self.effects,
                &self.read_set.computation_fact_ordinals,
                control,
            )
            .map_err(WorthQueryProducerIdentityDenial::DependencyStorage)?;
        let maximum_dependency_bytes = usize::try_from(
            runtime
                .runtime
                .application_candidate_resource_profile()
                .maximum_producer_dependency_bytes(),
        )
        .map_err(|_| WorthQueryProducerIdentityDenial::DependencyByteCapacityUnsupported)?;
        let root_funding = control.policy().reserve_memory(8_192);
        control
            .check_live()
            .map_err(WorthQueryProducerIdentityDenial::DependencyStorage)?;
        let _root_funding = root_funding
            .map_err(|cause| WorthQueryProducerIdentityDenial::DependencyStorage(cause.into()))?;
        let (dependency, work) = content
            .identity(declared_key, maximum_dependency_bytes)
            .map_err(|_| WorthQueryProducerIdentityDenial::DependencyCanonicalizationRejected)?;
        control
            .check_live()
            .map_err(WorthQueryProducerIdentityDenial::DependencyStorage)?;
        self.output_currentness_facts = Some(
            complete_content_currentness_facts(dependency_facts, moved, content.reusable())
                .with_rebase_carry(self.read_set.postcondition_carry.clone()),
        );
        self.read_set
            .admission
            .retain_execution_canonical_work(cold_work.combine(normalization_work).combine(work));
        // The lineage head lookup is framework preparation on the request
        // meter, reserved before it reads.
        let partition = self
            .read_set
            .admission
            .source_partition_identity()
            .ok_or(WorthQueryProducerIdentityDenial::MissingSourcePartition)?;
        let lineage = runtime
            .primary_provider
            .graph
            .output_lineage
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let head = lineage
            .producer_head::<OutputBinding>(
                self.read_set.admission.operation_scope_binding(),
                self.read_set.lease.product().observation(),
                partition,
                execution,
                request_admission,
            )
            .map_err(|stop| match stop {
                worth_relational::facade::mvcc::CompanionPreflightStop::Execution(cause) => {
                    WorthQueryProducerIdentityDenial::Execution(cause)
                }
                _ => WorthQueryProducerIdentityDenial::LineageLookupBudgetExceeded,
            })?;
        drop(lineage);
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
            // Only a runtime performed origin permits execution-key replay;
            // checkpoint prior identity does not retain its native receipt.
            // A head that consumed upstream outputs is replayed only while a
            // row posts it: that row holds its claims, and a replay publishes
            // none. A head that consumed none is replayed whenever its
            // dependencies still match, and its row posts it again.
            let replays = head.may_replay_idempotency
                && !force_successor
                && head.dependency_identity == Some(dependency);
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
