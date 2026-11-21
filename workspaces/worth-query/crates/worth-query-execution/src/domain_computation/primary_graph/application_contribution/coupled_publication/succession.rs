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
        candidate: &RelationalAuthoredCoupledCandidate<'_>,
        chain: &ContinuedChain,
        session: crate::domain_computation::WorthQueryProviderSessionView<'_>,
    ) -> Result<(), CoupledSourceReadStop> {
        match self
            .application
            .runtime
            .primary_provider
            .coupled_writer_prior_effect(session, &chain.own, &chain.prior)
        {
            None => Ok(()),
            Some(record) => Err(candidate.prior_effect_read(chain.index, record).into()),
        }
    }

    /// The successor writer takes its chain's slot: the superseded round's
    /// session is discarded, never published (Native folded it).
    pub(super) fn replace_writer(
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
