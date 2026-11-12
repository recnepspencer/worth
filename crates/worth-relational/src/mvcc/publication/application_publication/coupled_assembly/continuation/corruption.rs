//! Duplicate only a real issued phase at the closed corruption receiver.
use super::{
    ExecutionRequest, RelationalAuthoredCoupledCandidate, RelationalCoupledProducerContinuation,
    TransactionCommitError,
};
impl RelationalAuthoredCoupledCandidate<'_> {
    pub(in crate::mvcc::publication::application_publication::coupled_assembly) fn replay_continuation_probe(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<(), TransactionCommitError> {
        let token = self.next_producer_round(execution, self.parts.len() - 1)?;
        // Closed receiver corruption: duplicate only the real issued token.
        // No constructor or clone for this lifecycle phase reaches Query.
        let replay = RelationalCoupledProducerContinuation {
            identity: token.identity.clone(),
            sequence: token.sequence,
            part: token.part,
            phase: token.phase,
            source: token.source,
        };
        self.continuation
            .consume(token, self.identity, self.applied_parts())?;
        self.continuation
            .consume(replay, self.identity, self.applied_parts())?;
        Ok(())
    }

    /// Re-present the real writer's own commands as a continued round of the
    /// route Native just applied. Probe 7 returns every command; 8 only the
    /// commands that patch an existing record, whose values D already holds;
    /// 9 those same patches with each unsigned value moved by one.
    pub(in crate::mvcc::publication::application_publication::coupled_assembly) fn continued_round_probe(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        posture: super::super::RelationalCoupledProducerPosture,
        content: &crate::content::RelationalContentAttachments,
        batches: Vec<crate::transactions::data::WorkerIntentBatch>,
        probe: u8,
    ) -> Result<(), super::super::RelationalCoupledAssemblyFailure> {
        use crate::transactions::data::{AspectFieldPatch, EntityMutationIntent, MutationIntent};
        use worth_foundational::facade::AspectValue;
        let mut round = batches.clone();
        let unchanged_content = crate::content::RelationalContentAttachments::default();
        if probe != 7 {
            for batch in &mut round {
                batch.intents.retain(super::round::patches_in_place);
            }
            if round.iter().all(|batch| batch.intents.is_empty()) {
                return Err(super::part::reject(
                    super::Cause::CorruptPublication,
                    self.authored_source(),
                )
                .into());
            }
        }
        if probe == 9 {
            for intent in round.iter_mut().flat_map(|batch| &mut batch.intents) {
                if let MutationIntent::Entity(EntityMutationIntent::UpdateFields(update)) = intent {
                    update.fields = AspectFieldPatch::new(
                        update
                            .fields
                            .iter()
                            .map(|(locator, value)| {
                                let moved = match value {
                                    AspectValue::UInt64(value) => {
                                        AspectValue::UInt64(value.wrapping_add(1))
                                    }
                                    other => other.clone(),
                                };
                                (locator.clone(), moved)
                            })
                            .collect(),
                    );
                }
            }
        }
        self.append_publication(execution, posture, content, batches)?;
        let token = self.next_producer_round(execution, self.parts.len() - 1)?;
        self.verify_producer_round(
            execution,
            token,
            if probe == 7 {
                content
            } else {
                &unchanged_content
            },
            round,
        )
    }
}
