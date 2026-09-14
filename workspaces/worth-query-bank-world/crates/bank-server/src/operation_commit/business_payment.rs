use bank_domain::payments::BusinessPayment;
use bank_domain::proposals::BankProposedEffect;
use bank_domain::schema::*;

use super::journal::{lower_journal, resolve_journal_accounts};
use super::{
    application_idempotency, entity_key, BankCommitPreparationDenial, BankMutationCommitOutcome,
};
use crate::graph_bootstrap::{approval_key, payment_key};
use crate::{BankAuthorizedProposal, BankIdentityRuntime};

impl BankIdentityRuntime {
    pub fn commit_initiate_business_payment(
        &self,
        proposal: BankAuthorizedProposal<
            InitiateBusinessPaymentOperation,
            InitiateBusinessPayment,
            Business,
            bank_domain::model::BusinessId,
        >,
    ) -> Result<BankMutationCommitOutcome, BankCommitPreparationDenial> {
        let (admission, invariant, projection) = proposal.into_parts();
        let payment = exact_created_payment(invariant.effects())?;
        let (_, business_id, query_admission) = admission.into_parts();
        if payment.business() != business_id {
            return Err(BankCommitPreparationDenial::InvalidProposalShape);
        }
        let reads = self
            .application_runtime()
            .begin_projected_application_read_attempt(query_admission, projection)?;
        let business = reads.resolve_entity(BusinessIdentityField::reference(), business_id)?;
        let source = reads.resolve_entity(AccountIdentity::reference(), payment.source())?;
        let destination =
            reads.resolve_entity(AccountIdentity::reference(), payment.destination())?;
        let initiator =
            reads.resolve_entity(PrincipalIdentityField::reference(), payment.initiator())?;
        let mut effects = reads
            .complete_projected_dependencies()?
            .begin_effect_program();
        let business = effects.existing_entity(&business)?;
        let payment_entity = effects.create_entity_in_context(
            &business,
            PaymentIntent::reference(),
            entity_key(payment_key(payment.id()))?,
        )?;
        effects.initialize_field(
            &payment_entity,
            PaymentIdentityField::reference(),
            payment.id(),
        )?;
        effects.initialize_field(
            &payment_entity,
            PaymentAmount::reference(),
            payment.amount(),
        )?;
        effects.initialize_field(
            &payment_entity,
            PaymentStatusField::reference(),
            payment.status(),
        )?;
        let source = effects.existing_entity(&source)?;
        let destination = effects.existing_entity(&destination)?;
        let initiator = effects.existing_entity(&initiator)?;
        effects.link(
            PaymentSource::reference(),
            format!("payment-source:{}", payment.id().canonical_text()),
            &payment_entity,
            &source,
        )?;
        effects.link(
            PaymentDestination::reference(),
            format!("payment-destination:{}", payment.id().canonical_text()),
            &payment_entity,
            &destination,
        )?;
        effects.link(
            PaymentBusiness::reference(),
            format!("payment-business:{}", payment.id().canonical_text()),
            &payment_entity,
            &business,
        )?;
        effects.link(
            PaymentInitiator::reference(),
            format!("payment-initiator:{}", payment.id().canonical_text()),
            &initiator,
            &payment_entity,
        )?;
        commit(self, invariant, effects.finish()?)
    }

    pub fn commit_approve_payment(
        &self,
        proposal: BankAuthorizedProposal<
            ApprovePaymentOperation,
            ApprovePayment,
            PaymentIntent,
            bank_domain::model::PaymentId,
        >,
    ) -> Result<BankMutationCommitOutcome, BankCommitPreparationDenial> {
        let (admission, invariant, projection) = proposal.into_parts();
        let (journal, payment) = exact_approved_payment(invariant.effects())?;
        let approver = payment
            .deciding_principal()
            .ok_or(BankCommitPreparationDenial::InvalidProposalShape)?;
        let (_, payment_id, query_admission) = admission.into_parts();
        if payment.id() != payment_id {
            return Err(BankCommitPreparationDenial::InvalidProposalShape);
        }
        let mut reads = self
            .application_runtime()
            .begin_projected_application_read_attempt(query_admission, projection)?;
        let payment_identity =
            reads.resolve_entity(PaymentIdentityField::reference(), payment_id)?;
        let approver = reads.resolve_entity(PrincipalIdentityField::reference(), approver)?;
        let accounts = resolve_journal_accounts(&mut reads, journal)?;
        let mut effects = reads
            .complete_projected_dependencies()?
            .begin_effect_program();
        lower_journal(&mut effects, journal, accounts)?;
        lower_payment_decision(&mut effects, &payment_identity, &approver, payment)?;
        commit(self, invariant, effects.finish()?)
    }

