use bank_domain::accounting::BankAccount;
use bank_domain::model::AccountJournalRevision;
use bank_domain::proposals::BankProposedEffect;
use bank_domain::schema::*;
use worth_query_host::facade::primary_graph::WorthQueryApplicationEffectProgramBuilder;

use super::{
    application_idempotency, entity_key, BankCommitPreparationDenial, BankMutationCommitOutcome,
};
use crate::graph_bootstrap::account_key;
use crate::{BankAuthorizedProposal, BankIdentityRuntime};

impl BankIdentityRuntime {
    pub fn commit_create_personal_account(
        &self,
        proposal: BankAuthorizedProposal<
            CreatePersonalAccountOperation,
            CreatePersonalAccount,
            Institution,
            bank_domain::model::InstitutionId,
        >,
    ) -> Result<BankMutationCommitOutcome, BankCommitPreparationDenial> {
        let (admission, invariant, projection) = proposal.into_parts();
        let account = exact_account(invariant.effects())?;
        let owner = account
            .personal_owner()
            .ok_or(BankCommitPreparationDenial::InvalidProposalShape)?;
        let (_, institution_id, query_admission) = admission.into_parts();
        if account.institution() != institution_id || account.business_owner().is_some() {
            return Err(BankCommitPreparationDenial::InvalidProposalShape);
        }
        let reads = self
            .application_runtime()
            .begin_projected_application_read_attempt(query_admission, projection)?;
        let institution =
            reads.resolve_entity(InstitutionIdentityField::reference(), institution_id)?;
        let owner = reads.resolve_entity(PrincipalIdentityField::reference(), owner)?;
        let mut effects = reads
            .complete_projected_dependencies()?
            .begin_effect_program();
        let institution = effects.existing_entity(&institution)?;
        let owner = effects.existing_entity(&owner)?;
        let created = initialize_account(&mut effects, &institution, account)?;
        effects.link(
            InstitutionAccount::reference(),
            format!("institution-account:{}", account.id().canonical_text()),
            &institution,
            &created,
        )?;
        effects.link(
            PersonalOwner::reference(),
            format!("personal-owner:{}", account.id().canonical_text()),
            &owner,
            &created,
        )?;
        commit(self, invariant, effects)
    }

    pub fn commit_create_business_account(
        &self,
        proposal: BankAuthorizedProposal<
            CreateBusinessAccountOperation,
            CreateBusinessAccount,
            Institution,
            bank_domain::model::InstitutionId,
        >,
    ) -> Result<BankMutationCommitOutcome, BankCommitPreparationDenial> {
        let (admission, invariant, projection) = proposal.into_parts();
        let account = exact_account(invariant.effects())?;
        let business = account
            .business_owner()
            .ok_or(BankCommitPreparationDenial::InvalidProposalShape)?;
        let (_, institution_id, query_admission) = admission.into_parts();
        if account.institution() != institution_id || account.personal_owner().is_some() {
            return Err(BankCommitPreparationDenial::InvalidProposalShape);
        }
        let reads = self
            .application_runtime()
            .begin_projected_application_read_attempt(query_admission, projection)?;
        let institution =
            reads.resolve_entity(InstitutionIdentityField::reference(), institution_id)?;
        let business = reads.resolve_entity(BusinessIdentityField::reference(), business)?;
        let mut effects = reads
            .complete_projected_dependencies()?
            .begin_effect_program();
        let institution = effects.existing_entity(&institution)?;
        let business = effects.existing_entity(&business)?;
        let created = initialize_account(&mut effects, &institution, account)?;
        effects.link(
            InstitutionAccount::reference(),
            format!("institution-account:{}", account.id().canonical_text()),
            &institution,
            &created,
        )?;
        effects.link(
            BusinessAccount::reference(),
            format!("business-account:{}", account.id().canonical_text()),
            &business,
            &created,
        )?;
        commit(self, invariant, effects)
    }
}

fn exact_account(
    effects: &[BankProposedEffect],
) -> Result<&BankAccount, BankCommitPreparationDenial> {
    let [BankProposedEffect::CreateAccount(account)] = effects else {
        return Err(BankCommitPreparationDenial::InvalidProposalShape);
    };
    Ok(account)
}

fn initialize_account<Operation, Input, Scope, ContextEntity>(
    effects: &mut WorthQueryApplicationEffectProgramBuilder<BankSchema, Operation, Input, Scope>,
    context: &worth_query_host::facade::primary_graph::WorthQueryApplicationEffectEntity<
        BankSchema,
        ContextEntity,
    >,
    account: &BankAccount,
) -> Result<
    worth_query_host::facade::primary_graph::WorthQueryApplicationEffectEntity<BankSchema, Account>,
    BankCommitPreparationDenial,
>
where
    Account: worth_query_host::facade::domain::OperationCreates<Operation>,
    AccountIdentity: worth_query_host::facade::domain::OperationWrites<Operation>,
    AccountDisplayName: worth_query_host::facade::domain::OperationWrites<Operation>,
    AccountingRevision: worth_query_host::facade::domain::OperationWrites<Operation>,
    Kind: worth_query_host::facade::domain::OperationWrites<Operation>,
    Status: worth_query_host::facade::domain::OperationWrites<Operation>,
{
    let created = effects.create_entity_in_context(
        context,
        Account::reference(),
        entity_key(account_key(account.id()))?,
    )?;
    effects.initialize_field(&created, AccountIdentity::reference(), account.id())?;
    effects.initialize_field(
        &created,
        AccountDisplayName::reference(),
        account.display_name().clone(),
    )?;
    effects.initialize_field(
        &created,
        AccountingRevision::reference(),
        AccountJournalRevision::default(),
    )?;
    effects.initialize_field(&created, Kind::reference(), account.kind())?;
    effects.initialize_field(&created, Status::reference(), account.status())?;
    Ok(created)
}

fn commit<Operation, Input, Scope>(
    runtime: &BankIdentityRuntime,
    invariant: bank_domain::proposals::BankInvariantApprovedProposal,
    effects: WorthQueryApplicationEffectProgramBuilder<BankSchema, Operation, Input, Scope>,
) -> Result<BankMutationCommitOutcome, BankCommitPreparationDenial>
where
    Input: Clone + Send + Sync + 'static,
{
    let idempotency = application_idempotency(&invariant);
    let program = effects.finish()?;
    Ok(runtime
        .application_runtime()
        .compare_and_commit_application(program, idempotency)
        .into())
}
