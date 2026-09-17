use bank_domain::accounting::BankAccount;
use bank_domain::model::AccountJournalRevision;
use bank_domain::proposals::{BankProposalDenial, BankProposalEngine, BankProposedEffect};
use bank_domain::schema::{
    Account, AccountDisplayName, AccountIdentity, AccountKind, AccountStatus, AccountingRevision,
    BankSchema, BusinessAccount, BusinessIdentityField, CreateBusinessAccount,
    CreateBusinessAccountMutationBinding, CreateBusinessAccountResult, InstitutionAccount,
    InstitutionIdentityField, Kind, Status, CREATE_BUSINESS_ACCOUNT_OUTPUT_ACCOUNT,
};
use worth_query_host::facade::declaration::application_operation::{
    ApplicationCandidateRequirements, ApplicationMutationBinding,
};
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerInterruption, HandlerResult,
    OperationHandler, WorthQueryApplicationEntityKey, WorthQueryApplicationOutputRole,
    WorthQueryCreateOutput,
};

use crate::bank_projection::project_business_account_creation;
use crate::graph_bootstrap::account_key;
use crate::operation_admission::bank_operation_scope_binding;

pub(crate) struct CreateBusinessAccountHandler;

impl OperationHandler<BankSchema, CreateBusinessAccountMutationBinding>
    for CreateBusinessAccountHandler
{
    fn decide(
        &self,
        input: &CreateBusinessAccount,
        reader: &mut DecisionReader<'_, '_, '_, BankSchema, CreateBusinessAccountMutationBinding>,
    ) -> HandlerResult<bank_domain::proposals::BankInvariantApprovedProposal, BankProposalDenial>
    {
        if let Err(interruption) = reader.checkpoint() {
            return interrupted(interruption);
        }
        let institution = reader.scope().clone();
        let snapshot = match project_business_account_creation(reader.reader(), &institution, input)
        {
            Ok(snapshot) => snapshot,
            Err(error) => return execution_denied(error),
        };
        match BankProposalEngine::prepare_create_business_account(
            &snapshot,
            bank_operation_scope_binding(reader.operation_scope_binding()),
            reader.idempotency_key(),
            input,
        ) {
            Ok(proposal) => HandlerResult::Completed(proposal),
            Err(denial) => HandlerResult::DomainDenied(denial),
        }
    }

    fn candidate_requirements(
        &self,
        _: &CreateBusinessAccount,
        _: &bank_domain::proposals::BankInvariantApprovedProposal,
    ) -> ApplicationCandidateRequirements {
        CreateBusinessAccountMutationBinding::CANDIDATES
    }

    fn build_candidate(
        &self,
        _: &CreateBusinessAccount,
        decision: bank_domain::proposals::BankInvariantApprovedProposal,
        candidate: &mut CandidateWriter<'_, BankSchema, CreateBusinessAccountMutationBinding>,
    ) -> HandlerResult<CreateBusinessAccountResult, BankProposalDenial> {
        if let Err(interruption) = candidate.checkpoint() {
            return interrupted(interruption);
        }
        match exact_account(decision.effects())
            .and_then(|account| author_account(account, candidate))
        {
            Ok(result) => HandlerResult::Completed(result),
            Err(error) => HandlerResult::ExecutionDenied(error),
        }
    }
}

fn exact_account(effects: &[BankProposedEffect]) -> Result<&BankAccount, HandlerExecutionDenial> {
    let [BankProposedEffect::CreateAccount(account)] = effects else {
        return Err(HandlerExecutionDenial::new(InvalidBusinessAccountCandidate));
    };
    if account.business_owner().is_none() || account.personal_owner().is_some() {
        return Err(HandlerExecutionDenial::new(InvalidBusinessAccountCandidate));
    }
    Ok(account)
}

fn author_account(
    account: &BankAccount,
    candidate: &mut CandidateWriter<'_, BankSchema, CreateBusinessAccountMutationBinding>,
) -> Result<CreateBusinessAccountResult, HandlerExecutionDenial> {
    let business_id = account
        .business_owner()
        .ok_or_else(|| HandlerExecutionDenial::new(InvalidBusinessAccountCandidate))?;
    let institution = candidate
        .resolve_entity(InstitutionIdentityField::reference(), account.institution())
        .map_err(HandlerExecutionDenial::new)?;
    let business = candidate
        .resolve_entity(BusinessIdentityField::reference(), business_id)
        .map_err(HandlerExecutionDenial::new)?;
    let created = candidate
        .create_entity_in_context(
            &institution,
            Account::reference(),
            WorthQueryApplicationEntityKey::new(account_key(account.id()))
                .map_err(HandlerExecutionDenial::new)?,
        )
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .initialize_field(&created, AccountIdentity::reference(), account.id())
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .initialize_field(
            &created,
            AccountDisplayName::reference(),
            account.display_name().clone(),
        )
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .initialize_field(
            &created,
            AccountingRevision::reference(),
            AccountJournalRevision::default(),
        )
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .initialize_field(&created, Kind::reference(), AccountKind::Business)
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .initialize_field(&created, Status::reference(), AccountStatus::Open)
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .link(
            InstitutionAccount::reference(),
            format!("institution-account:{}", account.id().canonical_text()),
            &institution,
            &created,
        )
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .link(
            BusinessAccount::reference(),
            format!("business-account:{}", account.id().canonical_text()),
            &business,
            &created,
        )
        .map_err(HandlerExecutionDenial::new)?;
    candidate
        .create_output(
            WorthQueryApplicationOutputRole::<
                CreateBusinessAccountMutationBinding,
                Account,
                WorthQueryCreateOutput,
            >::from_static(CREATE_BUSINESS_ACCOUNT_OUTPUT_ACCOUNT),
            &created,
        )
        .map_err(HandlerExecutionDenial::new)?;
    Ok(CreateBusinessAccountResult {
        account: account.id(),
    })
}

fn execution_denied<Value, Denial>(
    error: impl std::error::Error + Send + Sync + 'static,
) -> HandlerResult<Value, Denial> {
    HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error))
}

fn interrupted<Value, Denial>(interruption: HandlerInterruption) -> HandlerResult<Value, Denial> {
    match interruption {
        HandlerInterruption::Cancelled => HandlerResult::Cancelled,
        HandlerInterruption::DeadlineExceeded => HandlerResult::DeadlineExceeded,
    }
}

#[derive(Debug)]
struct InvalidBusinessAccountCandidate;

impl std::fmt::Display for InvalidBusinessAccountCandidate {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("business account proposal has an invalid effect shape")
    }
}

impl std::error::Error for InvalidBusinessAccountCandidate {}
