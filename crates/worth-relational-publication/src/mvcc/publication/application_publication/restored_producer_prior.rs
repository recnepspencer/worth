//! A Native-restored publication and its actual producer antecedent, without replay inputs.
use worth_relational_contracts::internal_port as contracts;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::coupled_authored::CanonicalCommitEnvelopeBodyOwnerPort as _;
use super::*;
use worth_relational_branch::internal_port::branch::basis::AdmittedRelationalBranchBasis;
use contracts::execution::request_stage::checkpoint;
use worth_relational_branch::internal_port::history::data::codec_retirement_closure::history_request::RetirementHistoryInputs;
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::{CanonicalOutputPublication, CanonicalRequiredOutputOrigin};
use worth_relational_branch::internal_port::history::data::codec_retirement_closure::CodecRetirementClosureStop as Stop;
use contracts::identity::data::record_ids::EntityId;
use contracts::transactions::data::primitives::RecordRef;
use std::sync::Arc;
use worth_execution::ExecutionRequest;
use contracts::publication::data::publication_error::PublicationErrorOwnerPort as _;

pub struct RelationalRestoredProducerPrior {
    publication: RelationalRequiredOutputPublication,
    antecedent: Arc<CanonicalRequiredOutputOrigin>,
}
impl RelationalRestoredProducerPrior {
    pub fn publication(&self) -> &RelationalRequiredOutputPublication {
        &self.publication
    }
    pub fn producer_scope(&self) -> EntityId {
        self.antecedent.meaning.source_scope
    }
    pub fn matches_producer_publication(
        &self,
        previous: &RelationalRequiredOutputPublication,
    ) -> bool {
        previous.runtime_instance_id == self.publication.runtime_instance_id
            && previous.origin.commit == self.antecedent.commit
            && previous.origin.meaning == self.antecedent.meaning
            && previous.origin.producer == self.antecedent.producer
            && previous.origin.dependency_identity == self.antecedent.dependency_identity
    }
}

pub fn readmit_restored_producer_prior(
    inputs: RetirementHistoryInputs<'_>,
    execution: &ExecutionRequest<'_, '_>,
    installed: &RelationalInstalledApplicationPublicationAuthority,
    basis: &AdmittedRelationalBranchBasis,
    meaning: &RequiredOutputPublicationMeaning,
) -> Result<Option<RelationalRestoredProducerPrior>, TransactionCommitError> {
    let Some(publication) =
        crate::publication::ownership::native_transition_succession::readmit_native_output_transition_successor(
            inputs, execution, installed, basis, meaning,
        )?
    else {
        return Ok(None);
    };
    if !publication.is_authored_restoration() && !publication.is_historical_selection() {
        return Ok(None);
    }
    let antecedent = execution
        .run_native_with_checkpoint(|context| {
            let mut validated_history = worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::origin_history::OutputOriginHistory::default();
            let mut current = validated_history
                .by_commit(
                    publication.origin.commit.commit_id,
                    &mut |commit| {
                        inputs
                            .canonical_history
                            .demand_commit_envelope(execution, commit)
                            .map_err(Stop::Request)
                    },
                    execution,
                    context,
                )
                .map_err(custody::cold_stop)?
                .ok_or_else(|| corrupt(publication.origin.meaning.source_scope))?;
            let mut origin = Arc::clone(&publication.origin);
            let bound = inputs.projection.history.catalog_len();
            for _ in 0..=bound {
                if matches!(
                    origin.publication,
                    CanonicalOutputPublication::HistoricalSelection { .. }
                ) {
                    checkpoint(context, 1).map_err(custody::execution_stop)?;
                    let meaning = origin.meaning.clone();
                    current = worth_relational_branch::internal_port::history::data::codec_retirement_closure::historical_selection::antecedent(
                        inputs.canonical_history,
                        execution,
                        &current,
                        &origin,
                        &mut validated_history,
                        context,
                    )
                    .map_err(custody::cold_stop)?;
                    origin = current
                        .output_origin_for_route(&meaning, context)
                        .map(Arc::clone)
                        .map_err(custody::cold_stop)?;
                    continue;
                }
                let CanonicalOutputPublication::AuthoredRestoration { predecessor, .. } =
                    origin.publication
                else {
                    return Ok(origin);
                };
                let corrupted = || corrupt(origin.meaning.source_scope);
                let restored = current
                    .authored_source_restoration()
                    .ok_or_else(corrupted)?;
                checkpoint(
                    context,
                    bound
                        .saturating_add(restored.entities.len())
                        .saturating_add(origin.meaning.family.len())
                        .saturating_add(1) as u64,
                )
                .map_err(custody::execution_stop)?;
                let old = restored
                    .entities
                    .iter()
                    .find(|(_, fresh)| *fresh == origin.meaning.source_scope)
                    .map(|(old, _)| *old)
                    .ok_or_else(corrupted)?;
                let mut original = origin.meaning.clone();
                original.source_scope = old;
                current = validated_history
                    .by_commit(
                        predecessor,
                        &mut |commit| {
                            inputs
                                .canonical_history
                                .demand_commit_envelope(execution, commit)
                                .map_err(Stop::Request)
                        },
                        execution,
                        context,
                    )
                    .map_err(custody::cold_stop)?
                    .ok_or_else(corrupted)?;
                origin = current
                    .output_origin_for_route(&original, context)
                    .map(Arc::clone)
                    .map_err(|stop| match stop {
                        worth_relational_branch::internal_port::history::data::codec_retirement_closure::CodecRetirementClosureStop::Request(cause) => {
                            custody::execution_stop(cause)
                        }
                        worth_relational_branch::internal_port::history::data::codec_retirement_closure::CodecRetirementClosureStop::Payload(_) => {
                            corrupt(old)
                        }
                    })?;
            }
            Err(corrupt(origin.meaning.source_scope))
        })
        .map_err(custody::execution_stop)?
        .0?;
    Ok(Some(RelationalRestoredProducerPrior {
        publication,
        antecedent,
    }))
}

fn corrupt(source: EntityId) -> TransactionCommitError {
    TransactionCommitError::publication(
        contracts::publication::data::publication_error::PublicationError::authored_retirement(
            contracts::publication::data::authored_retirement_refusal::AuthoredRetirementRefusal::CorruptPublication,
            RecordRef::Entity(source),
        ),
    )
}
