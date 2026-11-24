//! One owned route's later round folds into its chain in place, or refuses
//! under #119 before anything applies. The chain stays one producer part, so
//! every first-match consumer of `parts` reads its latest round.
use super::*;
use crate::publication::data::CoupledProvenancePostponement as Reason;
use crate::transactions::data::primitives::WorkerIntentBatch;

mod coverage;
mod fold;
mod proof;
mod relations;
pub(super) use fold::{appended_rounds, batches_after_withdrawal};

impl RelationalAuthoredCoupledCandidate<'_> {
    /// Fold a complete re-derivation of owned route `token`'s chain into it.
    /// Admitted only for the same route, producer and posture kind, and only
    /// when the round re-writes everything the chain wrote with no other kind
    /// of effect. Any other round refuses with its #119 reason before applying.
    pub fn append_successor_publication(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        token: RelationalCoupledProducerContinuation,
        mut publication: RelationalCoupledProducerPosture,
        content: &crate::content::RelationalContentAttachments,
        mut batches: Vec<WorkerIntentBatch>,
    ) -> Result<(), RelationalCoupledAssemblyFailure> {
        let index = self
            .continuation
            .consume(token, self.identity, self.applied_parts())?;
        let source = proof::require_same_route(&self.parts[index].publication, &publication)?;
        let predecessor = super::participant_admission::admit_posture(
            self.binding,
            self.runtime.services.runtime_instance_id(),
            &self.mutation.outcome.created_entities,
            execution,
            &publication,
        )?;
        let chain = &self.parts[index].chain;
        if super::local_keys::rescope(
            execution,
            &chain.local_keys,
            self.mutation.candidate_symbols().interner(),
            &mut batches,
            publication.roles_mut(),
        )?
        .is_err()
        {
            return Err(continuation::postponed(Reason::StructuralEffect, source).into());
        }
        super::local_keys::repartition(
            execution,
            &self.parts[index].commands.plan().merged_intents,
            self.mutation.candidate_symbols().interner(),
            &mut batches,
            publication.roles_mut(),
        )?;
        proof::prove_succession(
            execution,
            &self.parts[index],
            &self.mutation.outcome,
            self.mutation.candidate_symbols().interner(),
            content,
            &publication,
            &batches,
        )?;
        let first_record = self
            .mutation
            .outcome
            .effect
            .publication
            .changed_records
            .len();
        let first_batch = self.transaction.batches().len();
        let first_fragment = self
            .mutation
            .outcome
            .effect
            .publication
            .patch_fragments
            .len();
        let mut applied = self.apply_suffix(
            execution,
            part::SuffixWriter::Successor {
                index,
                source,
                predecessor: predecessor.as_deref(),
            },
            content,
            batches,
        )?;
        fold::fold_records(
            execution,
            fold::FoldInputs {
                schema: self.prepared.schema_authority.aspect_plans(),
                version: self.identity.proposed_version_id(),
                outcome: &mut self.mutation.outcome,
                chain: &mut self.parts[index].chain,
                state: &mut self.mutation.state,
                symbols: &mut self.mutation.identities.symbols,
            },
            first_fragment,
            &applied.plan,
            &applied.read,
            &mut applied.delta_state,
        )?;
        let records_end = self
            .mutation
            .outcome
            .effect
            .publication
            .changed_records
            .len();
        let batches_end = self.transaction.batches().len();
        let prior_round_touches =
            crate::mvcc::validation::proposal_touches::project_applied_intents(
                execution,
                &self.mutation.state,
                &self.parts[index].commands.plan().merged_intents,
            )?;
        fold::adopt_round(
            execution,
            &mut self.parts,
            index,
            prior_round_touches,
            fold::AppliedRound {
                records: &applied.records,
                input: applied.read,
                plan: applied.plan,
                publication,
                batches: first_batch..batches_end,
                record_range: first_record..records_end,
            },
        )?;
        self.successor_rounds += 1;
        let input = self.parts[index].input.clone();
        self.advance_pin(
            execution,
            &applied.delta_state,
            &applied.content,
            &applied.records,
            &input,
        )
    }
}
