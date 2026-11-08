//! All participants prepare against one Native candidate and one planned successor.
use super::*;
mod support;
use crate::domain_computation::primary_graph::provider::CoupledApplicationMember;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;
use worth_relational::facade::{
    mvcc::PreparedRelationalCommitCandidate, snapshots::SnapshotHandle,
};

pub(super) fn prepare_members(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    root: &WorthQueryPrimaryGraphApplicationAttempt,
    members: &mut [CoupledApplicationMember],
    candidate: &mut PreparedRelationalCommitCandidate,
    before: &SnapshotHandle,
    policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
) -> Result<(), crate::domain_computation::WorthQueryProviderSessionCommitStop> {
    let complete = if members.is_empty() && root.candidate_read_support().is_empty() {
        None
    } else {
        Some(
            candidate
                .pin_coupled_candidate(execution)
                .map_err(stops::transaction_commit_stop)?
                .expect("Native retained genuine coupled proposal read custody"),
        )
    };
    if !root.candidate_read_support().is_empty() {
        support::require_read_support(
            execution,
            provider,
            root,
            before,
            complete.as_ref().expect("actual edit query candidate"),
        )?;
    }
    for member in members {
        let owned = member.owned.as_mut().expect("atomic group extraction");
        support::require(
            execution,
            provider,
            &owned.attempt,
            before,
            complete.as_ref().expect("coupled complete image"),
            member.meaning.source_scope,
        )?;
        support::require_read_support(
            execution,
            provider,
            &owned.attempt,
            before,
            complete.as_ref().expect("actual member query candidate"),
        )?;
        let prepared = owned
            .commit
            .as_mut()
            .expect("member evidence admitted before candidate preparation");
        prepared.required =
            consumed_output::verify(execution, provider, &mut owned.attempt, before)?
                .required_prerequisites;
        managed_source::bind(execution, provider, before, &owned.attempt, candidate)?;
        prepared.witness = crate::domain_computation::primary_graph::output_lineage::PreparedNativeOutputWitness::prepare(&owned.attempt, &provider.graph.layout).map_err(native_output_witness_stop)?;
        owned.attempt.reserve_coupled_dispatch(provider).map_err(|cause| match cause {
            crate::domain_computation::primary_graph::live_delivery::WorthQueryLiveAdmissionDenial::Execution(cause) => crate::domain_computation::primary_graph::provider::relational_execution_denial::relational_execution_stop(cause, None),
            crate::domain_computation::primary_graph::live_delivery::WorthQueryLiveAdmissionDenial::Unavailable(detail) => crate::domain_computation::WorthQueryProviderSessionCommitStop::PreEffectDenied(super::failure(detail)),
        })?;
        let request = owned.attempt.affinity().publication_request();
        prepared.rebase = Some(provider.graph.with_runtime(execution, |runtime| prepared.rebase.take().expect("one rebase preparation").prepare_candidate(candidate, runtime, before,
            crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl::new(policy, Some(request))
        )).map_err(stops::source_rebase_preparation_stop)?);
    }
    Ok(())
}

pub(super) fn prepare_lineage(
    provider: &WorthQueryPrimaryGraphProvider,
    members: &mut [CoupledApplicationMember],
    planned: &worth_runtime_world::facade::PlannedProductReferenceSuccessor,
) -> Result<(), WorthQueryOutputDemandDenial> {
    for member in members {
        let owned = member.owned.as_mut().expect("group owns writer");
        let prepared = owned.commit.as_mut().expect("group owns prepared evidence");
        if let Some(slot) = prepared.slot.as_ref()
            && slot.matches_planned_successor(
                owned.attempt.affinity().operation_scope(),
                owned
                    .attempt
                    .output_binding_type()
                    .expect("producer output binding"),
                owned.attempt.idempotency().source_partition_identity(),
                planned,
            )?
        {
            continue;
        }
        let mut slot =
            crate::domain_computation::primary_graph::output_lineage::prepare_output_lineage_slot(
                &provider.graph.output_lineage,
                owned.attempt.affinity().operation_scope(),
                owned
                    .attempt
                    .output_binding_type()
                    .expect("producer output binding"),
                owned.attempt.idempotency().source_partition_identity(),
                planned,
            )?;
        if let Some(prior) = prepared.slot.take() {
            slot.retain_recovery_metadata(prior.into_recovery_metadata());
        } else {
            let mut source = owned
                .attempt
                .take_coupled_preparation()
                .expect("actual registered producer source and completed handler");
            slot.retain_native_prior_checkpoint(
                &member.producer,
                source.completion.source().checkpoint_identity(),
            )?;
            slot.retain_coupled_completion(source.completion);
            slot.retain_completed_handler_facts(
                source
                    .handler
                    .take_completed_handler_facts()
                    .expect("actual handler completion"),
            );
            let (sealed, prior) = source.handler.take_sealed_computation();
            slot.retain_computation(sealed, prior);
            if let Some(proof) = source.handler.take_completed_decision_reuse() {
                slot.retain_completed_decision_reuse(proof);
            }
            if let Some(key) = source.handler.take_prepared_input_reuse_key() {
                slot.retain_prepared_input_reuse_key(key);
            }
        }
        prepared.slot = Some(slot);
    }
    Ok(())
}

