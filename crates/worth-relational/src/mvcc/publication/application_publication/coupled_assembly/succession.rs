//! A successor round supersedes its chain's live round (#119): the live round
//! is retracted to the image before the chain's head, the new round applies
//! as written, and it becomes the chain's live round. The commit is what the
//! chain's last round alone would publish at the head's position, or the
//! round refuses with its #119 reason before anything publishes.
use worth_relational_publication::internal_port::mvcc::publication::application_publication::coupled_assembly::producer_posture::RelationalCoupledProducerPostureOwnerPort as _;
use super::*;
use crate::publication::data::CoupledProvenancePostponement as Reason;
use crate::transactions::data::primitives::WorkerIntentBatch;

pub(crate) use worth_relational_publication::internal_port::mvcc::publication::application_publication::coupled_assembly::succession::{adoption, guards, indexes, ledger, route, segment};
pub(super) use adoption::{appended_rounds, batches_after_withdrawal};
use indexes::MemberId;

mod completion;
mod retraction;

impl RelationalAuthoredCoupledCandidate<'_> {
    /// Supersede owned route `token`'s chain with a new round of the same
    /// route, producer and posture kind (`append_named_successor_publication`).
    pub fn append_successor_publication(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        token: RelationalCoupledProducerContinuation,
        publication: RelationalCoupledProducerPosture,
        content: &crate::content::RelationalContentAttachments,
        batches: Vec<WorkerIntentBatch>,
    ) -> Result<(), RelationalCoupledAssemblyFailure> {
        self.append_named_successor_publication(
            execution,
            token,
            publication,
            content,
            batches,
            None,
        )
    }

    /// The same round, carrying its naming plan. Its keys take the chain's
    /// coordinates (a new key gets a new one) and partitions; a key an earlier
    /// round created keeps that identity. Refuses `IntervenedOutput`,
    /// `PriorEffectRead` or `RetiredReference` (#119 §2.3) with no effect.
    pub fn append_named_successor_publication(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        token: RelationalCoupledProducerContinuation,
        mut publication: RelationalCoupledProducerPosture,
        content: &crate::content::RelationalContentAttachments,
        mut batches: Vec<WorkerIntentBatch>,
        mut naming: Option<crate::lineage::data::CommitNamingPlan>,
    ) -> Result<(), RelationalCoupledAssemblyFailure> {
        let index = self
            .continuation
            .consume(token, self.identity, self.applied_parts())?;
        let source = route::require_same_route(&self.parts[index].publication, &publication)?;
        let predecessor = super::participant_admission::admit_posture(
            self.binding,
            self.runtime.services.runtime_instance_id(),
            &self.mutation.outcome.created_entities,
            execution,
            &publication,
        )?;
        let member = MemberId::Part(index);
        let head_ordinal = self.parts[index].chain.head_ordinal;
        self.charge(execution, self.parts[index].chain.touched.len())?;
        if let Some(record) = guards::intervened(
            &self.indexes,
            &self.parts[index].chain.touched,
            member,
            head_ordinal,
        ) {
            return Err(continuation::postponed(
                Reason::IntervenedOutput,
                guards::subject(record, source),
            )
            .into());
        }
        let interner = self.mutation.candidate_symbols().interner();
        let chain = &mut self.parts[index].chain;
        super::local_keys::rescope(
            execution,
            &mut chain.local_keys,
            chain.scope,
            &self.mutation.outcome.created_entities,
            &chain.ledger,
            interner,
            &mut batches,
            publication.roles_mut(),
            naming.as_mut(),
        )?;
        super::local_keys::repartition(
            execution,
            &chain.ledger,
            interner,
            &mut batches,
            publication.roles_mut(),
            naming.as_mut(),
        )?;
        let superseded = self.retract_chain(execution, index)?;
        let named = guards::named_records(
            batches.iter().flat_map(|batch| &batch.intents),
            publication.roles(),
            naming.as_ref(),
        );
        self.charge(execution, named.len())?;
        if let Some(record) = guards::stale(&self.parts[index].chain.ledger, &named) {
            return Err(continuation::postponed(
                Reason::PriorEffectRead,
                guards::subject(record, source),
            )
            .into());
        }
        let prebound = self.parts[index].chain.ledger.prebound();
        self.mutation.identities.records.prebind(prebound);
        let start = segment::SpanStart::of(&self.mutation.outcome.effect);
        let first_batch = self.transaction.batches().len();
        let applied = self.apply_suffix(
            execution,
            part::SuffixWriter::Successor {
                index,
                source,
                predecessor: predecessor.as_deref(),
            },
            content,
            batches,
            naming.as_mut(),
        );
        self.mutation.identities.records.prebind(Default::default());
        let applied = applied?;
        let span = start.end(&self.mutation.outcome.effect, applied.has_entity_deletions);
        self.guard_applied_round(execution, index, source, &applied.records)?;
        self.parts[index].chain.ledger.bind_round(&applied.creates);
        if let Some(record) =
            guards::retired(&self.parts[index].chain.ledger, &self.indexes, member)
        {
            return Err(continuation::postponed(
                Reason::RetiredReference,
                guards::subject(record, source),
            )
            .into());
        }
        let references = guards::named_records(
            &applied.plan.plan().merged_intents,
            publication.roles(),
            naming.as_ref(),
        );
        self.indexes.insert(member, &applied.records, references);
        let ordinary_ids = publication
            .resolve_ordinary_roles(execution, &self.mutation.outcome.created_entities)?;
        adoption::adopt_round(
            &mut self.parts,
            index,
            adoption::AppliedRound {
                records: applied.records,
                input: applied.read,
                plan: applied.plan,
                publication,
                batches: first_batch..self.transaction.batches().len(),
                span,
                content: content.clone(),
                naming,
                creates: applied.creates,
                superseded,
            },
        );
        self.parts[index].ordinary_ids = ordinary_ids;
        self.successor_rounds += 1;
        let part = &self.parts[index];
        self.advance_pin(
            execution,
            &applied.delta_state,
            &applied.content,
            &part.records,
            &part.input,
        )
    }

    /// Option A's forward guard and the stale-base guard over the applied
    /// round's changed records (§2.3 step 8; Lean `StaleBaseOk`).
    fn guard_applied_round(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        index: usize,
        source: crate::identity::data::EntityId,
        records: &[crate::transactions::data::primitives::RecordRef],
    ) -> Result<(), RelationalCoupledAssemblyFailure> {
        self.charge(
            execution,
            records.len().saturating_mul(self.parts.len() + 1),
        )?;
        let refused = guards::applied_round(&self.parts, &self.indexes, index, records);
        match refused {
            Some(record) => Err(continuation::postponed(
                Reason::IntervenedOutput,
                guards::subject(record, source),
            )
            .into()),
            None => Ok(()),
        }
    }

    /// Charge `units` of bounded Native work.
    pub(super) fn charge(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        units: usize,
    ) -> Result<(), TransactionCommitError> {
        execution
            .run_native_with_checkpoint(|context| {
                checkpoint(context, units as u64 + 1).map_err(custody::execution_stop)
            })
            .map_err(custody::execution_stop)?
            .0
    }
}
