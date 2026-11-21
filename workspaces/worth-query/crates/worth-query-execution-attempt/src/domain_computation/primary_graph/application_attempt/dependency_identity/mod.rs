use worth_query_execution_lineage::facade::ports::PrimaryGraphIntegrationHandleInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;
pub use worth_query_execution_lineage::facade::primary_graph::application_attempt::dependency_identity::*;
pub use worth_query_execution_lineage::facade::primary_graph::application_attempt::dependency_identity::content_tree::CompletedFactRoutes;
pub use worth_query_execution_lineage::facade::primary_graph::application_attempt::dependency_identity::output_postcondition::OutputCurrentnessFacts;
pub mod producing_implementation;

use worth_query_execution_lineage::facade::primary_graph::application_attempt::dependency_identity::canonical_encoding;
pub use worth_query_execution_lineage::facade::primary_graph::application_attempt::dependency_identity::FactContent;
use worth_query_execution_lineage::facade::primary_graph::application_attempt::dependency_identity::output_postcondition::complete_content_currentness_facts;

use super::retained_decision_facts::{StorageControl, StoreDenial};
use super::WorthQueryApplicationEffectProgram;
use canonical_encoding::lineage_identity;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryProducerIdentityDenial {
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
    pub fn producer_idempotency_identities<OutputBinding>(
        &mut self,
        runtime: &dyn crate::ports::ProducerIdentityRuntimePort,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        declared_key: [u8; 32],
        implementation: worth_relational::facade::mvcc::RequiredOutputProducingImplementation,
        successor_of: Option<[u8; 32]>,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
        retain_output_field_evidence: bool,
    ) -> Result<([u8; 32], [u8; 32]), WorthQueryProducerIdentityDenial>
    where
        OutputBinding: 'static,
    {
        self.producer_identities::<OutputBinding>(
            runtime,
            execution,
            declared_key,
            implementation,
            successor_of,
            None,
            policy,
            retain_output_field_evidence,
        )
    }

    pub(crate) fn coupled_producer_identities<OutputBinding: 'static>(
        &mut self,
        runtime: &dyn crate::ports::ProducerIdentityRuntimePort,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        declared_key: [u8; 32],
        implementation: worth_relational::facade::mvcc::RequiredOutputProducingImplementation,
        predecessor: Option<&worth_relational::facade::mvcc::RelationalRequiredOutputPublication>,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
        retain_output_field_evidence: bool,
    ) -> Result<([u8; 32], [u8; 32]), WorthQueryProducerIdentityDenial> {
        self.producer_identities::<OutputBinding>(
            runtime,
            execution,
            declared_key,
            implementation,
            None,
            predecessor,
            policy,
            retain_output_field_evidence,
        )
    }

    fn producer_identities<OutputBinding: 'static>(
        &mut self,
        runtime: &dyn crate::ports::ProducerIdentityRuntimePort,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        declared_key: [u8; 32],
        implementation: worth_relational::facade::mvcc::RequiredOutputProducingImplementation,
        successor_of: Option<[u8; 32]>,
        required_predecessor: Option<
            &worth_relational::facade::mvcc::RelationalRequiredOutputPublication,
        >,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
        retain_output_field_evidence: bool,
    ) -> Result<([u8; 32], [u8; 32]), WorthQueryProducerIdentityDenial> {
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
        let performed_inputs =
            crate::domain_computation::primary_graph::application_checkpoint::facts::FactPayload::copy_performed_field_reads(
                &self.read_set.facts,
                control,
            )
            .map_err(WorthQueryProducerIdentityDenial::DependencyStorage)?;
        let (dependency_facts, content, moved, normalization_work) = original_content
            .postcondition(
                &self.effects,
                &self.read_set.computation_fact_ordinals,
                control,
            )
            .map_err(WorthQueryProducerIdentityDenial::DependencyStorage)?;
        let root_funding = control.policy().reserve_memory(8_192);
        control
            .check_live()
            .map_err(WorthQueryProducerIdentityDenial::DependencyStorage)?;
        let _root_funding = root_funding
            .map_err(|cause| WorthQueryProducerIdentityDenial::DependencyStorage(cause.into()))?;
        let (fact_dependency, fact_work) = content
            .identity(declared_key)
            .map_err(|_| WorthQueryProducerIdentityDenial::DependencyCanonicalizationRejected)?;
        control
            .check_live()
            .map_err(WorthQueryProducerIdentityDenial::DependencyStorage)?;
        let (dependency, implementation_work) =
            producing_implementation::identity(fact_dependency, implementation).map_err(|_| {
                WorthQueryProducerIdentityDenial::DependencyCanonicalizationRejected
            })?;
        let work = fact_work.combine(implementation_work);
        self.output_currentness_facts = Some(
            complete_content_currentness_facts(dependency_facts, moved, content.reusable())
                .with_rebase_carry(self.read_set.postcondition_carry.clone())
                .with_performed_inputs(performed_inputs)
                .retaining_output_field_evidence(retain_output_field_evidence),
        );
        self.read_set
            .admission
            .retain_execution_canonical_work(cold_work.combine(normalization_work).combine(work));
        let partition = self
            .read_set
            .admission
            .source_partition_identity()
            .ok_or(WorthQueryProducerIdentityDenial::MissingSourcePartition)?;
        let lineage = runtime
            .primary_provider()
            .graph
            .output_lineage_ref()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let head = lineage
            .producer_head::<OutputBinding>(
                self.read_set.admission.operation_scope_binding(),
                self.read_set.lease.product().observation(),
                partition,
                execution,
            )
            .map_err(|stop| match stop {
                worth_relational::facade::mvcc::CompanionPreflightStop::Execution(cause) => {
                    WorthQueryProducerIdentityDenial::Execution(cause)
                }
                _ => WorthQueryProducerIdentityDenial::LineageLookupBudgetExceeded,
            })?;
        drop(lineage);
        let force_successor = required_predecessor.is_some()
            || successor_of.is_some_and(|stale_key| {
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
            let replays = head.may_replay_idempotency
                && !force_successor
                && head.dependency_identity == Some(dependency);
            let unposted = (head.claims_upstream || !replays)
                && !runtime
                    .posts_settlement(&head.settlement)
                    .map_err(|_| WorthQueryProducerIdentityDenial::LineageLookupBudgetExceeded)?;
            if replays && !unposted {
                return Ok((head.idempotency_key_identity, dependency));
            }
        }
        let prior_head = head
            .map(|head| head.idempotency_key_identity)
            .or(successor_of);
        let (key, work) = match (prior_head, required_predecessor) {
            (None, Some(predecessor)) => canonical_encoding::native_parent_lineage_identity(
                dependency,
                predecessor.authority().publication_coordinate(),
            ),
            _ => lineage_identity(dependency, prior_head),
        }
        .map_err(|_| WorthQueryProducerIdentityDenial::LineageCanonicalizationRejected)?;
        self.read_set
            .admission
            .retain_execution_canonical_work(work);
        Ok((key, dependency))
    }
}
