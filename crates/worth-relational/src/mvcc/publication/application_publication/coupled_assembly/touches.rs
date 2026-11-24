//! Append-time semantic touches and exclusive-write-set chain overlays for
//! coupled candidates (PR C §1.4–§1.5).
use super::{continuation, part, touch_projection, RelationalAuthoredCoupledCandidate};
use crate::identity::data::EntityId;
use crate::publication::data::CoupledProvenancePostponement as Reason;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use worth_execution::ExecutionRequest;

pub use touch_projection::{RelationalCoupledAppendTouches, RelationalCoupledChainOverlay};

impl RelationalAuthoredCoupledCandidate<'_> {
    /// Certified touches of producer part `index`'s latest applied round.
    pub fn part_append_touches(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        index: usize,
    ) -> Result<RelationalCoupledAppendTouches, TransactionCommitError> {
        let Some(part) = self.parts.get(index) else {
            return Err(part::reject(
                crate::publication::data::AuthoredRetirementRefusal::MissingCoupledOwner,
                self.authored_source(),
            ));
        };
        touch_projection::append_touches(
            execution,
            &self.mutation.state,
            part.commands.plan(),
            &part.records,
        )
    }

    /// Certified touches of later authored stage `stage` (0-based after prefix).
    pub fn authored_stage_append_touches(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        stage: usize,
    ) -> Result<RelationalCoupledAppendTouches, TransactionCommitError> {
        let Some(part) = self.stages.get(stage) else {
            return Err(part::reject(
                crate::publication::data::AuthoredRetirementRefusal::UnadmittedAuthoredStage,
                self.authored_source(),
            ));
        };
        touch_projection::append_touches(
            execution,
            &self.mutation.state,
            part.commands.plan(),
            &part.records,
        )
    }

    /// Verify non-intervention on chain `index`'s exclusive write set and return
    /// its head pre-pin, exclusive entities, and cumulative prior write touches.
    /// Stops with `#119 IntervenedOutput` if any later member wrote a record in
    /// this chain's write set.
    pub fn chain_overlay(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        index: usize,
    ) -> Result<RelationalCoupledChainOverlay, TransactionCommitError> {
        let Some(part) = self.parts.get(index) else {
            return Err(part::reject(
                crate::publication::data::AuthoredRetirementRefusal::MissingCoupledOwner,
                self.authored_source(),
            ));
        };
        touch_projection::chain_overlay(
            execution,
            &self.mutation.state,
            &self.mutation.outcome.effect.publication.changed_records,
            part,
        )
    }

    /// The #119 `PriorEffectRead` stop for chain `index`'s next round: Query,
    /// which owns the round's read facts, found one of them outside the
    /// chain's own-output view intersecting a key an earlier round wrote.
    /// The refusal names `record` when the read had one, else the route source.
    pub fn prior_effect_read(
        &self,
        index: usize,
        record: Option<EntityId>,
    ) -> TransactionCommitError {
        let source = self
            .parts
            .get(index)
            .and_then(|part| part.publication.producing_meaning())
            .map_or_else(|| self.authored_source(), |meaning| meaning.source_scope);
        continuation::postponed(Reason::PriorEffectRead, record.unwrap_or(source))
    }
}