pub(super) type CoupledEvidenceSlots = std::sync::Arc<
    worth_execution::ExecutionArray<
        std::sync::OnceLock<publication::WorthQueryPrimaryGraphCommittedApplication>,
    >,
>;
pub(super) fn prepare_evidence_slots(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    count: usize,
) -> Result<
    Option<CoupledEvidenceSlots>,
    crate::domain_computation::WorthQueryProviderSessionCommitStop,
> {
    if count == 0 {
        return Ok(None);
    }
    let mut slots = worth_execution::ExecutionRequestArrayBuilder::allocate(*execution, count)
        .map_err(slot_denial)?;
    for _ in 0..count {
        slots
            .push(std::sync::OnceLock::new())
            .map_err(slot_denial)?;
    }
    Ok(Some(std::sync::Arc::new(
        slots.seal().map_err(slot_denial)?,
    )))
}
fn slot_denial(
    cause: worth_execution::ExecutionRequestArrayDenial,
) -> crate::domain_computation::WorthQueryProviderSessionCommitStop {
    match cause {
        worth_execution::ExecutionRequestArrayDenial::Request(cause) => {
            publication_request_stop(cause)
        }
        worth_execution::ExecutionRequestArrayDenial::Allocation(cause) => {
            crate::domain_computation::WorthQueryProviderSessionCommitStop::PreEffectDenied(
                WorthQueryProviderSessionFailure::allocation_denied(cause),
            )
        }
    }
}
impl WorthQueryCommittedApplicationSession {
    pub(super) fn finish_group_origins(&mut self) {
        let Some(prepared) = self.prepared_group_publications.take() else {
            return;
        };
        for publication in prepared
            .finish(&self.committed)
            .expect("group custody finishes only its performed Native envelope")
            .into_publications()
        {
            if let Some(member) = self.members.iter_mut().find(|member| {
                member.producer == publication.producer()
                    && &member.meaning == publication.meaning()
            }) {
                let slot = &mut member
                    .owned
                    .as_mut()
                    .expect("actual extracted writer")
                    .commit
                    .as_mut()
                    .expect("prepared writer")
                    .publication;
                assert!(
                    slot.replace(publication).is_none(),
                    "one performed origin for each admitted writer"
                );
            }
        }
        assert!(self.members.iter().all(|member| member
            .owned
            .as_ref()
            .unwrap()
            .commit
            .as_ref()
            .unwrap()
            .publication
            .is_some()));
    }
    pub(super) fn take_member_session(
        &self,
        member: &mut CoupledApplicationMember,
    ) -> WorthQueryCommittedApplicationSession {
        let owned = member
            .owned
            .take()
            .expect("one transfer from extracted writer to performed evidence");
        let prepared = owned.commit.expect("pre-effect writer preparation");
        WorthQueryCommittedApplicationSession {
            attempt: owned.attempt,
            work: owned.work,
            members: Vec::new(),
            prepared_group_publications: None,
            group_evidence: None,
            group_emissions: None,
            own_publication: prepared.publication,
            own_records: Some(prepared.records),
            predicate: None,
            index_maintenance_work: Default::default(),
            retained_preimage: prepared.retained_preimage,
            preimage_retention_work: prepared.preimage_work,
            before: self.before.clone(),
            next_basis: self.next_basis.clone(),
            committed: std::sync::Arc::clone(&self.committed),
            published_snapshot_custody: PublishedSnapshotCustody::SettledReleased,
            prepared_touched_records: prepared.touched,
            product_publication: self.product_publication.clone(),
            managed_views: None,
            source_fact_rebase: prepared.rebase,
            required_prerequisites: prepared.required,
            prepared_lineage_slot: prepared.slot,
            prepared_output_witness: prepared.witness,
        }
    }
}

pub(super) fn prepare_emissions(execution: &worth_execution::ExecutionRequest<'_, '_>, root: &WorthQueryPrimaryGraphApplicationAttempt, members: &[CoupledApplicationMember]) -> Result<Option<crate::domain_computation::primary_graph::application_attempt::WorthQueryAdmittedApplicationEmissionBatch>, crate::domain_computation::WorthQueryProviderSessionCommitStop>{
    if members.is_empty() {
        return Ok(None);
    }
    let batches = std::iter::once(root.admitted_emissions()).chain(members.iter().map(|member| {
        member
            .owned
            .as_ref()
            .expect("extracted writer")
            .attempt
            .admitted_emissions()
    }));
    crate::domain_computation::primary_graph::application_attempt::WorthQueryAdmittedApplicationEmissionBatch::combine(execution, batches).map(Some)
}