    pub fn commit_reject_payment(
        &self,
        proposal: BankAuthorizedProposal<
            RejectPaymentOperation,
            RejectPayment,
            PaymentIntent,
            bank_domain::model::PaymentId,
        >,
    ) -> Result<BankMutationCommitOutcome, BankCommitPreparationDenial> {
        let (admission, invariant, projection) = proposal.into_parts();
        let payment = exact_updated_payment(invariant.effects())?;
        let decider = payment
            .deciding_principal()
            .ok_or(BankCommitPreparationDenial::InvalidProposalShape)?;
        let (_, payment_id, query_admission) = admission.into_parts();
        if payment.id() != payment_id {
            return Err(BankCommitPreparationDenial::InvalidProposalShape);
        }
        let reads = self
            .application_runtime()
            .begin_projected_application_read_attempt(query_admission, projection)?;
        let payment_identity =
            reads.resolve_entity(PaymentIdentityField::reference(), payment_id)?;
        let decider = reads.resolve_entity(PrincipalIdentityField::reference(), decider)?;
        let mut effects = reads
            .complete_projected_dependencies()?
            .begin_effect_program();
        lower_payment_decision(&mut effects, &payment_identity, &decider, payment)?;
        commit(self, invariant, effects.finish()?)
    }
}

fn lower_payment_decision<Operation, Input>(
    effects: &mut worth_query_host::facade::primary_graph::WorthQueryApplicationEffectProgramBuilder<
        BankSchema,
        Operation,
        Input,
        PaymentIntent,
    >,
    payment: &worth_query_host::facade::primary_graph::WorthQueryApplicationEntityIdentity<
        BankSchema,
        PaymentIntent,
    >,
    decider: &worth_query_host::facade::primary_graph::WorthQueryApplicationEntityIdentity<
        BankSchema,
        Principal,
    >,
    replacement: &BusinessPayment,
) -> Result<(), BankCommitPreparationDenial>
where
    Approval: worth_query_host::facade::domain::OperationCreates<Operation>,
    PaymentStatusField: worth_query_host::facade::domain::OperationWrites<Operation>,
    PaymentApproval: worth_query_host::facade::domain::OperationLinks<Operation>,
    ApprovalPrincipal: worth_query_host::facade::domain::OperationLinks<Operation>,
{
    let payment = effects.existing_entity(payment)?;
    let decider = effects.existing_entity(decider)?;
    let approval = effects.create_entity_in_context(
        &payment,
        Approval::reference(),
        entity_key(approval_key(replacement.id()))?,
    )?;
    effects.write_field(
        &payment,
        PaymentStatusField::reference(),
        replacement.status(),
    )?;
    effects.link(
        PaymentApproval::reference(),
        format!("payment-approval:{}", replacement.id().canonical_text()),
        &payment,
        &approval,
    )?;
    effects.link(
        ApprovalPrincipal::reference(),
        format!("approval-principal:{}", replacement.id().canonical_text()),
        &approval,
        &decider,
    )?;
    Ok(())
}

fn exact_created_payment(
    effects: &[BankProposedEffect],
) -> Result<&BusinessPayment, BankCommitPreparationDenial> {
    let [BankProposedEffect::CreatePayment(payment)] = effects else {
        return Err(BankCommitPreparationDenial::InvalidProposalShape);
    };
    Ok(payment)
}

fn exact_approved_payment(
    effects: &[BankProposedEffect],
) -> Result<
    (&bank_domain::accounting::BankJournalEntry, &BusinessPayment),
    BankCommitPreparationDenial,
> {
    let [BankProposedEffect::AppendJournal(journal), BankProposedEffect::UpdatePayment {
        payment,
        replacement,
    }] = effects
    else {
        return Err(BankCommitPreparationDenial::InvalidProposalShape);
    };
    if *payment != replacement.id() {
        return Err(BankCommitPreparationDenial::InvalidProposalShape);
    }
    Ok((journal, replacement))
}

fn exact_updated_payment(
    effects: &[BankProposedEffect],
) -> Result<&BusinessPayment, BankCommitPreparationDenial> {
    let [BankProposedEffect::UpdatePayment {
        payment,
        replacement,
    }] = effects
    else {
        return Err(BankCommitPreparationDenial::InvalidProposalShape);
    };
    if *payment != replacement.id() {
        return Err(BankCommitPreparationDenial::InvalidProposalShape);
    }
    Ok(replacement)
}

fn commit<Operation, Input, Scope>(
    runtime: &BankIdentityRuntime,
    invariant: bank_domain::proposals::BankInvariantApprovedProposal,
    program: worth_query_host::facade::primary_graph::WorthQueryApplicationEffectProgram<
        BankSchema,
        Operation,
        Input,
        Scope,
    >,
) -> Result<BankMutationCommitOutcome, BankCommitPreparationDenial>
where
    Input: Clone + Send + Sync + 'static,
{
    let idempotency = application_idempotency(&invariant);
    Ok(runtime
        .application_runtime()
        .compare_and_commit_application(program, idempotency)
        .into())
}
