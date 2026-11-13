//! All participants prepare against one Native candidate and one planned successor.
use super::*;
mod support;
use super::durable_custody::QueryCandidatePreparationDenial as Denial;
use crate::domain_computation::primary_graph::provider::CoupledApplicationMember;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial;
pub(super) use support::CoupledSupportPreparationDenial;
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
) -> Result<(), Denial> {
    let complete = if members.is_empty()
        && root.candidate_read_support().is_empty()
        && root.coupled_after_own().is_none()
    {
        None
    } else {
        Some(
            candidate
                .pin_coupled_candidate(execution)
                .map_err(Denial::Native)?
                .expect("Native retained genuine coupled proposal read custody"),
        )
    };
    if let Some(complete) = &complete {
        if root.coupled_after_own().is_some() {
            let scope = root.affinity().operation_scope().scope();
            let source = worth_relational::facade::identity::EntityId::new(
                worth_relational::facade::identity::PartitionId::new(scope.partition_id()),
                scope.local_slot(),
                scope.generation(),
            );
            support::require(
                execution, provider, root, before, complete, source, None, 0, members,
            )
            .map_err(Denial::CoupledSupport)?;
        }
        support::require_read_support(execution, provider, root, before, complete, members)
            .map_err(Denial::CoupledSupport)?;
        for member in members.iter() {
            let source = member.source();
            let owned = member.owned.as_ref().expect("atomic group extraction");
            support::require(
                execution,
                provider,
                &owned.attempt,
                before,
                complete,
                source,
                owned
                    .commit
                    .as_ref()
                    .and_then(|prepared| prepared.rebase.as_ref()),
                member.position,
                members,
            )
            .map_err(Denial::CoupledSupport)?;
            support::require_read_support(
                execution,
                provider,
                &owned.attempt,
                before,
                complete,
                members,
            )
            .map_err(Denial::CoupledSupport)?;
        }
    }
    for member in members {
        let producer = member.producer_route().is_some();
        let owned = member.owned.as_mut().expect("atomic group extraction");
        let prepared = owned
            .commit
            .as_mut()
            .expect("member evidence admitted before candidate preparation");
        prepared.required =
            consumed_output::verify(execution, provider, &mut owned.attempt, before)
                .map_err(Denial::ConsumedOutput)?
                .required_prerequisites;
        managed_source::bind(execution, provider, before, &owned.attempt, candidate)
            .map_err(Denial::ManagedSource)?;
        prepared.witness = if producer {
            crate::domain_computation::primary_graph::output_lineage::PreparedNativeOutputWitness::prepare(&owned.attempt, &provider.graph.layout)
                .map_err(Denial::OutputWitness)?
        } else {
            None
        };
        owned
            .attempt
            .reserve_coupled_dispatch(provider)
            .map_err(Denial::Dispatch)?;
        let request = owned.attempt.affinity().publication_request();
        if let Some(rebase) = prepared.rebase.take() {
            prepared.rebase = Some(provider.graph.with_runtime(execution, |runtime| rebase.prepare_candidate(
                candidate, runtime, before,
                crate::domain_computation::primary_graph::application_attempt::retained_decision_facts::StorageControl::new(policy, Some(request)),
            )).map_err(Denial::SourceFacts)?);
        }
    }
    Ok(())
}

pub(super) fn prepare_lineage(
    provider: &WorthQueryPrimaryGraphProvider,
    members: &mut [CoupledApplicationMember],
    planned: &worth_runtime_world::facade::PlannedProductReferenceSuccessor,
) -> Result<(), WorthQueryOutputDemandDenial> {
    for member in members {
        let crate::domain_computation::primary_graph::provider::CoupledApplicationMemberKind::Producer { producer, .. } = &member.kind else { continue };
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
                producer,
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
                WorthQueryProviderSessionFailure::allocation_denied(
                    cause,
                    "coupled completion backing",
                ),
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
                member.producer_route().is_some_and(|(producer, meaning)| {
                    producer == publication.producer() && meaning == publication.meaning()
                })
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
        assert!(
            self.members.iter().all(|member| {
                let prepared = member
                    .owned
                    .as_ref()
                    .expect("actual participant")
                    .commit
                    .as_ref()
                    .expect("prepared participant");
                prepared.publication.is_some() == prepared.publication_required
            }),
            "Native finishes exactly the owned participants; ordinary writers have no origin"
        );
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
            coupled_deletions: self.coupled_deletions.clone(),
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
