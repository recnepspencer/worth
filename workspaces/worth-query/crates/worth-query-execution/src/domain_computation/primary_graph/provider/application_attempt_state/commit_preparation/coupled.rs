//! Per-writer evidence reserved before the shared World movement.
use super::relational_commit::PreparedTouchedRecords;
use super::*;
use crate::domain_computation::WorthQueryProviderSessionCommitStop;
use worth_execution::{ExecutionArray, ExecutionArrayBuilder};
use worth_relational::facade::{mvcc::ValidatedRelationalProposal, transactions::RecordRef};

pub(in crate::domain_computation::primary_graph::provider) struct CoupledCommitPreparation {
    pub(in crate::domain_computation::primary_graph::provider) publication:
        Option<worth_relational::facade::mvcc::RelationalRequiredOutputPublication>,
    pub(in crate::domain_computation::primary_graph::provider) records: ExecutionArray<RecordRef>,
    pub(in crate::domain_computation::primary_graph::provider) touched:
        Option<PreparedTouchedRecords>,
    pub(in crate::domain_computation::primary_graph::provider) rebase:
        Option<relational_commit::PreparedSourceFactRebase>,
    pub(in crate::domain_computation::primary_graph::provider) retained_preimage:
        Option<WorthQueryRetainedPreImage>,
    pub(in crate::domain_computation::primary_graph::provider) preimage_work:
        WorthQueryPreImageRetentionWork,
    pub(in crate::domain_computation::primary_graph::provider) required:
        Option<crate::domain_computation::primary_graph::PreparedPrerequisiteClaims>,
    pub(in crate::domain_computation::primary_graph::provider) slot:
        Option<crate::domain_computation::primary_graph::output_lineage::PreparedOutputLineageSlot>,
    pub(in crate::domain_computation::primary_graph::provider) witness: Option<
        crate::domain_computation::primary_graph::output_lineage::PreparedNativeOutputWitness,
    >,
}

pub(super) fn prepare(
    members: &mut [super::super::CoupledApplicationMember],
    candidate: &ValidatedRelationalProposal,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    layout: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout,
    policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
) -> Result<(), WorthQueryProviderSessionFailure> {
    for member in members {
        let owned = member
            .owned
            .as_mut()
            .expect("atomic extraction retains every participant");
        let exact = candidate
            .coupled_contribution_records(&member.producer, &member.meaning)
            .expect("Query checked the exact Native participant routes");
        let mut records = ExecutionArrayBuilder::allocate(exact.len(), policy)
            .map_err(WorthQueryProviderSessionFailure::allocation_denied)?;
        for record in exact {
            records
                .push(record.clone())
                .map_err(WorthQueryProviderSessionFailure::allocation_denied)?;
        }
        let touched =
            PreparedTouchedRecords::from_records(exact, policy).map_err(|stop| match stop {
                WorthQueryProviderSessionCommitStop::PreEffectDenied(failure) => failure,
                _ => unreachable!("record storage returns only physical allocation refusal"),
            })?;
        let (retained_preimage, preimage_work) = preimage_retention::retain_attempt_preimage(
            &owned.attempt,
            candidate,
            execution,
            layout,
            policy,
        )?
        .into_parts();
        owned.commit = Some(CoupledCommitPreparation {
            publication: None,
            records: records.seal().map_err(WorthQueryProviderSessionFailure::allocation_denied)?,
            touched: Some(touched), rebase: Some(owned.attempt.take_source_fact_rebase().with_predecessor(candidate.output_observation_predecessor(execution, &member.producer, &owned.attempt.idempotency().producer_dependency_identity().expect("actual coupled producer dependency")).map_err(|cause| crate::domain_computation::primary_graph::provider::session_commit::provider_failure(WorthQueryProviderSessionProtocolStage::Commit, "producer predecessor classification refused").with_native_preparation_error(cause))?)), retained_preimage, preimage_work,
            required: None, slot: None, witness: None,
        });
    }
    Ok(())
}
