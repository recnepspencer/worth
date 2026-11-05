//! A Native-restored publication and its actual producer antecedent, without replay inputs.
use super::*;
use crate::{
    branch::AdmittedRelationalBranchBasis,
    history::data::{
        CanonicalCommitEnvelope, CanonicalOutputPublication, CanonicalRequiredOutputOrigin,
    },
    identity::data::EntityId,
    runtime::RelationalRuntime,
    transactions::data::RecordRef,
};
use std::sync::Arc;
use worth_execution::ExecutionRequest;

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
impl RelationalRuntime {
    pub fn readmit_restored_producer_prior(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        installed: &RelationalInstalledApplicationPublicationAuthority,
        basis: &AdmittedRelationalBranchBasis,
        meaning: &RequiredOutputPublicationMeaning,
    ) -> Result<Option<RelationalRestoredProducerPrior>, TransactionCommitError> {
        let Some(publication) =
            self.readmit_native_output_transition_successor(execution, installed, basis, meaning)?
        else {
            return Ok(None);
        };
        if !publication.is_authored_restoration() && !publication.is_historical_selection() {
            return Ok(None);
        }
        let antecedent = execution
            .run_native_with_checkpoint(|context| {
                let mut current: Arc<CanonicalCommitEnvelope> = self
                    .history
                    .canonical_envelope(publication.origin.commit.commit_id)
                    .ok_or_else(|| corrupt(publication.origin.meaning.source_scope))?;
                let mut origin = Arc::clone(&publication.origin);
                let bound = self.history.catalog_len();
                for _ in 0..=bound {
                    if matches!(
                        origin.publication,
                        CanonicalOutputPublication::HistoricalSelection { .. }
                    ) {
                        context.checkpoint(1).map_err(custody::execution_stop)?;
                        let meaning = origin.meaning.clone();
                        current = crate::history::data::historical_producer_antecedent(
                            self, &current, &origin, context,
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
                    context
                        .checkpoint(
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
                    current = self
                        .history
                        .canonical_envelope(predecessor)
                        .ok_or_else(corrupted)?;
                    origin = current
                        .output_origin_for_route(&original, context)
                        .map(Arc::clone)
                        .map_err(|stop| match stop {
                            crate::history::data::CodecRetirementClosureStop::Request(cause) => {
                                custody::execution_stop(cause)
                            }
                            crate::history::data::CodecRetirementClosureStop::Payload(_) => {
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
}
fn corrupt(source: EntityId) -> TransactionCommitError {
    TransactionCommitError::publication(
        crate::publication::data::PublicationError::authored_retirement(
            crate::publication::data::AuthoredRetirementRefusal::CorruptPublication,
            RecordRef::Entity(source),
        ),
    )
}
