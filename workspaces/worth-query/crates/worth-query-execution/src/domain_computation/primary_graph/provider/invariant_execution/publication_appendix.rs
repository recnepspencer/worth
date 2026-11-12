//! Hand the sealed producer body and real receipt writer to Native in order.
use super::*;
use crate::domain_computation::primary_graph::{
    output_lineage::ExecutedAdoptionSelection,
    provider::{ReceiptAppendix, WorthQueryPrimaryGraphApplicationAttempt},
};
use worth_relational::facade::mvcc::{
    RelationalPublicationAdoptionRequest, RequiredOutputProducingImplementation,
};

pub(super) struct PublicationAppendixPreparation {
    receipt: ReceiptAppendix,
    producer: String,
    family: &'static str,
    scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    partition: [u8; 32],
    selection: ExecutedAdoptionSelection,
    implementation: RequiredOutputProducingImplementation,
    roles: super::super::native_publication_roles::funded::FundedPublicationRoles,
}
impl PublicationAppendixPreparation {
    pub(super) fn from_attempt(
        attempt: &WorthQueryPrimaryGraphApplicationAttempt,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        layout: &crate::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryGraphLayout,
        policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<Option<Self>, WorthQueryInvariantExecutionFailure> {
        if !attempt.is_required_output_completion()
            || attempt.has_nonrelational_completion_effects()
        {
            return Ok(None);
        }
        // Created or explicitly retired roles cannot equal a live Preserve
        // roster. Declining this optimization leaves all admission to Native.
        for (_, posture, _, reference) in attempt.native_publication_roles() {
            crate::domain_computation::primary_graph::source_facts::SourceFactsAllocation::work(
                execution, 1,
            )
            .map_err(|cause| {
                super::super::invariant_execution_failure::companion_failure(
                    worth_relational::facade::mvcc::CompanionPreflightStop::Execution(cause),
                )
            })?;
            if posture != crate::domain_computation::primary_graph::WorthQueryApplicationOutputPosture::Preserve
                || !matches!(reference, worth_relational::facade::transactions::EntityReference::Existing(_)) {
                return Ok(None);
            }
        }
        let Some(receipt) = attempt.receipt_appendix() else {
            return Ok(None);
        };
        let Some(producer) = attempt.output_producer() else {
            return Ok(None);
        };
        let Some(family) = attempt.output_family() else {
            return Ok(None);
        };
        let Some(partition) = attempt.idempotency().source_partition_identity() else {
            return Ok(None);
        };
        let Some(selection) = ExecutedAdoptionSelection::from_attempt(attempt) else {
            return Ok(None);
        };
        let Some(implementation) = attempt.idempotency().producing_implementation() else {
            return Ok(None);
        };
        let roles = super::super::native_publication_roles::funded::retain(
            execution,
            layout,
            attempt.native_publication_roles(),
            producer.len(),
            policy,
        )?;
        Ok(Some(Self {
            receipt,
            producer: producer.to_owned(),
            family,
            scope: attempt.affinity().operation_scope().scope(),
            partition,
            selection,
            implementation,
            roles,
        }))
    }
}

pub(super) struct SelectedPublicationAppendix {
    receipt: ReceiptAppendix,
    producer: String,
    family: &'static str,
    scope: crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding,
    partition: [u8; 32],
    previous: crate::domain_computation::primary_graph::output_lineage::required_publication::head::SelectedRequiredPublicationHead,
    implementation: RequiredOutputProducingImplementation,
    roles: super::super::native_publication_roles::funded::FundedPublicationRoles,
}

pub(super) fn select(
    provider: &WorthQueryPrimaryGraphProvider,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    material: &mut ApplicationInvariantCandidateMaterial,
) -> Result<Option<SelectedPublicationAppendix>, WorthQueryInvariantExecutionFailure> {
    let Some(prepared) = material.publication_appendix.take() else {
        return Ok(None);
    };
    if provider.graph.application_publication.is_none() {
        return Ok(None);
    }
    let selected = prepared
        .selection
        .select_native(&provider.graph.output_lineage, execution)
        .map_err(super::super::invariant_execution_failure::companion_failure)?;
    let Some(selected) = selected else {
        return Ok(None);
    };
    let previous = selected
        .required_publication_head()
        .map_err(super::super::invariant_execution_failure::companion_failure)?;
    // A cold stale output has not established a live mark cell. Mint through
    // its existing Native owner before any candidate companion preflight;
    // this installs derived state at C, never a publication or durable write.
    provider
        .graph
        .source_owner
        .mint_mark_cell_at_head(execution, prepared.selection.native_branch_id())
        .map_err(super::super::invariant_execution_failure::head_cell_failure)?;
    Ok(Some(SelectedPublicationAppendix {
        receipt: prepared.receipt,
        producer: prepared.producer,
        family: prepared.family,
        scope: prepared.scope,
        partition: prepared.partition,
        previous,
        implementation: prepared.implementation,
        roles: prepared.roles,
    }))
}

impl SelectedPublicationAppendix {
    pub(super) fn judge(
        self,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        basis: &worth_relational::facade::branch::AdmittedRelationalBranchBasis,
        batch: worth_relational::facade::transactions::WorkerIntentBatch,
        content: &worth_relational::facade::content::RelationalContentAttachments,
        installed: &worth_relational::facade::mvcc::RelationalInstalledApplicationPublicationAuthority,
        layout: &crate::domain_computation::primary_graph::schema_layout::WorthQueryPrimaryGraphLayout,
    ) -> Result<
        worth_relational::facade::mvcc::RelationalPublicationBodySelection,
        WorthQueryInvariantExecutionFailure,
    > {
        let previous = self.previous.readmit_if_published(runtime, execution, installed, basis,
            self.family, self.scope, self.partition, layout).map_err(|stop| match stop {
                crate::domain_computation::primary_graph::output_lineage::required_publication::head::RequiredPublicationReadmissionStop::Selection(stop) => super::super::invariant_execution_failure::companion_failure(stop),
                crate::domain_computation::primary_graph::output_lineage::required_publication::head::RequiredPublicationReadmissionStop::Native(stop) => super::super::invariant_execution_failure::readmission_failure(stop),
                crate::domain_computation::primary_graph::output_lineage::required_publication::head::RequiredPublicationReadmissionStop::Allocation(stop) => WorthQueryInvariantExecutionFailure::physical_allocation(stop, "current publication role readmission allocation refused"),
            })?;
        let (body, appendix) = self.receipt.detach(batch).map_err(closure_failure)?;
        runtime
            .prepare_publication_appendix(
                execution,
                basis,
                body,
                content,
                appendix,
                previous
                    .as_ref()
                    .map(|previous| RelationalPublicationAdoptionRequest {
                        installed,
                        producer: &self.producer,
                        previous: previous.publication(),
                        implementation: &self.implementation,
                        roles: &self.roles.roles,
                    }),
            )
            .map_err(map_validation_failure)
    }
}
