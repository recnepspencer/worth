//! A reopened route that already has a part derives its chain's next round
//! (PR C §1.5): its own records read as they were before the chain, any
//! other read of a key the chain wrote stops (`PriorEffectRead`), and Native
//! folds the round in place, so the route keeps one part and one writer.
use super::*;
use crate::domain_computation::primary_graph::invariant_projection::read_basis::OwnOutputOverlay;

/// The chain a reopened route continues: its Native part, its own-output
/// view and the keys its earlier rounds wrote.
pub(super) struct ContinuedChain {
    pub(super) index: usize,
    pub(super) own: OwnOutputOverlay,
    prior: Vec<worth_relational::facade::mvcc::ValidatedMutationTouch>,
}

impl<'run, S: ApplicationSchema> CoupledApplicationAssembly<'_, 'run, '_, S> {
    /// The chain this route's next round continues, if it is reopened and
    /// already has a part. Native refuses `IntervenedOutput` here when
    /// another writer changed a record of the chain after its earlier round.
    pub(super) fn continued_chain(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
        family: &str,
        scope: worth_relational::facade::identity::EntityId,
        partition: &[u8; 32],
    ) -> Result<Option<ContinuedChain>, CoupledSourceReadStop> {
        if !self.superseding.get() {
            return Ok(None);
        }
        let Some(index) = candidate.prepared_route_index(execution, family, scope, partition)?
        else {
            return Ok(None);
        };
        let chain = candidate.chain_overlay(execution, index)?;
        Ok(Some(ContinuedChain {
            index,
            own: OwnOutputOverlay::from_chain(&chain),
            prior: chain.prior_touches().to_vec(),
        }))
    }

    /// Stop the round before append when one of its reads outside the
    /// own-output view intersects a key its chain's earlier rounds wrote.
    pub(super) fn require_independent(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
        chain: &ContinuedChain,
        session: crate::domain_computation::WorthQueryProviderSessionView<'_>,
        current: &worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
    ) -> Result<(), CoupledSourceReadStop> {
        match self
            .application
            .runtime
            .primary_provider
            .coupled_writer_prior_effect(
                execution,
                session,
                self.before,
                current,
                &chain.own,
                &chain.prior,
            )
            .map_err(CoupledSourceReadStop::capture)?
        {
            None => Ok(()),
            Some(record) => Err(candidate.prior_effect_read(chain.index, record).into()),
        }
    }

    /// The successor's original idempotency record is the chain's record:
    /// its key binds the round's own dependency, so it is re-created under
    /// the head's local coordinate with the successor's values (§1.5
    /// Identity). The key is only a create coordinate inside the candidate.
    pub(super) fn rekey_original(
        &self,
        index: usize,
        successor: crate::domain_computation::WorthQueryProviderSessionView<'_>,
        batch: &mut worth_relational::facade::transactions::WorkerIntentBatch,
    ) {
        use worth_relational::facade::transactions::{CreateIntent, MutationIntent};
        let provider = &self.application.runtime.primary_provider;
        let head = {
            let writers = self.writers.borrow();
            let Some(CoupledWriterStage::Inspected(writer)) = writers[index].stage.as_ref() else {
                unreachable!("an appended chain keeps its inspected writer")
            };
            provider.coupled_writer_original_key(writer.session())
        };
        let own = provider.coupled_writer_original_key(successor);
        for intent in &mut batch.intents {
            if let MutationIntent::Create(CreateIntent::Entity(spec)) = intent
                && spec.client_key.as_raw_str() == Some(own.as_str())
            {
                spec.client_key = worth_relational::facade::symbols::ClientKey::raw(head.clone());
            }
        }
    }

    /// Append writer `next`'s round `(publication, content, batch)`: as a new
    /// part, or folded into its continued `chain` (§1.5), where the
    /// superseded part retires and `next` takes the chain's writer slot.
    /// Returns the part index.
    pub(super) fn append_round(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        candidate: &mut RelationalAuthoredCoupledCandidate<'_>,
        chain: Option<usize>,
        next: CoupledWriter<'run, S>,
        (publication, content, batch): (
            worth_relational::facade::mvcc::RelationalCoupledProducerPosture,
            worth_relational::facade::content::RelationalContentAttachments,
            worth_relational::facade::transactions::WorkerIntentBatch,
        ),
    ) -> Result<usize, CoupledSourceReadStop> {
        let Some(index) = chain else {
            self.writers.borrow_mut().push(next);
            let fault = self
                .application
                .runtime
                .primary_provider
                .application_attempt_operation_control
                .take_coupled_participant_corruption(self.writers.borrow().len());
            match fault {
                1..=9 => candidate.append_corrupt_participant_for_test(
                    execution,
                    publication,
                    &content,
                    vec![batch],
                    fault,
                )?,
                _ => candidate.append_publication(execution, publication, &content, vec![batch])?,
            }
            return Ok(self.writers.borrow().len() - 1);
        };
        let superseded = self.writers.borrow()[index].produces;
        self.retire_part(superseded);
        self.replace_writer(execution, index, next);
        let token = candidate.next_producer_round(execution, index)?;
        candidate.append_successor_publication(
            execution,
            token,
            publication,
            &content,
            vec![batch],
        )?;
        Ok(index)
    }

    /// The successor writer takes its chain's slot: the superseded round's
    /// session is discarded, never published (Native folded it).
    fn replace_writer(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        index: usize,
        successor: CoupledWriter<'run, S>,
    ) {
        let superseded = std::mem::replace(&mut self.writers.borrow_mut()[index], successor);
        self.successors.set(self.successors.get() + 1);
        let outcome = match superseded.stage {
            Some(CoupledWriterStage::Inspected(writer)) => {
                writer.abort(execution, self.phase, self.application)
            }
            Some(CoupledWriterStage::Prepared(writer)) => {
                writer.abort(execution, self.phase, self.application)
            }
            None => return,
        };
        self.retain_abort_outcome(outcome);
    }
}
