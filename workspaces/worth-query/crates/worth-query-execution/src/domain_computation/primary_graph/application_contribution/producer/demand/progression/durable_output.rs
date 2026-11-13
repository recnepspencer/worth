//! Preserve exact accepted facts across quiescent Directory teardown.
use super::*;
use crate::domain_computation::primary_graph::application_checkpoint::{self as checkpoint, facts};
use crate::domain_computation::primary_graph::application_output_demand::{
    AcceptedCheckpointFactSource as Source, WorthQueryOutputDemandKey,
};
impl<S: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub(super) fn retain_accepted_output(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        key: &WorthQueryOutputDemandKey,
        basis: Option<&worth_runtime_world::facade::CompositeCommitIdentity>,
        occurrence: Option<worth_runtime_world::facade::ProductBranchIncarnation>,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let Some(basis) = basis.filter(|basis| basis.owner_identity().durable().is_some()) else {
            return Ok(());
        };
        let support = self.installed_program_support().ok_or_else(unavailable)?;
        let Some((mut identity, source, _)) =
            self.runtime.output_demands.accepted_checkpoint_record(key)
        else {
            return Ok(());
        };
        if let Some(source) = source {
            let lineage = self
                .runtime
                .primary_provider
                .graph
                .output_lineage
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            identity.producer_facts = match source {
                Source::Committed(receipt) => lineage
                    .checkpoint_facts_for_publication((&receipt).into())
                    .and_then(|(facts, witness)| {
                        checkpoint::capture::output_facts::encode(&facts, witness)
                    }),
                Source::Recovered(receipt) => lineage
                    .checkpoint_facts_for_publication(receipt.view())
                    .and_then(|(facts, witness)| {
                        checkpoint::capture::output_facts::encode(&facts, witness)
                    }),
                Source::Stable(stable) => stable.checkpoint_source_facts().and_then(|source| {
                    match stable.checkpoint_output_witness() {
                        Some(witness) => {
                            checkpoint::capture::output_facts::encode(&source, witness)
                        }
                        None => facts::encode(&source),
                    }
                }),
            }
            .map(std::sync::Arc::new);
            identity.producer_fact_wire_version = if identity.producer_facts.is_some() {
                facts::WIRE_VERSION
            } else {
                0
            };
        }
        let payload =
            checkpoint::encode::body::encode_accepted(&identity, execution.allocation_policy())
                .map_err(|_| unavailable())?;
        let observation = self
            .on_branch(crate::basis::WorthQueryProductBranch::from_occurrence(
                occurrence.ok_or_else(unavailable)?,
            ))
            .select_with_request(execution)
            .map_err(|cause| {
                WorthQueryOutputDemandDenial::product_selection(cause, "accepted output retention")
            })?;
        let commit = worth_foundational::facade::DurableCompositeCommitIdentity::from_recorded(
            basis.owner_identity().durable().expect("durable checked"),
            basis.ordinal(),
        );
        support.lifecycle().record_accepted_output(execution, crate::domain_computation::primary_graph::query_obligations::QueryAcceptedOutputRecord {
            commit, branch: observation.product().observation().branch_identity().name().as_str().to_owned(), producer: identity.producer.as_ref().clone(),
            scope: (identity.scope.partition_id(),identity.scope.local_slot(),identity.scope.generation()), partition: identity.source_partition, payload,
        }).map_err(|cause| WorthQueryOutputDemandDenial::new(WorthQueryOutputDemandDenialKind::ObligationPublication(cause), "accepted output retention"))
    }
}
fn unavailable() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
        "accepted output retention",
    )
}

impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    /// Source publication already completed these exact coupled writers. Save
    /// their existing immutable facts before caller-held source custody escapes.
    pub(in crate::domain_computation::primary_graph) fn retain_coupled_output_records(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
        observation: &worth_runtime_world::facade::ProductBranchObservation,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let Some(owner) = observation.owner_identity().durable() else {
            return Ok(());
        };
        let provider = &self.runtime.primary_provider;
        let committed = provider
            .observe_completed_application(receipt.commit_reference())
            .ok_or_else(unavailable)?;
        let identities = {
            let lineage = provider
                .graph
                .output_lineage
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let mut identities = Vec::new();
            for identity in committed
                .publication_members()
                .filter(|member| member.is_coupled_member())
                .filter_map(|member| member.exact_output_settlement())
            {
                if let Some(identity) = lineage.checkpoint_identity_for_settlement(identity)? {
                    identities.push(identity);
                }
            }
            identities
        };
        let support = self.installed_program_support().ok_or_else(unavailable)?;
        let commit = worth_foundational::facade::DurableCompositeCommitIdentity::from_recorded(
            owner,
            observation.selected_commit().ordinal(),
        );
        for identity in identities {
            let payload =
                checkpoint::encode::body::encode_accepted(&identity, execution.allocation_policy())
                    .map_err(|_| unavailable())?;
            support.lifecycle().record_accepted_output(execution,
                crate::domain_computation::primary_graph::query_obligations::QueryAcceptedOutputRecord {
                    commit,
                    branch: observation.branch_identity().name().as_str().to_owned(),
                    producer: identity.producer.as_ref().clone(),
                    scope: (identity.scope.partition_id(), identity.scope.local_slot(), identity.scope.generation()),
                    partition: identity.source_partition,
                    payload,
                }).map_err(|cause| WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::ObligationPublication(cause), "coupled output retention"))?;
        }
        Ok(())
    }
}
