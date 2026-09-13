use bank_domain::proposals::BankProposedEffect;
use bank_domain::schema::*;

use super::journal::{lower_journal, resolve_journal_accounts};
use super::{application_idempotency, BankCommitPreparationDenial, BankMutationCommitOutcome};
use crate::{BankAuthorizedProposal, BankIdentityRuntime};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationEffectProgram, WorthQueryApplicationIdempotencyBinding,
};

type ReverseJournalEffectProgram = WorthQueryApplicationEffectProgram<
    BankSchema,
    ReverseJournalOperation,
    ReverseJournal,
    Institution,
>;

impl BankIdentityRuntime {
    pub fn commit_reverse_journal(
        &self,
        proposal: BankAuthorizedProposal<
            ReverseJournalOperation,
            ReverseJournal,
            Institution,
            bank_domain::model::InstitutionId,
        >,
    ) -> Result<BankMutationCommitOutcome, BankCommitPreparationDenial> {
        let (program, idempotency) = self.materialize_reverse_journal(proposal)?;
        Ok(self
            .application_runtime()
            .compare_and_commit_application(program, idempotency)
            .into())
    }

    pub(crate) fn materialize_reverse_journal(
        &self,
        proposal: BankAuthorizedProposal<
            ReverseJournalOperation,
            ReverseJournal,
            Institution,
            bank_domain::model::InstitutionId,
        >,
    ) -> Result<
        (
            ReverseJournalEffectProgram,
            WorthQueryApplicationIdempotencyBinding,
        ),
        BankCommitPreparationDenial,
    > {
        let (admission, invariant, projection) = proposal.into_parts();
        let (original, reversal) = exact_reversal(invariant.effects())?;
        let (_, _, query_admission) = admission.into_parts();
        let mut reads = self
            .application_runtime()
            .begin_projected_application_read_attempt(query_admission, projection)?;
        let original = reads.resolve_entity(JournalIdentityField::reference(), original)?;
        let accounts = resolve_journal_accounts(&mut reads, reversal)?;
        let mut effects = reads
            .complete_projected_dependencies()?
            .begin_effect_program();
        let reversal_entity = lower_journal(&mut effects, reversal, accounts)?;
        let original = effects.existing_entity(&original)?;
        effects.link(
            JournalReversal::reference(),
            format!("journal-reversal:{}", reversal.id().canonical_text()),
            &reversal_entity,
            &original,
        )?;
        let idempotency = application_idempotency(&invariant);
        let program = effects.finish()?;
        Ok((program, idempotency))
    }
}

fn exact_reversal(
    effects: &[BankProposedEffect],
) -> Result<
    (
        bank_domain::model::JournalEntryId,
        &bank_domain::accounting::BankJournalEntry,
    ),
    BankCommitPreparationDenial,
> {
    let [BankProposedEffect::ReverseJournal { original, reversal }] = effects else {
        return Err(BankCommitPreparationDenial::InvalidProposalShape);
    };
    if reversal.reversal_of() != Some(*original) {
        return Err(BankCommitPreparationDenial::InvalidProposalShape);
    }
    Ok((*original, reversal))
}
